use super::super::ParameterType;

use internal::*;
use debug::*;

#[derive(Debug, Clone)]
pub struct FunctionParameter {
    pub key:           Option<Data>,
    pub type_filter:   Option<Vec<ParameterType>>,
    pub variadic:      bool,
}

impl FunctionParameter {

    pub fn new(appearance: &Data) -> Status<Self> {
        let parameter_list = unpack_list!(appearance);
        let mut parameter_stack = DataStack::new(&parameter_list);

        let parameter_type = expect!(parameter_stack.pop(), Message, string!("expected parameter type"));
        let variadic = match unpack_keyword!(&parameter_type).printable().as_str() {
            "single" => false,
            "list" => true,
            invalid => panic!("invalid parameter type {}", invalid),
        };

        let mut key = None;
        if let Some(next) = parameter_stack.peek(0) {
            if next.is_key() {
                parameter_stack.advance(1);
                key = Some(next);
            }
        }

        let mut type_filter = None;
        if let Some(next) = parameter_stack.pop() {
            let mut filters = Vec::new();
            for filter in unpack_list!(&next).into_iter() {
                let source = unpack_identifier!(&filter);
                filters.push(confirm!(ParameterType::from(&source.serialize())));
            }
            type_filter = Some(filters);
        }

        // assert parameter stack is empty

        success!(Self {
            key:            key,
            type_filter:    type_filter,
            variadic:       variadic,
        })
    }

    pub fn validate(scope: &mut Data, parameters: &SharedVector<Data>, expected_parameters: &Vec<FunctionParameter>) -> Status<()> {
        let mut parameter_stack = DataStack::new(parameters);

        for (index, expected_parameter) in expected_parameters.iter().enumerate() {
            if expected_parameter.variadic {
                ensure!(index == expected_parameters.len() - 1, InvalidVariadic, integer!(index as i64 + 1));
                let mut collected_parameters = SharedVector::new();

                while let Some(parameter) = parameter_stack.pop() {
                    if let Some(type_filter) = &expected_parameter.type_filter {
                        confirm!(ParameterType::validate(&parameter, integer!(index as i64 + 1), type_filter));
                    }
                    collected_parameters.push(parameter);
                }
                if let Some(key) = &expected_parameter.key {
                    let overwritten = confirm!(scope.set_entry(&key, list!(collected_parameters), false)); // TODO: CHECK IF IT WAS OVERWRITTEN AND ERROR IF YES
                    ensure!(!overwritten, Message, string!("parameters may not share the same name"));
                }
            } else {
                if let Some(type_filter) = &expected_parameter.type_filter {
                    let parameter = expect!(parameter_stack.pop(), ExpectedParameter, integer!(index as i64 + 1), ParameterType::expected_list(type_filter));
                    confirm!(ParameterType::validate(&parameter, integer!(index as i64 + 1), type_filter));
                    if let Some(key) = &expected_parameter.key {
                        let overwritten = confirm!(scope.set_entry(&key, parameter, false)); // TODO: CHECK IF IT WAS OVERWRITTEN AND ERROR IF YES
                        ensure!(!overwritten, Message, string!("parameters may not share the same name"));
                    }
                } else {
                    let parameter = expect!(parameter_stack.pop(), ExpectedParameter, integer!(index as i64 + 1), expected_list!["instance"]);
                    if let Some(key) = &expected_parameter.key {
                        let overwritten = confirm!(scope.set_entry(&key, parameter, false)); // TODO: CHECK IF IT WAS OVERWRITTEN AND ERROR IF YES
                        ensure!(!overwritten, Message, string!("parameters may not share the same name"));
                    }
                }
            }
        }

        match parameter_stack.pop() {
            Some(parameter) => return error!(UnexpectedParameter, parameter),
            None => return success!(()),
        }
    }
}
