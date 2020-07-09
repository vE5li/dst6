mod time;
mod parameter;
mod signature;
mod description;
mod compile;
mod shell;

pub use self::parameter::InstructionParameter;
pub use self::description::INSTRUCTIONS;
pub use self::time::initialize_time;

use internal::*;
use self::time::*;
use self::compile::*;
use self::shell::shell;
use self::signature::Signature;
use std::process::{ Command, Stdio };
use rand::Rng;

macro_rules! reduce_list {
    ($parameters:expr, $function:ident) => ({
        let mut iterator = $parameters.iter();
        let mut result = iterator.next().unwrap().clone();
        while let Some(instance) = iterator.next() {
            result = confirm!(result.$function(instance));
        }
        Some(result)
    });
}

macro_rules! last_return {
    ($value:expr, $last:expr) => ({
        *$last = $value;
        return success!(true);
    });
}

macro_rules! reduce_positions {
    ($parameters:expr, $function:ident) => ({
        let mut iterator = $parameters.iter();
        let mut first = confirm!(Position::parse_positions(iterator.next().unwrap()));
        while let Some(next) = iterator.next() {
            first.append(&mut confirm!(Position::parse_positions(next)));
        }
        Position::serialize_positions(&Position::$function(first, false))
    });
}

macro_rules! combine_data {
    ($parameters:expr, $variant:ident, $name:expr) => ({
        let value: VectorString = $parameters.iter().map(|item| item.to_string()).collect();
        ensure!(!value.is_empty(), Message, string!(str, "{} may not be empty", $name));
        ensure!(!value.first().unwrap().is_digit(), Message, string!(str, "{} may not start with a digit", $name));
        ensure!(CharacterStack::new(VectorString::from(""), None).is_pure(&value), Message, string!(str, "{} may only contain non breaking characters", $name));
        Data::$variant(value)
    });
}

pub fn instruction(name: &VectorString, raw_parameters: Option<Vector<Data>>, stack: &mut DataStack, last: &mut Option<Data>, current_pass: &Option<VectorString>, root: &Data, scope: &Data, build: &Data, context: &Data) -> Status<bool> {

    let internal_name = name.printable();
    let description = match (*INSTRUCTIONS).get(internal_name.as_str()) {
        Some(description) => description,
        None => return error!(InvalidCompilerFunction, keyword!(name.clone())),
    };

    if !description.invokable && raw_parameters.is_some() {
        return error!(Message, string!(str, "instruction may not be invoked"));
    }

    if description.conditional {
        match &description.signature {

            Signature::While => confirm!(stack.looped_condition(last, root, scope, build, context)),

            Signature::Else => confirm!(stack.dependent_condition(last, root, scope, build, context)),

            _invalid => panic!(),
        }
    } else {
        let mut parameters = match raw_parameters {
            Some(raw_parameters) => confirm!(InstructionParameter::validate(&raw_parameters, &description.parameters, description.variadic)),
            None => confirm!(InstructionParameter::validate(&confirm!(stack.parameters(&last, root, scope, build, context)), &description.parameters, description.variadic)),
        };

        match &description.signature {

            Signature::Shell => confirm!(shell(last, current_pass, root, scope, build, context)),

            Signature::Return => last_return!(Some(parameters.remove(0)), last),

            Signature::Terminate => last_return!(None, last),

            Signature::Remember => *last = Some(parameters.remove(0)),

            Signature::Fuze => *last = Some(reduce_positions!(parameters, fuze)),

            Signature::Range => *last = Some(reduce_positions!(parameters, range)),

            Signature::FillBack => {
                let mut source = parameters[0].to_string();
                let filler = unpack_literal!(&parameters[1]);
                let length = unpack_number!(&parameters[2]) as usize;

                if source.len() >= length {
                    *last = Some(string!(source));
                } else {
                    while source.len() < length {
                        source.push_str(&filler);
                    }
                    *last = Some(string!(source));
                }
            }

            Signature::Fill => {
                let mut source = parameters[0].to_string();
                let filler = unpack_literal!(&parameters[1]);
                let length = unpack_number!(&parameters[2]) as usize;

                if source.len() >= length {
                    *last = Some(string!(source));
                } else {
                    while source.len() < length {
                        source.insert_str(0, &filler);
                    }
                    *last = Some(string!(source));
                }
            }

            Signature::Random => {
                let mut generator = rand::thread_rng();
                let smallest = unpack_number!(&parameters[0]) as i64;
                let biggest = unpack_number!(&parameters[1]) as i64;
                ensure!(smallest <= biggest, Message, string!(str, "first parameter must be smaller or equal to the second one"));
                *last = Some(integer!(generator.gen_range(smallest, biggest + 1)));
            }

            Signature::Time => {
                let start_time = *START_TIME;
                let now = SystemTime::now();
                let elapsed = now.duration_since(start_time).expect("time went backwards");
                *last = Some(integer!(elapsed.as_millis() as i64));
            }

            Signature::Input => {
                let mut line = String::new();
                match std::io::stdin().read_line(&mut line) {
                    Ok(_bytes) => line.remove(line.len() - 1),
                    Err(_error) => return error!(Message, string!(str, "failed to read stdin")), // TODO:
                };
                *last = Some(string!(str, &line));
            }

            Signature::Error => {
                let mut string = VectorString::new();
                for parameter in parameters.iter() {
                    string.push_str(&parameter.to_string());
                }
                *last = None;
                return error!(Message, string!(string));
            }

            Signature::Ensure => {
                let (state, length) = confirm!(DataStack::resolve_condition(&parameters, last));
                ensure!(parameters.len() >= length, Message, string!(str, "ensure expectes an error message"));
                if !state {
                    let mut string = VectorString::new();
                    for parameter in &parameters[length..] {
                        string.push_str(&parameter.to_string());
                    }
                    return error!(Message, string!(string));
                }
            }

            Signature::PrintLine => {
                for parameter in parameters {
                    print!("{}", parameter.to_string().printable());
                }
                println!();
            }

            Signature::Print => {
                use std::io::{ Write, stdout };
                for parameter in parameters {
                    print!("{}", parameter.to_string().printable());
                }
                stdout().flush().ok().expect("failed to flush stdout");
            }

            Signature::Absolute => *last = Some(confirm!(parameters[0].absolute())),

            Signature::Negate => *last = Some(confirm!(parameters[0].negate())),

            Signature::Flip => *last = Some(confirm!(parameters[0].flip())),

            Signature::Not => *last = Some(confirm!(parameters[0].not())),

            Signature::Empty => *last = Some(confirm!(parameters[0].empty())),

            Signature::ShiftLeft => *last = Some(confirm!(parameters[0].shift_left(&parameters[1]))),

            Signature::ShiftRight => *last = Some(confirm!(parameters[0].shift_right(&parameters[1]))),

            Signature::And => *last = reduce_list!(parameters, and),

            Signature::Or => *last = reduce_list!(parameters, or),

            Signature::Xor => *last = reduce_list!(parameters, xor),

            Signature::Add => *last = reduce_list!(parameters, add),

            Signature::Subtract => *last = reduce_list!(parameters, subtract),

            Signature::Multiply => *last = reduce_list!(parameters, multiply),

            Signature::Divide => *last = reduce_list!(parameters, divide),

            Signature::Modulo => *last = Some(confirm!(parameters[0].modulo(&parameters[1]))),

            Signature::Power => *last = Some(confirm!(parameters[0].power(&parameters[1]))),

            Signature::Logarithm => *last = Some(confirm!(parameters[0].logarithm(&parameters[1]))),

            Signature::Ceiling => *last = Some(confirm!(parameters[0].ceiling())),

            Signature::Floor => *last = Some(confirm!(parameters[0].floor())),

            Signature::SquareRoot => *last = Some(confirm!(parameters[0].square_root())),

            Signature::Sine => *last = Some(confirm!(parameters[0].sine())),

            Signature::Cosine => *last = Some(confirm!(parameters[0].cosine())),

            Signature::Tangent => *last = Some(confirm!(parameters[0].tangent())),

            Signature::Round => *last = Some(confirm!(parameters[0].round())),

            Signature::Integer => *last = Some(confirm!(parameters[0].integer())),

            Signature::Float => *last = Some(confirm!(parameters[0].float())),

            Signature::Character => *last = Some(confirm!(parameters[0].character())),

            Signature::String => *last = Some(string!(parameters.iter().map(|item| item.to_string()).collect())),

            Signature::Join => {
                let list = unpack_list!(&parameters[0]);
                let seperator = unpack_literal!(&parameters[1]);
                let mut string = VectorString::new();
                for (index, item) in list.iter().enumerate() {
                    string.push_str(&item.to_string());
                    if index != list.len() - 1 {
                        string.push_str(&seperator);
                    }
                }
                *last = Some(string!(string));
            }

            Signature::Uppercase => *last = Some(string!(parameters.iter().map(|item| item.to_string().uppercase()).collect())),

            Signature::Lowercase => *last = Some(string!(parameters.iter().map(|item| item.to_string().lowercase()).collect())),

            Signature::Identifier => *last = Some(combine_data!(parameters, Identifier, "identifier")),

            Signature::Keyword => *last = Some(combine_data!(parameters, Keyword, "keyword")),

            Signature::Type => *last = Some(keyword!(parameters[0].data_type())),

            Signature::Insert => *last = Some(confirm!(parameters[0].insert(&parameters[1], parameters[2].clone()))),

            Signature::Overwrite => *last = Some(confirm!(parameters[0].overwrite(&parameters[1], parameters[2].clone()))),

            Signature::Replace => *last = Some(confirm!(parameters[0].replace(&parameters[1], &parameters[2]))),

            Signature::System => {
                let mut iterator = parameters.iter();
                let command = unpack_string!(iterator.next().unwrap());
                let mut command = Command::new(&command.serialize());
                while let Some(argument) = iterator.next() {
                    command.arg(&unpack_string!(argument).serialize());
                }

                *last = Some(boolean!(command.status().expect("failed to execute process").success())); // RETURN NONE INSTEAD OF PANICING
            }

            Signature::Silent => {
                let mut iterator = parameters.iter();
                let command = unpack_string!(iterator.next().unwrap());
                let mut command = Command::new(&command.serialize());
                while let Some(argument) = iterator.next() {
                    command.arg(&unpack_string!(argument).printable());
                }

                *last = Some(boolean!(command.stdout(Stdio::null()).status().expect("failed to execute process").success())); // RETURN NONE INSTEAD OF PANICING
            }

            Signature::Modify => {
                let mut iterator = parameters.iter();
                let mut index = 0;
                while let Some(key) = iterator.next() {
                    let value = expect!(iterator.next(), ExpectedParameter, integer!(index + 2), expected_list!["instance"]);

                    match key {
                        Data::Keyword(index) => {
                            match index.printable().as_str() {

                                "root" => confirm!(root.modify(None, value.clone())),

                                "scope" => confirm!(scope.modify(None, value.clone())),

                                "context" => confirm!(context.modify(None, value.clone())),

                                "build" => confirm!(build.modify(None, value.clone())),

                                "function" => confirm!(root.modify(Some(key), value.clone())),

                                "template" => confirm!(root.modify(Some(key), value.clone())),

                                other => return error!(Message, string!(str, "invalid scope for modify {}", other)),
                            }
                        },
                        Data::Path(steps) => {
                            match extract_keyword!(&steps[0]).printable().as_str() {

                                "root" => confirm!(root.modify(Some(&path!(steps.iter().skip(1).cloned().collect())), value.clone())),

                                "scope" => confirm!(scope.modify(Some(&path!(steps.iter().skip(1).cloned().collect())), value.clone())),

                                "context" => confirm!(context.modify(Some(&path!(steps.iter().skip(1).cloned().collect())), value.clone())),

                                "build" => confirm!(build.modify(Some(&path!(steps.iter().skip(1).cloned().collect())), value.clone())),

                                "function" => confirm!(root.modify(Some(key), value.clone())),

                                "template" => confirm!(root.modify(Some(key), value.clone())),

                                other => return error!(Message, string!(str, "invalid scope for modify {}", other)),
                            }
                        },
                        _other => return error!(Message, string!(str, "only key or path are valid")),
                    }

                    index += 2;
                }
                *last = None;
            }

            Signature::Serialize => *last = Some(string!(parameters[0].serialize())),

            Signature::Deserialize => {
                let source = unpack_string!(&parameters[0]);
                let mut character_stack = CharacterStack::new(source, None);
                *last = Some(confirm!(parse_data(&mut character_stack)));
            }

            Signature::Length => *last = Some(integer!(confirm!(parameters[0].length()) as i64)),

            Signature::CompileFile => *last = Some(confirm!(compile_file(parameters, context))),

            Signature::CompileString => *last = Some(confirm!(compile_string(parameters, context))),

            Signature::CompileModule => *last = Some(confirm!(compile_module(parameters, context))),

            Signature::Call => {
                let call_function = parameters.remove(0);
                let parameters = parameters.into_iter().collect();
                *last = confirm!(function(&call_function, parameters, current_pass, root, build, context));
            },

            Signature::CallList => {
                let passed_parameters = match parameters.len() { // TODO: combine these
                    1 => Vector::new(),
                    2 => unpack_list!(&parameters[1]),
                    _ => return error!(UnexpectedParameter, parameters[2].clone()),
                };
                *last = confirm!(function(&parameters[0], passed_parameters, current_pass, root, build, context));
            },

            Signature::Invoke => {
                let passed_parameters = match parameters.len() { // TODO: combine these
                    1 => Vector::new(),
                    2 => unpack_list!(&parameters[1]),
                    _ => return error!(UnexpectedParameter, parameters[2].clone()),
                };
                let instruction_name = unpack_keyword!(&parameters[0]);

                if confirm!(instruction(&instruction_name, Some(passed_parameters), stack, last, current_pass, root, scope, build, context)) {
                    return success!(true);
                }
            },

            Signature::Resolve => {
                match &parameters[0] {

                    Data::Keyword(index) => {
                        match index.printable().as_str() {

                            "root" => *last = Some(root.clone()),

                            "scope" => *last = Some(scope.clone()),

                            "context" => *last = Some(context.clone()),

                            "build" => *last = Some(build.clone()),

                            "function" => {
                                let function_map = confirm!(root.index(&keyword!(str, "function")));
                                *last = Some(expect!(function_map, Message, string!(str, "missing field function")));
                            },

                            "template" => {
                                let template_map = confirm!(root.index(&keyword!(str, "template")));
                                *last = Some(expect!(template_map, Message, string!(str, "missing field template")));
                            },

                            other => return error!(Message, string!(str, "invalid scope for resolve {}", other)),
                        }
                    },

                    Data::Path(steps) => {
                        match extract_keyword!(&steps[0]).printable().as_str() {

                            "root" => *last = Some(expect!(confirm!(root.index(&path!(steps.iter().skip(1).cloned().collect()))), Message, string!(str, "failed to resolve"))),

                            "scope" => *last = Some(expect!(confirm!(scope.index(&path!(steps.iter().skip(1).cloned().collect()))), Message, string!(str, "failed to resolve"))),

                            "context" => *last = Some(expect!(confirm!(context.index(&path!(steps.iter().skip(1).cloned().collect()))), Message, string!(str, "failed to resolve"))),

                            "build" => *last = Some(expect!(confirm!(build.index(&path!(steps.iter().skip(1).cloned().collect()))), Message, string!(str, "failed to resolve"))),

                            "function" => {
                                let function_map = confirm!(root.index(&keyword!(str, "function")));
                                let function_map = expect!(function_map, Message, string!(str, "missing field function"));
                                *last = Some(expect!(confirm!(function_map.index(&path!(steps.iter().skip(1).cloned().collect()))), Message, string!(str, "failed to resolve")));
                            },

                            "template" => {
                                let template_map = confirm!(root.index(&keyword!(str, "template")));
                                let template_map = expect!(template_map, Message, string!(str, "missing field template"));
                                *last = Some(expect!(confirm!(template_map.index(&path!(steps.iter().skip(1).cloned().collect()))), Message, string!(str, "failed to resolve")));
                            },

                            other => return error!(Message, string!(str, "invalid scope for resolve {}", other)),
                        }
                    },

                    _other => return error!(Message, string!(str, "only key or path are valid")),
                }
            }

            Signature::Pass => {
                let instance = parameters.remove(0);
                let parameters = list!(parameters.into_iter().collect());
                let mut pass_context = context.clone();
                confirm!(pass_context.set_entry(&keyword!(str, "parameters"), parameters, true));
                *last = Some(confirm!(instance.pass(current_pass, root, build, &pass_context)));
            }

            Signature::NewPass => {
                let new_pass = parameters.remove(0);
                let new_pass = unpack_identifier!(&new_pass);
                let instance = parameters.remove(0);
                let parameters = list!(parameters.into_iter().collect());
                let mut pass_context = context.clone();
                confirm!(pass_context.set_entry(&keyword!(str, "parameters"), parameters, true));
                *last = Some(confirm!(instance.pass(&Some(new_pass), root, build, &pass_context)));
            }

            Signature::Map => {
                let mut iterator = parameters.iter();
                let mut index = 2;
                let mut data_map = DataMap::new();
                while let Some(key) = iterator.next() {
                    let value = expect!(iterator.next(), ExpectedParameter, integer!(index), expected_list!["instance"]);
                    if let Some(_previous) = data_map.insert(key.clone(), value.clone()) {
                        return error!(Message, string!(str, "map may only have each field once")); // TODO: BETTER TEXT + WHAT FIELD + WHAT INDEX
                    }
                    index += 2;
                }
                *last = Some(map!(data_map));
            }

            Signature::Path => {
                let mut steps = Vector::new();
                for parameter in parameters {
                    if parameter.is_path() {
                        unpack_path!(&parameter).iter().for_each(|step| steps.push(step.clone()));
                    } else {
                        ensure!(parameter.is_selector(), Message, string!(str, "path may only contain selectors")); // TODO:
                        steps.push(parameter);
                    }
                }
                ensure!(steps.len() >= 2, InvalidPathLength, list!(steps));
                *last = Some(path!(steps));
            }

            Signature::List => *last = Some(list!(parameters.into_iter().collect())),

            Signature::ReadFile => *last = Some(string!(confirm!(read_file(&unpack_string!(&parameters[0]))))),

            Signature::WriteFile => {
                let filename = unpack_string!(&parameters[0]);
                let content = unpack_string!(&parameters[1]);
                confirm!(write_file(&filename, &content));
                *last = None;
            }

            Signature::ReadMap => *last = Some(confirm!(read_map(&unpack_string!(&parameters[0])))),

            Signature::WriteMap => {
                let filename = unpack_string!(&parameters[0]);
                confirm!(write_map(&filename, &parameters[1]));
                *last = None;
            }

            Signature::ReadList => *last = Some(confirm!(read_list(&unpack_string!(&parameters[0])))),

            Signature::WriteList => {
                let filename = unpack_string!(&parameters[0]);
                confirm!(write_list(&filename, &parameters[1]));
                *last = None;
            }

            Signature::Merge => {
                let mut merged = parameters.remove(0);
                for parameter in &parameters {
                    merged = confirm!(merged.merge(parameter));
                }
                *last = Some(merged);
            }

            Signature::Move => {
                let item = confirm!(parameters[0].index(&parameters[1]));
                let item = expect!(item, Message, string!(str, "missing entry {}", parameters[1].serialize()));
                let new_container = confirm!(parameters[0].remove(&parameters[1]));
                *last = Some(confirm!(new_container.insert(&parameters[2], item)));
            }

            Signature::Push => *last = Some(confirm!(parameters[0].insert(&integer!(1), parameters[1].clone()))),

            Signature::Append => *last = Some(confirm!(parameters[0].insert(&integer!(-1), parameters[1].clone()))),

            Signature::Remove => *last = Some(confirm!(parameters[0].remove(&parameters[1]))),

            Signature::Index => {
                match confirm!(parameters[0].index(&parameters[1])) {
                    Some(entry) => *last = Some(entry),
                    None => return error!(Message, string!(str, "missing entry {}", parameters[1].serialize())),
                };
            }

            Signature::Pairs => {
                let mut pairs = Vector::new();
                for (selector, instance) in confirm!(parameters[0].pairs()).into_iter() {
                    let mut map = DataMap::new();
                    map.insert(identifier!(str, "selector"), selector);
                    map.insert(identifier!(str, "instance"), instance);
                    pairs.push(map!(map));
                }
                *last = Some(list!(pairs));
            }

            Signature::Keys => *last = Some(confirm!(parameters[0].keys())),

            Signature::Values => *last = Some(confirm!(parameters[0].values())),

            Signature::Position => *last = Some(confirm!(parameters[0].position(&parameters[1]))),

            Signature::Split => *last = Some(confirm!(parameters[0].split(&parameters[1], &parameters[2]))),

            Signature::Slice => *last = Some(confirm!(parameters[0].slice(&parameters[1], &parameters[2]))),

            Signature::Boolean => {
                let (state, length) = confirm!(DataStack::resolve_condition(&parameters.iter().cloned().collect(), last));
                ensure!(length == parameters.len(), UnexpectedParameter, parameters[length].clone());
                *last = Some(boolean!(state));
            },

            Signature::For => confirm!(stack.counted(unpack_integer!(&parameters[0]), unpack_integer!(&parameters[1]), 1, last, root, scope, build, context)),

            Signature::Iterate => confirm!(stack.iterate(parameters, last, root, scope, build, context)),

            Signature::If => confirm!(stack.condition(parameters, last, root, scope, build, context)),

            Signature::Break => confirm!(stack.break_flow(parameters)),

            Signature::Continue => confirm!(stack.continue_flow(parameters, last, root, scope, build, context)),

            Signature::End => confirm!(stack.end(parameters, last, root, scope, build, context)),

            Signature::Tokenize => *last = Some(confirm!(tokenize_string(&parameters[0], &parameters[1], context))),

            Signature::Parse => *last = Some(confirm!(parse_token_stream(&parameters[0], &parameters[1], context))),

            Signature::Build => *last = Some(confirm!(build_top(&parameters[0], &parameters[1], context))),

            _invalid => panic!(),
        }
    }

    return success!(false);
}
