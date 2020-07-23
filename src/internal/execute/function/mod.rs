mod parameter;

use internal::*;
use debug::*;

use super::instruction;
use self::parameter::FunctionParameter;

pub fn function(function: &Data, parameters: Vector<Data>, pass: &Option<Pass>, root: &Data, build: &Data) -> Status<Option<Data>> {
    let function_body = unpack_list!(function);
    let mut function_stack = DataStack::new(&function_body);
    let mut scope = map!();
    let mut last = None;

    let mut expected_parameters = Vec::new();
    while let Some(next) = function_stack.peek(0) {
        if next.is_list() {
            function_stack.advance(1);
            expected_parameters.push(confirm!(FunctionParameter::new(&next)));
        } else {
            break;
        }
    }

    confirm!(FunctionParameter::validate(&mut scope, &parameters, &expected_parameters));
    while let Some(instruction_name) = function_stack.pop() {
        let internal_function = unpack_keyword!(&instruction_name);
        if confirm!(instruction(&internal_function, None, &mut function_stack, &mut last, pass, root, &scope, build), Tag, instruction_name.clone()) {
            return success!(last);
        }
    }

    ensure!(function_stack.closed(), UnclosedScope);
    return success!(None);
}
