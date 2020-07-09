use std::collections::HashMap;
use super::Signature;

macro_rules! push_description {
    ($map:expr, $name:expr, $signature:ident, $width:expr) => ( $map.insert($name, Description::new(Signature::$signature, $width)) );
}

#[derive(Clone)]
pub struct Description {
    pub signature:      Signature,
    pub width:          usize,
}

impl Description {

    pub fn new(signature: Signature, width: usize) -> Self {
        Self {
            signature:      signature,
            width:          width,
        }
    }
}

lazy_static! {
    pub static ref CONDITIONS: HashMap<&'static str, Description> = {
        let mut map = HashMap::new();
        push_description!(map, "always",            Always,             1);
        push_description!(map, "not_always",        NotAlways,          1);
        push_description!(map, "zero",              Zero,               2);
        push_description!(map, "not_zero",          NotZero,            2);
        push_description!(map, "true",              True,               2);
        push_description!(map, "not_true",          NotTrue,            2);
        push_description!(map, "false",             False,              2);
        push_description!(map, "not_false",         NotFalse,           2);
        push_description!(map, "empty",             Empty,              2);
        push_description!(map, "not_empty",         NotEmpty,           2);
        push_description!(map, "instruction",       Instruction,        2);
        push_description!(map, "not_instruction",   NotInstruction,     2);
        push_description!(map, "condition",         Condition,          2);
        push_description!(map, "not_condition",     NotCondition,       2);
        push_description!(map, "last_some",         LastSome,           1);
        push_description!(map, "not_last_some",     NotLastSome,        1);
        push_description!(map, "uppercase",         Uppercase,          2);
        push_description!(map, "not_uppercase",     NotUppercase,       2);
        push_description!(map, "lowercase",         Lowercase,          2);
        push_description!(map, "not_lowercase",     NotLowercase,       2);
        push_description!(map, "equals",            Equals,             3);
        push_description!(map, "not_equals",        NotEquals,          3);
        push_description!(map, "present",           Present,            3);
        push_description!(map, "not_present",       NotPresent,         3);
        push_description!(map, "bigger",            Bigger,             3);
        push_description!(map, "not_bigger",        NotBigger,          3);
        push_description!(map, "smaller",           Smaller,            3);
        push_description!(map, "not_smaller",       NotSmaller,         3);
        push_description!(map, "contains",          Contains,           3);
        push_description!(map, "not_contains",      NotContains,        3);
        push_description!(map, "pure",              Pure,               2);
        push_description!(map, "not_pure",          NotPure,            2);
        push_description!(map, "file_present",      FilePresent,        2);
        push_description!(map, "not_file_present",  NotFilePresent,     2);
        push_description!(map, "map",               Map,                2);
        push_description!(map, "not_map",           NotMap,             2);
        push_description!(map, "list",              List,               2);
        push_description!(map, "not_list",          NotList,            2);
        push_description!(map, "path",              Path,               2);
        push_description!(map, "not_path",          NotPath,            2);
        push_description!(map, "string",            String,             2);
        push_description!(map, "not_string",        NotString,          2);
        push_description!(map, "character",         Character,          2);
        push_description!(map, "not_character",     NotCharacter,       2);
        push_description!(map, "identifier",        Identifier,         2);
        push_description!(map, "not_identifier",    NotIdentifier,      2);
        push_description!(map, "keyword",           Keyword,            2);
        push_description!(map, "not_keyword",       NotKeyword,         2);
        push_description!(map, "integer",           Integer,            2);
        push_description!(map, "not_integer",       NotInteger,         2);
        push_description!(map, "float",             Float,              2);
        push_description!(map, "not_float",         NotFloat,           2);
        push_description!(map, "boolean",           Boolean,            2);
        push_description!(map, "not_boolean",       NotBoolean,         2);
        push_description!(map, "key",               Key,                2);
        push_description!(map, "not_key",           NotKey,             2);
        push_description!(map, "container",         Container,          2);
        push_description!(map, "not_container",     NotContainer,       2);
        push_description!(map, "literal",           Literal,            2);
        push_description!(map, "not_literal",       NotLiteral,         2);
        push_description!(map, "selector",          Selector,           2);
        push_description!(map, "not_selector",      NotSelector,        2);
        push_description!(map, "number",            Number,             2);
        push_description!(map, "not_number",        NotNumber,          2);
        push_description!(map, "location",          Location,           2);
        push_description!(map, "not_location",      NotLocation,        2);
        map
    };
}
