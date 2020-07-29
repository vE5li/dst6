mod number;
mod stack;

use internal::*;
use debug::*;

pub use self::stack::CharacterStack;

use self::number::parse_number;

// new parse_string that returns vector of data (for error handling)

fn collect(character_stack: &mut CharacterStack, name: &str, compare: char) -> Status<SharedString> {
    let mut literal = SharedString::new();
    while let Some(character) = character_stack.pop() {
        match character.as_char() {

            '\\' => {
                let next = expect!(character_stack.pop(), UnterminatedEscapeSequence);
                match next.as_char() {
                    '\\' => literal.push(Character::from_char('\\')),
                    '0' => literal.push(Character::from_char('\0')),
                    'b' => literal.push(Character::from_code(8)),
                    'e' => literal.push(Character::from_code(27)),
                    'n' => literal.push(Character::from_char('\n')),
                    't' => literal.push(Character::from_char('\t')),
                    'r' => literal.push(Character::from_char('\r')),
                    '\'' => literal.push(Character::from_char('\'')),
                    '\"' => literal.push(Character::from_char('\"')),
                    '[' => {
                        let mut code = String::new();
                        while !character_stack.check(']') {
                            ensure!(!character_stack.is_empty(), UnterminatedToken, identifier!("character"));
                            code.push(character_stack.pop().unwrap().as_char());
                        }
                        match code.parse::<u32>() {
                            Ok(value) => literal.push(Character::from_code(value)),
                            Err(_) => return error!(InvalidNumber, identifier!("decimal")),
                        }
                    },
                    invalid => return error!(InvalidEscapeSequence, string!("\\{}", invalid)),
                }
            },

            other => {
                match other == compare {
                    true => return success!(literal),
                    false => literal.push(character),
                }
            },
        }
    }
    return error!(UnterminatedToken, identifier!(name)); // TODO
}

fn check_path(character_stack: &mut CharacterStack, first: Data) -> Status<Data> {
    if character_stack.check(':') {
        ensure!(first.is_selector(), ExpectedFound, expected_list!["selector"], first);
        let mut path_data = vector![first];
        let next = confirm!(parse_data(character_stack));

        if next.is_path() {
            path_data.append(&mut unpack_path!(&next));
        } else {
            ensure!(next.is_selector(), ExpectedFound, expected_list!["selector"], next);
            path_data.push(next);
        }

        return success!(path!(path_data));
    } else {
        return success!(first);
    }
}

fn update(character_stack: &mut CharacterStack) -> Status<()> {
    'update: while let Some(character) = character_stack.peek(0) {
        match character.as_char() {

            '@' => {
                character_stack.advance(1);

                if character_stack.check('@') {
                    while let Some(character) = character_stack.pop() {
                        if character.as_char() == '@' && character_stack.check('@') {
                            continue 'update;
                        }
                    }
                    return error!(UnterminatedToken, identifier!("comment")); // TODO
                }

                while let Some(character) = character_stack.pop() {
                    if character.as_char() == '\n' {
                        continue 'update;
                    }
                }
            },

            '\n' | '\t' | '\r' | ';' | '.' | ' ' => character_stack.advance(1),

            _ => return success!(()),
        }
    }

    return success!(());
}

pub fn parse_data(character_stack: &mut CharacterStack) -> Status<Data> {
    confirm!(update(character_stack));
    if let Some(character) = character_stack.peek(0) {
        match character.as_char() {

            '{' => {
                character_stack.advance(1);
                confirm!(update(character_stack));
                let mut map = DataMap::new();
                while !character_stack.check('}') {
                    ensure!(!character_stack.is_empty(), UnterminatedToken, identifier!("map"));
                    let key = confirm!(parse_data(character_stack));
                    let value = confirm!(parse_data(character_stack));

                    ensure!(!key.is_path() && !key.is_integer(), ExpectedFound, expected_list!["key"], key);

                    if let Some(previous) = map.insert(key.clone(), value) {
                        return error!(InexplicitOverwrite, key, previous);
                    }
                    confirm!(update(character_stack));
                }
                return success!(map!(map));
            },

            '[' => {
                character_stack.advance(1);
                confirm!(update(character_stack));
                let mut items = SharedVector::new();
                while !character_stack.check(']') {
                    ensure!(!character_stack.is_empty(), UnterminatedToken, identifier!("list"));
                    items.push(confirm!(parse_data(character_stack)));
                    confirm!(update(character_stack));
                }
                return success!(list!(items));
            },

            '#' => {
                character_stack.advance(1);
                let keyword = confirm!(character_stack.till_breaking());
                return success!(confirm!(check_path(character_stack, keyword!(String, keyword))));
            },

            '\'' => {
                character_stack.advance(1);
                let character = confirm!(collect(character_stack, "character", '\''));
                ensure!(character.len() == 1, InvalidCharacterLength, string!(String, character));
                let character = character.chars().next().unwrap();
                return success!(confirm!(check_path(character_stack, character!(*character))));
            },

            '\"' => {
                character_stack.advance(1);
                let string = confirm!(collect(character_stack, "string", '\"'));
                return success!(confirm!(check_path(character_stack, string!(String, string))));
            },

            '$' => {
                character_stack.advance(1);
                let word = confirm!(character_stack.till_breaking());
                match word.printable().as_str() {
                    "true" => return success!(boolean!(true)),
                    "false" => return success!(boolean!(false)),
                    _ => return error!(ExpectedBooleanFound, identifier!(String, word)),
                }
            },

            '-' => {
                character_stack.advance(1);
                let word = confirm!(character_stack.till_breaking());
                if character_stack.check('.') {
                    let float_source = confirm!(character_stack.till_breaking());
                    return success!(confirm!(parse_number(&word, Some(&float_source), true)).unwrap()); // TODO: dont unwrap
                } else {
                    match confirm!(parse_number(&word, None, true)) {
                        Some(data) => return success!(confirm!(check_path(character_stack, data))),
                        None => return error!(Message, string!("expected number after -")),
                    }
                }
            }

            _other => {
                let word = confirm!(character_stack.till_breaking());
                if character_stack.check('.') {
                    let float_source = confirm!(character_stack.till_breaking());
                    return success!(confirm!(parse_number(&word, Some(&float_source), false)).unwrap()); // TODO: dont unwrap
                } else {
                    match confirm!(parse_number(&word, None, false)) {
                        Some(data) => return success!(confirm!(check_path(character_stack, data))),
                        None => return success!(confirm!(check_path(character_stack, identifier!(String, word)))),
                    }
                }
            },
        }
    }
    return error!(NothingToParse);
}
