use internal::*;
use debug::*;

use super::super::ParameterType;

#[derive(Debug, Clone)]
pub struct InstructionParameter {
    pub type_filter:    Option<Vec<ParameterType>>,
}

impl InstructionParameter {

    pub fn new(type_filter: Option<Vec<ParameterType>>) -> Self {
        Self {
            type_filter:    type_filter,
        }
    }

    pub fn validate(parameters: &SharedVector<Data>, expected_parameters: &Vec<InstructionParameter>, variadic: bool) -> Status<Vec<Data>> {
        let mut parameter_stack = DataStack::new(parameters);
        let mut collected_parameters = Vec::new();

        for (index, expected_parameter) in expected_parameters.iter().enumerate() {
            if index == expected_parameters.len() - 1 && variadic {
                while let Some(parameter) = parameter_stack.pop() {
                    if let Some(type_filter) = &expected_parameter.type_filter {
                        confirm!(ParameterType::validate(&parameter, integer!(index as i64 + 1), type_filter));
                    }
                    collected_parameters.push(parameter);
                }
            } else {
                if let Some(type_filter) = &expected_parameter.type_filter {
                    let parameter = expect!(parameter_stack.pop(), ExpectedParameter, integer!(index as i64 + 1), ParameterType::expected_list(type_filter));
                    confirm!(ParameterType::validate(&parameter, integer!(index as i64 + 1), type_filter));
                    collected_parameters.push(parameter);
                } else {
                    let parameter = expect!(parameter_stack.pop(), ExpectedParameter, integer!(index as i64 + 1), expected_list!["instance"]);
                    collected_parameters.push(parameter);
                }
            }
        }

        match parameter_stack.pop() {
            Some(parameter) => return error!(UnexpectedParameter, parameter),
            None => return success!(collected_parameters),
        }
    }
}
