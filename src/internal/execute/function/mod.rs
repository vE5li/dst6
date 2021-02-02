mod parameter;

use internal::*;
use debug::*;

use super::instruction;
use self::parameter::FunctionParameter;

pub fn function(function_path: &Data, parameters: SharedVector<Data>, pass: &Option<Pass>, root: &Data, build: &Data) -> Status<Option<Data>> {

    let mut full_steps = vector![keyword!("functions")];
    if function_path.is_path() {
        unpack_path!(function_path).iter().for_each(|step| full_steps.push(step.clone()));
    } else {
        ensure!(function_path.is_selector(), string!("path may only contain selectors; found {}", function_path.serialize()));
        full_steps.push(function_path.clone());
    }

    let full_path = path!(full_steps);
    let function_entry = confirm!(root.index(&full_path));
    let function_list = expect!(function_entry, string!("failed to get function {}", full_path.serialize()));
    let function_body = unpack_list!(&function_list);

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
