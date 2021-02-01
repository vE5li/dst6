#[macro_export]
macro_rules! success {
    ($content:expr) => (Status::Success($content));
}

#[macro_export]
macro_rules! guaranteed {
    ($status:expr) => (
        match $status {
            Status::Success(content) => content,
            Status::Error(_error) => panic!(),
        };
    );
    ($status:expr, $message:expr) => (
        match $status {
            Status::Success(content) => content,
            Status::Error(_error) => panic!("{}", $message),
        };
    );
}

#[macro_export]
macro_rules! confirm {
    ($status:expr) => (
        match $status {
            Status::Success(content) => content,
            Status::Error(error) => return $crate::Status::Error(error),
        };
    );
    ($status:expr, Tag, $tag:expr) => (
        match $status {
            Status::Success(content) => content,
            Status::Error(error) => return error!(Tag, error, $tag),
        };
    );
    ($status:expr, $wrapper:ident, $($arguments:tt)*) => (
        match $status {
            Status::Success(content) => content,
            Status::Error(error) => return error!($wrapper, error, $($arguments)*),
        };
    );
}

#[macro_export]
macro_rules! expect {
    ($item:expr, $($arguments:tt)*) => ({
        match $item {
            Some(item) => item,
            None => return error!($($arguments)*),
        }
    });
}

#[macro_export]
macro_rules! ensure {
    ($item:expr, $($arguments:tt)*) => ({
        if !$item {
            return error!($($arguments)*);
        }
    });
}

macro_rules! push_by_length {
    ($collection:expr, $single_item:expr) => (
        match $collection.iter().position(|iterator: &SharedString| iterator.len() <= $single_item.len()) {
            Some(index) => $collection.insert(index, $single_item),
            None => $collection.push($single_item),
        }
    );
    ($collection:expr, $primary_item:expr, $secondary_item:expr) => (
        match $collection.iter().position(|iterator: &(SharedString, _)| iterator.0.len() <= $primary_item.len()) {
            Some(index) => $collection.insert(index, ($primary_item, $secondary_item)),
            None => $collection.push(($primary_item, $secondary_item)),
        }
    )
}

#[macro_export]
macro_rules! display {
    ($status:expr) => (
        match $status {
            Status::Success(content) => content,
            Status::Error(error) => {
                println!("{}", error.display(&None, &map!()));
                std::process::exit(1);
            },
        };
    );
    ($status:expr, $root:expr, $build:expr) => (
        match $status {
            Status::Success(content) => content,
            Status::Error(error) => {
                println!("{}", error.display($root, $build));
                std::process::exit(1);
            },
        };
    );
}

macro_rules! format_hook {
    ($root:expr, $build:expr, $name:expr, $parameters:expr, $($arguments:tt)*) => (
        if let Some(root) = $root {
            let formatter_function_path = path!(vector![keyword!("function"), keyword!($name)]);
            let formatter_function = match root.index(&formatter_function_path) {
                Status::Success(formatter_function) => formatter_function,
                Status::Error(error) => panic!("index root failed: {}", error.display($root, $build)),
            };

            if let Some(formatter_function) = formatter_function {
                match function(&formatter_function, $parameters, &None, root, $build) {

                    Status::Success(return_value) => {
                        match return_value {

                            Some(return_value) => {
                                match return_value {
                                    Data::String(ref string) => string.clone(),
                                    _ => format_shared!("error in formatter {}: must return a string; found {}", $name, return_value.serialize()),
                                }
                            },

                            None => format_shared!("error in formatter {}: must return a string", $name),
                        }
                    },

                    Status::Error(error) => format_shared!("error in formatter {}: {}", $name, error.display(&None, $build)),
                }
            } else {
                format_shared!($($arguments)*)
            }
        } else {
            format_shared!($($arguments)*)
        }
    );
}

#[macro_export]
macro_rules! error {
    (Tag, $error:expr, $tag:expr)                                       => (Status::Error(Error::Tag($tag, Box::new($error))));
    (Compiler, $errors:expr)                                            => (Status::Error(Error::Compiler($errors)));
    (Tokenizer, $errors:expr)                                           => (Status::Error(Error::Tokenizer($errors)));
    (Parser, $errors:expr)                                              => (Status::Error(Error::Parser($errors)));
    (Builder, $errors:expr)                                             => (Status::Error(Error::Builder($errors)));
    (Execute, $name:expr, $error:expr)                                  => (Status::Error(Error::Execute($name, $error)));
    (InvalidItemCount, $specified:expr, $received:expr)                 => (Status::Error(Error::InvalidItemCount($specified, $received)));
    (InvalidCondition, $condition:expr)                                 => (Status::Error(Error::InvalidCondition($condition)));
    (UnexpectedToken, $token:expr)                                      => (Status::Error(Error::UnexpectedToken($token)));
    (InvalidToken, $token_type:expr, $token:expr)                       => (Status::Error(Error::InvalidToken($token_type, $token)));
    (InvalidTokenType, $token_type:expr)                                => (Status::Error(Error::InvalidTokenType($token_type)));
    (InvalidLocation, $location:expr)                                   => (Status::Error(Error::InvalidLocation($location)));
    (Expected, $expected:expr)                                          => (Status::Error(Error::Expected($expected)));
    (ExpectedFound, $expected:expr, $found:expr)                        => (Status::Error(Error::ExpectedFound($expected, $found)));
    (InvalidPieceType, $piece_type:expr)                                => (Status::Error(Error::InvalidPieceType($piece_type)));
    (UnregisteredCharacter, $character:expr)                            => (Status::Error(Error::UnregisteredCharacter($character)));
    (DuplicateSignature, $signature:expr)                               => (Status::Error(Error::DuplicateSignature($signature)));
    (DuplicateBreaking, $character:expr)                                => (Status::Error(Error::DuplicateBreaking($character)));
    (DuplicateNonBreaking, $character:expr)                             => (Status::Error(Error::DuplicateNonBreaking($character)));
    (ExpectedIdentifierType, $found:expr)                               => (Status::Error(Error::ExpectedIdentifierType($found)));
    (EmptyLiteral)                                                      => (Status::Error(Error::EmptyLiteral));
    (InvalidCharacterLength, $found:expr)                               => (Status::Error(Error::InvalidCharacterLength($found)));
    (InvalidPathLength, $found:expr)                                    => (Status::Error(Error::InvalidPathLength($found)));
    (ExpectedReturn, $expected:expr)                                    => (Status::Error(Error::ExpectedReturn($expected)));
    (ExpectedReturnFound, $expected:expr, $found:expr)                  => (Status::Error(Error::ExpectedReturnFound($expected, $found)));
    (ExpectedParameter, $number:expr, $expected:expr)                   => (Status::Error(Error::ExpectedParameter($number, $expected)));
    (ExpectedParameterFound, $number:expr, $expected:expr, $found:expr) => (Status::Error(Error::ExpectedParameterFound($number, $expected, $found)));
    (ExpectedCondition)                                                 => (Status::Error(Error::ExpectedCondition));
    (ExpectedConditionFound, $found:expr)                               => (Status::Error(Error::ExpectedConditionFound($found)));
    (InexplicitOverwrite, $selector:expr, $previous:expr)               => (Status::Error(Error::InexplicitOverwrite($selector, $previous)));
    (MissingEntry, $key:expr)                                           => (Status::Error(Error::MissingEntry($key)));
    (InvalidType, $type:expr)                                           => (Status::Error(Error::InvalidType($type)));
    (InvalidVariadic, $number:expr)                                     => (Status::Error(Error::InvalidVariadic($number)));
    (UnexpectedParameter, $instance:expr)                               => (Status::Error(Error::UnexpectedParameter($instance)));
    (UnclosedScope)                                                     => (Status::Error(Error::UnclosedScope));
    (InvalidCompilerFunction, $function:expr)                           => (Status::Error(Error::InvalidCompilerFunction($function)));
    (UnexpectedCompilerFunction, $function:expr)                        => (Status::Error(Error::UnexpectedCompilerFunction($function)));
    (ExpectedLocation)                                                  => (Status::Error(Error::ExpectedLocation));
    (ExpectedLocationFound, $found:expr)                                => (Status::Error(Error::ExpectedLocationFound($found)));
    (ExpectedImmediate)                                                 => (Status::Error(Error::ExpectedImmediate));
    (UnexpectedImmediate, $immediate:expr)                              => (Status::Error(Error::UnexpectedImmediate($immediate)));
    (NoPreviousReturn)                                                  => (Status::Error(Error::NoPreviousReturn));
    (MissingFile, $filename:expr)                                       => (Status::Error(Error::MissingFile($filename)));
    (ExpectedBooleanFound, $found:expr)                                 => (Status::Error(Error::ExpectedBooleanFound($found)));
    (UnterminatedToken, $type:expr)                                     => (Status::Error(Error::UnterminatedToken($type)));
    (IndexOutOfBounds, $selector:expr, $biggest:expr)                   => (Status::Error(Error::IndexOutOfBounds($selector, $biggest)));
    (NothingToParse)                                                    => (Status::Error(Error::NothingToParse));
    (UnterminatedEscapeSequence)                                        => (Status::Error(Error::UnterminatedEscapeSequence));
    (InvalidEscapeSequence, $list:expr)                                 => (Status::Error(Error::InvalidEscapeSequence($list)));
    (InvalidPrefix, $prefix:expr)                                       => (Status::Error(Error::InvalidPrefix($prefix)));
    (InvalidSuffix, $suffix:expr)                                       => (Status::Error(Error::InvalidSuffix($suffix)));
    (InvalidNumber, $system:expr)                                       => (Status::Error(Error::InvalidNumber($system)));
    (ExpectedWord)                                                      => (Status::Error(Error::ExpectedWord));
    (ExpectedWordFound, $found:expr)                                    => (Status::Error(Error::ExpectedWordFound($found)));
    (InvalidNumberSystem, $system:expr)                                 => (Status::Error(Error::InvalidNumberSystem($system)));
    (AmbiguousIdentifier, $identifier:expr)                             => (Status::Error(Error::AmbiguousIdentifier($identifier)));
    ($message:expr)                                                     => (Status::Error(Error::Message($message)));
}

#[macro_export]
macro_rules! boolean_to_string {
    ($boolean:expr) => (
        match $boolean {
            true => SharedString::from("true"),
            false => SharedString::from("false"),
        }
    );
}

#[macro_export]
macro_rules! ensure_empty {
    ($stack:expr, $error:ident) => ({
        if let Some(instance) = $stack.pop() {
            return error!($error, instance);
        }
    });
}

#[macro_export]
macro_rules! index { // FINALLY FIX ME
    ($container:expr, $selector:expr) => ({
        let instance = confirm!($container.index($selector));
        expect!(instance, MissingEntry, $selector.clone())
    });
    ($container:expr, $selector:expr, $($arguments:tt)*) => ({
        let instance = confirm!($container.index($selector));
        expect!(instance, $($arguments)*)
    });
}

#[macro_export]
macro_rules! index_field {
    ($container:expr, $name:expr) => ({
        let selector = keyword!($name);
        let instance = confirm!($container.index(&selector));

        match instance {
            Some(instance) => instance,
            None => return error!(MissingEntry, selector.clone()),
        }
    });
    ($container:expr, $name:expr, $($arguments:tt)*) => ({
        let selector = keyword!($name);
        let instance = confirm!($container.index(&selector));

        match instance {
            Some(instance) => instance,
            None => return error!($($arguments)*),
        }
    });
}

#[macro_export]
macro_rules! expected_list {
    ($($arguments:tt)*) => (list!([$($arguments)*].iter().map(|item| keyword!(item.as_ref() as &str)).collect()));
}

#[macro_export]
macro_rules! format_shared {
    ($format:expr) => (SharedString::from($format));
    ($format:expr, $($arguments:tt)*) => (SharedString::from(&format!($format, $($arguments)*)));
}

#[macro_export]
macro_rules! vector {
    ()                  => (Vector::<_>::new());
    ($($arguments:tt)*) => ([$($arguments)*].iter().cloned().collect::<SharedVector<_>>());
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! map {
    ()             => (Data::Map(DataMap::new()));
    ($fields:expr) => (Data::Map($fields));
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! list {
    ()            => (Data::List(SharedVector::new()));
    ($items:expr) => (Data::List($items));
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! path {
    ($steps:expr) => (Data::Path($steps));
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! identifier {
    (String, $identifier:expr) => (Data::Identifier($identifier));
    ($identifier:expr) => (Data::Identifier(SharedString::from($identifier)));
    ($identifier:expr, $($arguments:tt)*) => (Data::Identifier(format_shared!($identifier, $($arguments)*)));
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! keyword {
    (String, $keyword:expr) => (Data::Keyword($keyword));
    ($keyword:expr) => (Data::Keyword(SharedString::from($keyword)));
    ($keyword:expr, $($arguments:tt)*) => (Data::Keyword(format_shared!($keyword, $($arguments)*)));
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! string {
    () => (Data::String(SharedString::new()));
    (String, $string:expr) => (Data::String($string));
    ($string:expr) => (Data::String(SharedString::from($string)));
    ($string:expr, $($arguments:tt)*) => (Data::String(format_shared!($string, $($arguments)*)));
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! character {
    ($character:expr) => (Data::Character($character));
    (char, $code:expr) => (Data::Character(Character::from_char($code)));
    (code, $code:expr) => (Data::Character(Character::from_code($code)));
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! integer {
    ($integer:expr) => (Data::Integer($integer));
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! float {
    ($float:expr) => (Data::Float($float));
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! boolean {
    ($boolean:expr) => (Data::Boolean($boolean));
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! unpack_map {
    ($instance:expr) => (
        match $instance {
            Data::Map(ref map) => map.clone(),
            _other => return error!(ExpectedFound, expected_list!["map"], $instance.clone()),
        }
    );
    ($instance:expr, $($arguments:tt)*) => (
        match $instance {
            Data::Map(ref map) => map.clone(),
            _other => return error!($($arguments)*),
        }
    );
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! unpack_list {
    ($instance:expr) => (
        match $instance {
            Data::List(ref items) => items.clone(),
            _other => return error!(ExpectedFound, expected_list!["list"], $instance.clone()),
        }
    );
    ($instance:expr, $($arguments:tt)*) => (
        match $instance {
            Data::List(ref items) => items.clone(),
            _other => return error!($($arguments)*),
        }
    );
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! unpack_path {
    ($instance:expr) => (
        match $instance {
            Data::Path(ref steps) => steps.clone(),
            _other => return error!(ExpectedFound, expected_list!["path"], $instance.clone()),
        }
    );
    ($instance:expr, $($arguments:tt)*) => (
        match $instance {
            Data::Path(ref steps) => steps.clone(),
            _other => return error!($($arguments)*),
        }
    );
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! unpack_keyword {
    ($instance:expr) => (
        match $instance {
            Data::Keyword(ref keyword) => keyword.clone(),
            _other => return error!(ExpectedFound, expected_list!["keyword"], $instance.clone()),
        }
    );
    ($instance:expr, $($arguments:tt)*) => (
        match $instance {
            Data::Keyword(ref keyword) => keyword.clone(),
            _other => return error!($($arguments)*),
        }
    );
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! unpack_integer {
    ($instance:expr) => (
        match $instance {
            Data::Integer(ref integer) => *integer,
            _other => return error!(ExpectedFound, expected_list!["integer"], $instance.clone()),
        }
    );
    ($instance:expr, $($arguments:tt)*) => (
        match $instance {
            Data::Integer(ref integer) => *integer,
            _other => return error!($($arguments)*),
        }
    );
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! unpack_number {
    ($instance:expr) => (
        match $instance {
            Data::Integer(ref integer) => *integer,
            Data::Character(ref character) => character.code() as i64,
            _other => return error!(ExpectedFound, expected_list!["integer"], $instance.clone()),
        }
    );
    ($instance:expr, $($arguments:tt)*) => (
        match $instance {
            Data::Integer(ref integer) => *integer,
            Data::Character(ref character) => *character as i64,
            _other => return error!($($arguments)*),
        }
    );
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! unpack_float {
    ($instance:expr) => (
        match $instance {
            Data::Float(ref float) => *float,
            _other => return error!(ExpectedFound, expected_list!["float"], $instance.clone()),
        }
    );
    ($instance:expr, $($arguments:tt)*) => (
        match $instance {
            Data::Float(ref float) => *float,
            _other => return error!($($arguments)*),
        }
    );
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! unpack_character {
    ($instance:expr) => (
        match $instance {
            Data::Character(ref character) => *character,
            _other => return error!(ExpectedFound, expected_list!["character"], $instance.clone()),
        }
    );
    ($instance:expr, $($arguments:tt)*) => (
        match $instance {
            Data::Character(ref character) => *character,
            _other => return error!($($arguments)*),
        }
    );
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! unpack_string {
    ($instance:expr) => (
        match $instance {
            Data::String(ref string) => string.clone(),
            _other => return error!(ExpectedFound, expected_list!["string"], $instance.clone()),
        }
    );
    ($instance:expr, $($arguments:tt)*) => (
        match $instance {
            Data::String(ref string) => string.clone(),
            _other => return error!($($arguments)*),
        }
    );
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! unpack_identifier {
    ($instance:expr) => (
        match $instance {
            Data::Identifier(ref identifier) => identifier.clone(),
            _other => return error!(ExpectedFound, expected_list!["identifier"], $instance.clone()),
        }
    );
    ($instance:expr, $($arguments:tt)*) => (
        match $instance {
            Data::Identifier(ref identifier) => identifier.clone(),
            _other => return error!($($arguments)*),
        }
    );
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! unpack_boolean {
    ($instance:expr) => (
        match $instance {
            Data::Boolean(ref boolean) => *boolean,
            _other => return error!(ExpectedFound, list!(vector![identifier!("boolean")]), $instance.clone()),
        }
    );
    ($instance:expr, $($arguments:tt)*) => (
        match $instance {
            Data::Boolean(ref boolean) => *boolean,
            _other => return error!($($arguments)*),
        }
    );
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! unpack_literal {
    ($instance:expr) => (
        match $instance {
            Data::String(ref string) => string.clone(),
            Data::Identifier(ref identifier) => identifier.clone(),
            Data::Keyword(ref keyword) => keyword.clone(),
            Data::Character(ref character) => character.to_string(),
            _other => return error!(ExpectedFound, expected_list!["literal"], $instance.clone()),
        }
    );
    ($instance:expr, $($arguments:tt)*) => (
        match $instance {
            Data::String(ref string) => string.clone(),
            Data::Identifier(ref identifier) => identifier.clone(),
            Data::Keyword(ref keyword) => keyword.clone(),
            Data::Character(ref character) => character.to_string(),
            _other => return error!($($arguments)*),
        }
    );
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! unpack_key {
    ($instance:expr) => (
        match $instance {
            Data::String(ref string) => string.clone(),
            Data::Identifier(ref identifier) => identifier.clone(),
            Data::Keyword(ref keyword) => keyword.clone(),
            Data::Character(ref character) => character.to_string(),
            _other => return error!(ExpectedFound, expected_list!["key"], $instance.clone()),
        }
    );
    ($instance:expr, $($arguments:tt)*) => (
        match $instance {
            Data::String(ref string) => string.clone(),
            Data::Identifier(ref identifier) => identifier.clone(),
            Data::Keyword(ref keyword) => keyword.clone(),
            Data::Character(ref character) => character.to_string(),
            _other => return error!($($arguments)*),
        }
    );
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! extract_map {
    ($instance:expr) => (
        match $instance {
            Data::Map(ref map) => map.clone(),
            _other => panic!("failed to extract map"),
        }
    );
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! extract_list {
    ($instance:expr) => (
        match $instance {
            Data::List(ref items) => items.clone(),
            _other => panic!("failed to extract list"),
        }
    );
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! extract_path {
    ($instance:expr) => (
        match $instance {
            Data::Path(ref steps) => steps.clone(),
            _other => panic!("failed to extract path"),
        }
    );
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! extract_keyword {
    ($instance:expr) => (
        match $instance {
            Data::Keyword(ref keyword) => keyword.clone(),
            _other => panic!("failed to extract keyword"),
        }
    );
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! extract_integer {
    ($instance:expr) => (
        match $instance {
            Data::Integer(ref integer) => *integer,
            _other => panic!("failed to extract integer"),
        }
    );
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! extract_character {
    ($instance:expr) => (
        match $instance {
            Data::Character(ref character) => *character,
            _other => panic!("failed to extract character"),
        }
    );
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! extract_string {
    ($instance:expr) => (
        match $instance {
            Data::String(ref string) => string.clone(),
            _other => panic!("failed to extract string"),
        }
    );
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! extract_identifier {
    ($instance:expr) => (
        match $instance {
            Data::Identifier(ref identifier) => identifier.clone(),
            _other => panic!("failed to extract identifier"),
        }
    );
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! extract_boolean {
    ($instance:expr) => (
        match $instance {
            Data::Boolean(ref boolean) => *boolean,
            _other => panic!("failed to extract boolean"),
        }
    );
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! extract_literal {
    ($instance:expr) => (
        match $instance {
            Data::String(ref string) => string.clone(),
            Data::Identifier(ref identifier) => identifier.clone(),
            Data::Keyword(ref keyword) => keyword.clone(),
            Data::Character(ref character) => character.to_string(),
            _other => panic!("failed to extract literal"),
        }
    );
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! extract_key {
    ($instance:expr) => (
        match $instance {
            Data::String(ref string) => string.clone(),
            Data::Identifier(ref identifier) => identifier.clone(),
            Data::Keyword(ref keyword) => keyword.clone(),
            Data::Character(ref character) => character.to_string(),
            _other => panic!("failed to extract key"),
        }
    );
}
