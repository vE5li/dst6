mod flow;
mod signature;
mod description;

use internal::*;
use debug::*;

use std::path::Path;
use self::flow::Flow;
use self::signature::Signature;
use self::description::CONDITIONS;
use super::*;

#[derive(Debug)]
pub struct DataStack<'s> {
    items:  &'s SharedVector<Data>,
    flow:   Vec<Flow>,
    index:  usize,
}

impl<'s> DataStack<'s> {

    pub fn new(items: &'s SharedVector<Data>) -> Self {
        Self {
            items:  items,
            flow:   Vec::new(),
            index:  0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.items.len() <= self.index
    }

    pub fn pop(&mut self) -> Option<Data> {
        if self.is_empty() {
            return None;
        }
        let item = self.items[self.index].clone();
        self.advance(1);
        Some(item)
    }

    pub fn peek(&self, offset: usize) -> Option<Data> {
        match self.items.len() > self.index + offset {
            true => Some(self.items[self.index + offset].clone()),
            false => None,
        }
    }

    pub fn advance(&mut self, offset: usize) {
        self.index += offset;
    }

    pub fn ensure_empty(&self) -> Status<()> {
        if let Some(item) = self.peek(0) {
            return error!(UnexpectedImmediate, item);
        }
        return success!(())
    }

    pub fn closed(&self) -> bool {
        self.flow.is_empty()
    }

    pub fn iterate(&mut self, parameters: Vec<Data>, last: &mut Option<Data>, root: &Data, scope: &Data, build: &Data) -> Status<()> { // TODO: add step parameter

        let mut iterators = Vec::new();
        for (selector, instance) in confirm!(parameters[0].pairs()).into_iter() {
            let mut map = DataMap::new();
            map.insert(identifier!("key"), selector);
            map.insert(identifier!("value"), instance);
            iterators.push(map!(map));
        }

        if iterators.is_empty() { // FIX: dirty workarround
            confirm!(self.skip_condition(false));
            self.advance(1);
            self.skip_parameters();
            return success!(());
        }

        self.flow.push(Flow::IndexIteration(iterators, self.index));
        return self.update(true, false, last, root, scope, build);
    }

    pub fn looped_condition(&mut self, last: &mut Option<Data>, root: &Data, scope: &Data, build: &Data) -> Status<()> {
        let mut source = SharedVector::new();
        while let Some(next) = self.peek(0) {
            if next.is_list() {
                self.advance(1);
                source.push(next);
            } else {
                break;
            }
        }

        let mut source_stack = DataStack::new(&source); // FIX: dirty workarround
        let description = (*INSTRUCTIONS).get("while").unwrap();
        let extracted = confirm!(InstructionParameter::validate(&confirm!(source_stack.parameters(&last, root, &scope, build)), &description.parameters, description.variadic));
        if !confirm!(DataStack::resolve_condition(&extracted, last)).0 {
            confirm!(self.skip_condition(false));
            self.advance(1);
            self.skip_parameters();
            return success!(());
        }

        self.flow.push(Flow::While(source, last.clone(), self.index));
        return self.update(true, false, last, root, scope, build);
    }

    pub fn counted(&mut self, start: i64, end: i64, step: i64, last: &mut Option<Data>, root: &Data, scope: &Data, build: &Data) -> Status<()> {
        ensure!(step >= 0, string!("step may not be negative"));
        match start < end {
            true => self.flow.push(Flow::For(start - step, end, step, self.index)),
            false => self.flow.push(Flow::For(start + step, end, -step, self.index)),
        }
        return self.update(true, false, last, root, scope, build);
    }

    pub fn condition(&mut self, parameters: Vec<Data>, last: &mut Option<Data>, root: &Data, scope: &Data, build: &Data) -> Status<()> {
        let (state, length) = confirm!(DataStack::resolve_condition(&parameters, last));
        ensure!(length == parameters.len(), UnexpectedParameter, parameters[length].clone());
        self.flow.push(Flow::Condition(state));
        return self.update(true, false, last, root, scope, build);
    }

    pub fn dependent_condition(&mut self, last: &mut Option<Data>, root: &Data, scope: &Data, build: &Data) -> Status<()> {
        match self.flow.last().cloned() { // TODO

            Some(flow) => {
                if let Flow::Condition(state) = flow {
                    if state {
                        self.skip_parameters();
                        self.skip_condition(true);
                        return success!(());
                    }
                } else {
                    return error!(UnexpectedCompilerFunction, keyword!("else"));
                }
            },

            None => return error!(UnexpectedCompilerFunction, keyword!("else")),
        }

        let description = (*INSTRUCTIONS).get("else").unwrap();
        let parameters = confirm!(InstructionParameter::validate(&confirm!(self.parameters(&last, root, &scope, build)), &description.parameters, description.variadic));

        let state = match parameters.is_empty() {
            false => confirm!(DataStack::resolve_condition(&parameters, last)).0,
            true => true,
        };

        *self.flow.last_mut().unwrap() = Flow::Condition(state);
        return self.update(true, false, last, root, scope, build);
    }

    pub fn resolve_condition(source: &Vec<Data>, last: &Option<Data>) -> Status<(bool, usize)> {

        ensure!(!source.is_empty(), ExpectedCondition);
        let condition = unpack_keyword!(&source[0], ExpectedConditionFound, source[0].clone());
        let description = match (*CONDITIONS).get(condition.printable().as_str()) {
            Some(description) => description,
            None => return error!(string!("condition {} does not exist", condition.serialize())), // TODO:
        };

        if description.width > source.len() {
            return error!(string!("{} expected {} operants; found {}", condition.serialize(), description.width, source.len())); // TODO:
        }

        let state = match &description.signature {

            Signature::Always => true,

            Signature::NotAlways => false,

            Signature::Zero => source[1] == integer!(0) || source[1] == float!(0.0) || source[1] == character!(code, 0),

            Signature::NotZero => source[1] != integer!(0) && source[1] != float!(0.0) && source[1] != character!(code, 0),

            Signature::True => source[1] == boolean!(true),

            Signature::NotTrue => source[1] != boolean!(true),

            Signature::False => source[1] == boolean!(false),

            Signature::NotFalse => source[1] != boolean!(false),

            Signature::Empty => confirm!(source[1].length()) == 0,

            Signature::NotEmpty => confirm!(source[1].length()) != 0,

            Signature::Instruction => (*INSTRUCTIONS).contains_key(unpack_literal!(&source[1]).printable().as_str()),

            Signature::NotInstruction => !(*INSTRUCTIONS).contains_key(unpack_literal!(&source[1]).printable().as_str()),

            Signature::Condition => (*CONDITIONS).contains_key(unpack_literal!(&source[1]).printable().as_str()),

            Signature::NotCondition => !(*CONDITIONS).contains_key(unpack_literal!(&source[1]).printable().as_str()),

            Signature::LastSome => last.is_some(),

            Signature::NotLastSome => last.is_none(),

            Signature::Uppercase => confirm!(source[1].is_uppercase()),

            Signature::NotUppercase => !confirm!(source[1].is_uppercase()),

            Signature::Lowercase => confirm!(source[1].is_lowercase()),

            Signature::NotLowercase => !confirm!(source[1].is_lowercase()),

            Signature::Equals => source[1] == source[2],

            Signature::NotEquals => source[1] != source[2],

            Signature::Present => confirm!(source[1].index(&source[2])).is_some(),

            Signature::NotPresent => confirm!(source[1].index(&source[2])).is_none(),

            Signature::Bigger => confirm!(source[1].bigger(&source[2])),

            Signature::NotBigger => !confirm!(source[1].bigger(&source[2])),

            Signature::Smaller => confirm!(source[1].smaller(&source[2])),

            Signature::NotSmaller => !confirm!(source[1].smaller(&source[2])),

            Signature::Contains => confirm!(source[1].contains(&source[2])),

            Signature::NotContains => !confirm!(source[1].contains(&source[2])),

            Signature::Pure => CharacterStack::new(SharedString::from(""), None).is_pure(&unpack_literal!(&source[1])),

            Signature::NotPure => !CharacterStack::new(SharedString::from(""), None).is_pure(&unpack_literal!(&source[1])),

            Signature::FilePresent => Path::new(unpack_string!(&source[1]).printable().as_str()).exists(),

            Signature::NotFilePresent => !Path::new(unpack_string!(&source[1]).printable().as_str()).exists(),

            Signature::Map => source[1].is_map(),

            Signature::NotMap => !source[1].is_map(),

            Signature::List => source[1].is_list(),

            Signature::NotList => !source[1].is_list(),

            Signature::Path => source[1].is_path(),

            Signature::NotPath => !source[1].is_path(),

            Signature::String => source[1].is_string(),

            Signature::NotString => !source[1].is_string(),

            Signature::Character => source[1].is_character(),

            Signature::NotCharacter => !source[1].is_character(),

            Signature::Identifier => source[1].is_identifier(),

            Signature::NotIdentifier => !source[1].is_identifier(),

            Signature::Keyword => source[1].is_keyword(),

            Signature::NotKeyword => !source[1].is_keyword(),

            Signature::Integer => source[1].is_integer(),

            Signature::NotInteger => !source[1].is_integer(),

            Signature::Float => source[1].is_float(),

            Signature::NotFloat => !source[1].is_float(),

            Signature::Boolean => source[1].is_boolean(),

            Signature::NotBoolean => !source[1].is_boolean(),

            Signature::Key => source[1].is_key(),

            Signature::NotKey => !source[1].is_key(),

            Signature::Container => source[1].is_container(),

            Signature::NotContainer => !source[1].is_container(),

            Signature::Literal => source[1].is_literal(),

            Signature::NotLiteral => !source[1].is_literal(),

            Signature::Selector => source[1].is_selector(),

            Signature::NotSelector => !source[1].is_selector(),

            Signature::Number => source[1].is_number(),

            Signature::NotNumber => !source[1].is_number(),

            Signature::Location => source[1].is_location(),

            Signature::NotLocation => !source[1].is_location(),
        };

        return success!((state, description.width));
    }

    fn update(&mut self, skip: bool, skip_end: bool, last: &mut Option<Data>, root: &Data, scope: &Data, build: &Data) -> Status<()> {
        match self.flow.last_mut().unwrap() {

            Flow::IndexIteration(iterators, saved) => {
                if !iterators.is_empty() {
                    *last = Some(iterators.remove(0));
                    self.index = *saved;
                    return success!(());
                }
            }

            Flow::For(current, end, step, saved) => {
                if current != end {
                    *current += *step;
                    *last = Some(integer!(*current));
                    self.index = *saved;
                    return success!(());
                }
            }

            Flow::While(source, initial_last, saved) => {
                *last = initial_last.clone();
                let mut source_stack = DataStack::new(source);
                let description = (*INSTRUCTIONS).get("while").unwrap();
                let extracted = confirm!(InstructionParameter::validate(&confirm!(source_stack.parameters(&last, root, &scope, build)), &description.parameters, description.variadic));
                if confirm!(DataStack::resolve_condition(&extracted, last)).0 {
                    self.index = *saved;
                    return success!(());
                }
            }

            _ => {}
        }

        match self.flow.last().cloned().unwrap() {

            Flow::Condition(state) => {
                if skip {
                    if !state {
                        confirm!(self.skip_condition(true));
                    }
                    return success!(());
                }
            }

            _ => {}
        }

        //*last = None;
        self.flow.pop().unwrap();

        if skip {
            confirm!(self.skip_condition(false));
        }

        if skip_end {
            self.advance(1);
            self.skip_parameters();
        }

        return success!(());
    }

    fn skip_condition(&mut self, stepped: bool) -> Status<()> {
        let mut level = 0;
        while let Some(instruction) = self.peek(0) {
            let instruction_name = unpack_keyword!(&instruction);
            match instruction_name.printable().as_str() {
                "if" =>  level += 1,
                "while" =>  level += 1,
                "for" =>  level += 1,
                "iterate" => level += 1,
                "else" => {
                    if stepped && level == 0 {
                        return success!(());
                    }
                }
                "end" => {
                    match level {
                        0 => return success!(()),
                        _ => level -= 1,
                    }
                }
                _ => {},
            }
            self.advance(1);
            self.skip_parameters();
        }
        return error!(UnclosedScope);
    }

    fn skip_parameters(&mut self) {
        while let Some(parameter) = self.peek(0) {
            match parameter.is_list() {
                true => self.advance(1),
                false => break,
            }
        }
    }

    fn confirm_paramters(parameters: Vec<Data>) -> Status<()> {
        match parameters.len() {
            0 => {},
            1 => ensure!(parameters[0] == keyword!("always"), string!("condition may only be #always")),
            _other => return error!(string!("unexpected parameter")), // TODO
        }
        return success!(());
    }

    pub fn break_flow(&mut self, parameters: Vec<Data>) -> Status<()> {
        confirm!(Self::confirm_paramters(parameters));
        loop {
            let top_flow = match self.flow.pop() {
                Some(flow) => flow,
                None => return error!(UnexpectedCompilerFunction, keyword!("break")),
            };
            match top_flow {
                Flow::IndexIteration(..) => break,
                Flow::While(..) => break,
                Flow::For(..) => break,
                Flow::Condition(..) => {
                    confirm!(self.skip_condition(false));
                    self.advance(1);
                    self.skip_parameters();
                },
            }
        }
        confirm!(self.skip_condition(false));
        self.advance(1);
        self.skip_parameters();
        return success!(());
    }

    pub fn continue_flow(&mut self, parameters: Vec<Data>, last: &mut Option<Data>, root: &Data, scope: &Data, build: &Data) -> Status<()> {
        confirm!(Self::confirm_paramters(parameters));
        loop {
            match self.flow.last().cloned() {
                Some(Flow::IndexIteration(..)) => break,
                Some(Flow::While(..)) => break,
                Some(Flow::For(..)) => break,
                Some(Flow::Condition(..)) => {
                    confirm!(self.skip_condition(false));
                    self.advance(1);
                    self.skip_parameters();
                    self.flow.pop().unwrap();
                },
                None => return error!(UnexpectedCompilerFunction, keyword!("continue")),
            }
        }
        return self.update(true, true, last, root, scope, build);
    }

    pub fn end(&mut self, parameters: Vec<Data>, last: &mut Option<Data>, root: &Data, scope: &Data, build: &Data) -> Status<()> {
        confirm!(Self::confirm_paramters(parameters));
        ensure!(!self.flow.is_empty(), UnexpectedCompilerFunction, keyword!("end"));
        return self.update(false, false, last, root, scope, build);
    }

    pub fn parameters(&mut self, last: &Option<Data>, root: &Data, scope: &Data, build: &Data) -> Status<SharedVector<Data>> {
        let mut parameters = SharedVector::new();

        while let Some(parameter) = self.peek(0) {
            match parameter.is_list() {
                true => self.advance(1),
                false => break,
            }

            let parameter_content = unpack_list!(&parameter);
            let mut parameter_stack = DataStack::new(&parameter_content);

            let location = expect!(parameter_stack.pop(), ExpectedLocation);
            let location_name = match &location {
                Data::Path(steps) => unpack_keyword!(&steps[0]),
                Data::Keyword(keyword) => keyword.clone(),
                _invalid => return error!(string!("not a location")),
            };

            let start = match location_name.printable().as_str() {

                "data" => {
                    let immediate = expect!(parameter_stack.pop(), ExpectedImmediate); // self.expect
                    ensure_empty!(parameter_stack, UnexpectedImmediate);
                    parameters.push(immediate);
                    continue;
                },

                "last" => {
                    match last {
                        Some(data) => data.clone(),
                        None => return error!(NoPreviousReturn),
                    }
                },

                "build" => build.clone(),

                "scope" => scope.clone(),

                "root" => root.clone(),

                _ => return error!(InvalidLocation, location), // THESE NEED CHANGE
            };

            ensure_empty!(parameter_stack, UnexpectedImmediate);
            if let Data::Path(mut steps) = location {
                steps.remove(0);

                let selector = match steps.len() {
                    1 => steps.remove(0),
                    _other => path!(steps),
                };

                match confirm!(start.index(&selector)) {
                    Some(instance) => parameters.push(instance),
                    None => return error!(MissingEntry, selector), // THESE NEED CHANGE
                }
            } else {
                parameters.push(start.clone());
            }
        }

        return success!(parameters);
    }
}
