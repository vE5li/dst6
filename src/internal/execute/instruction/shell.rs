use internal::*;
use std::io::{ BufRead, stdin };

pub fn shell(last: &mut Option<Data>, current_pass: &Option<VectorString>, root: &Data, scope: &Data, build: &Data, context: &Data) -> Status<()> {
    for line in stdin().lock().lines() {
        let source = format_vector!("[{}]", line.unwrap());
        let mut character_stack = CharacterStack::new(source, None);
        let data = confirm!(parse_data(&mut character_stack));

        let list = unpack_list!(&data);
        let mut instruction_stack = DataStack::new(&list);

        let instruction_name = expect!(instruction_stack.pop(), Message, string!(str, "shell expected instruction"));
        let instruction_name = match instruction_name.is_identifier() {
            true => unpack_identifier!(&instruction_name),
            false => unpack_keyword!(&instruction_name),
        };

        if instruction_name == VectorString::from("exit") {
            break;
        }

        let parameters = confirm!(instruction_stack.parameters(&last, root, &scope, build, context));
        let status = instruction(&instruction_name, Some(parameters), &mut instruction_stack, last, current_pass, root, scope, build, context);

        if let Status::Error(error) = status {
            println!("{}", error.display(&Some(root), build, context));
        }

        if let Some(last) = last {
            println!("$ {}", last.serialize());
        }
    }

    return success!(());
}
