mod time;
mod parameter;
mod signature;
mod description;
mod shell;

pub use self::parameter::InstructionParameter;
pub use self::description::INSTRUCTIONS;
pub use self::time::initialize_time;

use internal::*;
use debug::*;
#[cfg(feature = "tokenize")]
use tokenize::call_tokenize;
#[cfg(feature = "parse")]
use parse::call_parse;
#[cfg(feature = "build")]
use build::call_build;

use self::time::*;
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

macro_rules! reduce_positions {
    ($parameters:expr, $function:ident) => ({
        let mut iterator = $parameters.iter();
        let mut first = Vec::new();

        for positions in unpack_list!(iterator.next().unwrap()).iter() {
            first.push(confirm!(Position::deserialize(positions)));
        }

        while let Some(next) = iterator.next() {
            for positions in unpack_list!(next).iter() {
                first.push(confirm!(Position::deserialize(positions)));
            }
        }

        list!(Position::$function(first, false).iter().map(|position| position.serialize()).collect())
    });
}

macro_rules! combine_data {
    ($parameters:expr, $variant:ident, $name:expr) => ({
        let value: SharedString = $parameters.iter().map(|item| item.to_string()).collect();
        ensure!(!value.is_empty(), string!("{} may not be empty", $name));
        ensure!(!value.first().unwrap().is_digit(), string!("{} may not start with a digit", $name));
        ensure!(CharacterStack::new(SharedString::from(""), None).is_pure(&value), string!("{} may only contain non breaking characters", $name));
        Data::$variant(value)
    });
}

pub fn instruction(name: &SharedString, raw_parameters: Option<SharedVector<Data>>, stack: &mut DataStack, last: &mut Option<Data>, pass: &Option<Pass>, root: &Data, scope: &Data, build: &Data) -> Status<bool> {
    let internal_name = name.printable();
    let description = match (*INSTRUCTIONS).get(internal_name.as_str()) {
        Some(description) => description,
        None => return error!(InvalidCompilerFunction, keyword!(String, name.clone())),
    };

    if !description.invokable && raw_parameters.is_some() {
        return error!(string!("instruction may not be invoked"));
    }

    if description.conditional {
        match &description.signature {

            Signature::While => confirm!(stack.looped_condition(last, root, scope, build)),

            Signature::Else => confirm!(stack.dependent_condition(last, root, scope, build)),

            _invalid => panic!(),
        }
    } else {
        let mut parameters = match raw_parameters {
            Some(raw_parameters) => confirm!(InstructionParameter::validate(&raw_parameters, &description.parameters, description.variadic)),
            None => confirm!(InstructionParameter::validate(&confirm!(stack.parameters(&last, root, scope, build)), &description.parameters, description.variadic)),
        };

        match &description.signature {

            Signature::Shell => confirm!(shell(last, pass, root, scope, build)),

            Signature::Return => {
                ensure!(parameters.len() < 2, string!("return expected 0 or 1 parameter; got {}", parameters.len()));
                *last = parameters.pop();
                return success!(true);
            },

            Signature::Remember => *last = Some(parameters.remove(0)),

            Signature::Fuze => *last = Some(reduce_positions!(parameters, fuze)),

            Signature::Range => *last = Some(reduce_positions!(parameters, range)),

            Signature::FillBack => {
                let mut source = parameters[0].to_string();
                let filler = unpack_literal!(&parameters[1]);
                let length = unpack_number!(&parameters[2]) as usize;

                if source.len() >= length {
                    *last = Some(string!(String, source));
                } else {
                    while source.len() < length {
                        source.push_str(&filler);
                    }
                    *last = Some(string!(String, source));
                }
            }

            Signature::Fill => {
                let mut source = parameters[0].to_string();
                let filler = unpack_literal!(&parameters[1]);
                let length = unpack_number!(&parameters[2]) as usize;

                if source.len() >= length {
                    *last = Some(string!(String, source));
                } else {
                    while source.len() < length {
                        source.insert_str(0, &filler);
                    }
                    *last = Some(string!(String, source));
                }
            }

            Signature::Random => {
                let mut generator = rand::thread_rng();
                let smallest = unpack_number!(&parameters[0]) as i64;
                let biggest = unpack_number!(&parameters[1]) as i64;
                ensure!(smallest <= biggest, string!("first parameter must be smaller or equal to the second one"));
                *last = Some(integer!(generator.gen_range(smallest..=biggest)));
            }

            Signature::Time => {
                let start_time = *START_TIME;
                let now = SystemTime::now();
                let elapsed = now.duration_since(start_time).expect("time went backwards");
                *last = Some(integer!(elapsed.as_millis() as i64));
            }

            Signature::Input => {
                use std::io::{ Write, stdout };

                if !parameters.is_empty() {
                    for parameter in parameters {
                        print!("{}", parameter.to_string());
                    }
                    stdout().flush().ok().expect("failed to flush stdout");
                }

                let mut line = String::new();
                match std::io::stdin().read_line(&mut line) {
                    Ok(_bytes) => line.remove(line.len() - 1),
                    Err(_error) => return error!(string!("failed to read stdin")), // TODO:
                };
                *last = Some(string!(&line));
            }

            Signature::Error => {
                let mut string = SharedString::new();
                for parameter in parameters.iter() {
                    string.push_str(&parameter.to_string());
                }
                *last = None;
                return error!(string!(String, string));
            }

            Signature::Ensure => {
                let (state, length) = confirm!(DataStack::resolve_condition(&parameters, last));
                ensure!(parameters.len() >= length, string!("ensure expectes an error message"));
                if !state {
                    let mut string = SharedString::new();
                    for parameter in &parameters[length..] {
                        string.push_str(&parameter.to_string());
                    }
                    return error!(string!(String, string));
                }
            }

            Signature::PrintLine => {
                for parameter in parameters {
                    print!("{}", parameter.to_string());
                }
                println!();
            }

            Signature::Print => {
                use std::io::{ Write, stdout };
                for parameter in parameters {
                    print!("{}", parameter.to_string());
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

            Signature::String => *last = Some(string!(String, parameters.iter().map(|item| item.to_string()).collect())),

            Signature::Join => {
                let list = unpack_list!(&parameters[0]);
                let seperator = unpack_literal!(&parameters[1]);
                let mut string = SharedString::new();
                for (index, item) in list.iter().enumerate() {
                    string.push_str(&item.to_string());
                    if index != list.len() - 1 {
                        string.push_str(&seperator);
                    }
                }
                *last = Some(string!(String, string));
            }

            Signature::Uppercase => *last = Some(string!(String, parameters.iter().map(|item| item.to_string().uppercase()).collect())),

            Signature::Lowercase => *last = Some(string!(String, parameters.iter().map(|item| item.to_string().lowercase()).collect())),

            Signature::Identifier => *last = Some(combine_data!(parameters, Identifier, "identifier")),

            Signature::Keyword => *last = Some(combine_data!(parameters, Keyword, "keyword")),

            Signature::Type => *last = Some(keyword!(String, parameters[0].data_type())),

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

                let output = command.output().expect("failed to execute process");
                let mut return_map = DataMap::new();
                return_map.insert(identifier!("output"), string!(&String::from_utf8_lossy(&output.stdout)));
                return_map.insert(identifier!("success"), boolean!(output.status.success()));
                *last = Some(map!(return_map));
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

                                "build" => confirm!(build.modify(None, value.clone())),

                                "function" => panic!("implement me correctly"), // TODO:

                                "template" => panic!("implement me correctly"), // TODO:

                                other => return error!(string!("invalid scope for modify {}", other)),
                            }
                        },
                        Data::Path(steps) => {
                            match extract_keyword!(&steps[0]).printable().as_str() {

                                "root" => confirm!(root.modify(Some(&path!(steps.iter().skip(1).cloned().collect())), value.clone())),

                                "scope" => confirm!(scope.modify(Some(&path!(steps.iter().skip(1).cloned().collect())), value.clone())),

                                "build" => confirm!(build.modify(Some(&path!(steps.iter().skip(1).cloned().collect())), value.clone())),

                                "function" => panic!("implement me correctly"), // TODO:

                                "template" => panic!("implement me correctly"), // TODO:

                                other => return error!(string!("invalid scope for modify {}", other)),
                            }
                        },
                        _other => return error!(string!("only key or path are valid")),
                    }

                    index += 2;
                }
                *last = None;
            }

            Signature::Serialize => *last = Some(string!(String, parameters[0].serialize())),

            Signature::Deserialize => {
                let source = unpack_string!(&parameters[0]);
                let mut character_stack = CharacterStack::new(source, None);
                *last = Some(confirm!(parse_data(&mut character_stack)));
            }

            Signature::Length => *last = Some(integer!(confirm!(parameters[0].length()) as i64)),

            Signature::Call => {
                let call_function = parameters.remove(0);
                let parameters = parameters.into_iter().collect();
                *last = confirm!(function(&call_function, parameters, pass, root, build));
            },

            Signature::CallList => {
                let passed_parameters = match parameters.len() { // TODO: combine these
                    1 => SharedVector::new(),
                    2 => unpack_list!(&parameters[1]),
                    _ => return error!(UnexpectedParameter, parameters[2].clone()),
                };
                *last = confirm!(function(&parameters[0], passed_parameters, pass, root, build));
            },

            Signature::Invoke => {
                let passed_parameters = match parameters.len() { // TODO: combine these
                    1 => SharedVector::new(),
                    2 => unpack_list!(&parameters[1]),
                    _ => return error!(UnexpectedParameter, parameters[2].clone()),
                };
                let instruction_name = unpack_keyword!(&parameters[0]);

                if confirm!(instruction(&instruction_name, Some(passed_parameters), stack, last, pass, root, scope, build)) {
                    return success!(true);
                }
            },

            Signature::Resolve => {
                match &parameters[0] {

                    Data::Keyword(index) => {
                        match index.printable().as_str() {

                            "root" => *last = Some(root.clone()),

                            "scope" => *last = Some(scope.clone()),

                            "build" => *last = Some(build.clone()),

                            "function" => {
                                let function_map = confirm!(root.index(&keyword!("function")));
                                *last = Some(expect!(function_map, string!("missing field function")));
                            },

                            "template" => {
                                let template_map = confirm!(root.index(&keyword!("template")));
                                *last = Some(expect!(template_map, string!("missing field template")));
                            },

                            other => return error!(string!("invalid scope for resolve {}", other)),
                        }
                    },

                    Data::Path(steps) => {
                        match extract_keyword!(&steps[0]).printable().as_str() {

                            "root" => *last = Some(expect!(confirm!(root.index(&path!(steps.iter().skip(1).cloned().collect()))), string!("failed to resolve"))),

                            "scope" => *last = Some(expect!(confirm!(scope.index(&path!(steps.iter().skip(1).cloned().collect()))), string!("failed to resolve"))),

                            "build" => *last = Some(expect!(confirm!(build.index(&path!(steps.iter().skip(1).cloned().collect()))), string!("failed to resolve"))),

                            "function" => {
                                let function_map = confirm!(root.index(&keyword!("function")));
                                let function_map = expect!(function_map, string!("missing field function"));
                                *last = Some(expect!(confirm!(function_map.index(&path!(steps.iter().skip(1).cloned().collect()))), string!("failed to resolve")));
                            },

                            "template" => {
                                let template_map = confirm!(root.index(&keyword!("template")));
                                let template_map = expect!(template_map, string!("missing field template"));
                                *last = Some(expect!(confirm!(template_map.index(&path!(steps.iter().skip(1).cloned().collect()))), string!("failed to resolve")));
                            },

                            other => return error!(string!("invalid scope for resolve {}", other)),
                        }
                    },

                    _other => return error!(string!("only key or path are valid")),
                }
            }

            Signature::Pass => {
                ensure!(pass.is_some(), string!("pass can only be called during a pass, try running new_pass instead"));
                let instance = parameters.remove(0);
                let mut new_pass = pass.clone().unwrap();
                new_pass.parameters = parameters;
                *last = Some(confirm!(instance.pass(&new_pass, root, build)));
            }

            Signature::NewPass => {
                let pass_name = parameters.remove(0);
                let instance = parameters.remove(0);
                let new_pass = Pass::new(pass_name, parameters);
                *last = Some(confirm!(instance.pass(&new_pass, root, build)));
            }

            Signature::Map => {
                let mut iterator = parameters.iter();
                let mut index = 2;
                let mut data_map = DataMap::new();
                while let Some(key) = iterator.next() {
                    let value = expect!(iterator.next(), ExpectedParameter, integer!(index), expected_list!["instance"]);
                    if let Some(_previous) = data_map.insert(key.clone(), value.clone()) {
                        return error!(string!("map may only have each field once")); // TODO: BETTER TEXT + WHAT FIELD + WHAT INDEX
                    }
                    index += 2;
                }
                *last = Some(map!(data_map));
            }

            Signature::Path => {
                let mut steps = SharedVector::new();
                for parameter in parameters {
                    if parameter.is_path() {
                        unpack_path!(&parameter).iter().for_each(|step| steps.push(step.clone()));
                    } else {
                        ensure!(parameter.is_selector(), string!("path may only contain selectors")); // TODO:
                        steps.push(parameter);
                    }
                }
                ensure!(steps.len() >= 2, InvalidPathLength, list!(steps));
                *last = Some(path!(steps));
            }

            Signature::List => *last = Some(list!(parameters.into_iter().collect())),

            Signature::ReadFile => *last = Some(string!(String, confirm!(read_file(&unpack_string!(&parameters[0]))))),

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
                let item = expect!(item, string!("missing entry {}", parameters[1].serialize()));
                let new_container = confirm!(parameters[0].remove(&parameters[1]));
                *last = Some(confirm!(new_container.insert(&parameters[2], item)));
            }

            Signature::Push => {
                let mut combined = parameters.remove(0);
                while !parameters.is_empty() {
                    combined = confirm!(combined.insert(&integer!(1), parameters.remove(0)));
                }
                *last = Some(combined);
            }

            Signature::Append => {
                let mut combined = parameters.remove(0);
                while !parameters.is_empty() {
                    combined = confirm!(combined.insert(&integer!(-1), parameters.remove(0)));
                }
                *last = Some(combined);
            }

            Signature::Remove => *last = Some(confirm!(parameters[0].remove(&parameters[1]))),

            Signature::Index => {
                match confirm!(parameters[0].index(&parameters[1])) {
                    Some(entry) => *last = Some(entry),
                    None => return error!(string!("missing entry {}", parameters[1].serialize())),
                };
            }

            Signature::Pairs => {
                let mut pairs = SharedVector::new();
                for (selector, instance) in confirm!(parameters[0].pairs()).into_iter() {
                    let mut map = DataMap::new();
                    map.insert(identifier!("selector"), selector);
                    map.insert(identifier!("instance"), instance);
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

            Signature::For => confirm!(stack.counted(unpack_integer!(&parameters[0]), unpack_integer!(&parameters[1]), 1, last, root, scope, build)),

            Signature::Iterate => confirm!(stack.iterate(parameters, last, root, scope, build)),

            Signature::If => confirm!(stack.condition(parameters, last, root, scope, build)),

            Signature::Break => confirm!(stack.break_flow(parameters)),

            Signature::Continue => confirm!(stack.continue_flow(parameters, last, root, scope, build)),

            Signature::End => confirm!(stack.end(parameters, last, root, scope, build)),

            #[cfg(feature = "tokenize")]
            Signature::Tokenize => *last = Some(confirm!(call_tokenize(&parameters[0], &parameters[1], &parameters[2], &parameters[3], root, build))),

            #[cfg(feature = "parse")]
            Signature::Parse => *last = Some(confirm!(call_parse(&parameters[0], &parameters[1], &parameters[2]))),
            
            #[cfg(feature = "build")]
            Signature::Build => *last = Some(confirm!(call_build(&parameters[0], &parameters[1]))),

            _invalid => panic!(),
        }
    }

    return success!(false);
}
