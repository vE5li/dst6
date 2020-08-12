use super::super::ParameterType;
use super::InstructionParameter;
use std::collections::HashMap;
use super::Signature;

macro_rules! push_description {
    ($map:expr, $name:expr, $signature:ident, $invokable:expr, $conditional:expr, $variadic:expr) => ( $map.insert($name, Description::new(Signature::$signature, $invokable, $conditional, $variadic, Vec::new())) );
    ($map:expr, $name:expr, $signature:ident, $invokable:expr, $conditional:expr, $variadic:expr, $($arguments:tt)*) => ( $map.insert($name, Description::new(Signature::$signature, $invokable, $conditional, $variadic, vec![$($arguments)*])) );
}

#[derive(Clone)]
pub struct Description {
    pub signature:      Signature,
    pub invokable:      bool,
    pub conditional:    bool,
    pub variadic:       bool,
    pub parameters:     Vec<InstructionParameter>,
}

impl Description {

    pub fn new(signature: Signature, invokable: bool, conditional: bool, variadic: bool, parameters: Vec<InstructionParameter>) -> Self {
        Self {
            signature:      signature,
            invokable:      invokable,
            conditional:    conditional,
            variadic:       variadic,
            parameters:     parameters,
        }
    }
}

lazy_static! {
    pub static ref INSTRUCTIONS: HashMap<&'static str, Description> = {
        let mut map = HashMap::new();
        push_description!(map, "map",           Map,            true,   false,  true,   InstructionParameter::new(None));
        push_description!(map, "list",          List,           true,   false,  true,   InstructionParameter::new(None));
        push_description!(map, "path",          Path,           true,   false,  true,   InstructionParameter::new(Some(vec![ParameterType::Location])), InstructionParameter::new(Some(vec![ParameterType::Location])));
        push_description!(map, "string",        String,         true,   false,  true,   InstructionParameter::new(None));
        push_description!(map, "identifier",    Identifier,     true,   false,  true,   InstructionParameter::new(None));
        push_description!(map, "keyword",       Keyword,        true,   false,  true,   InstructionParameter::new(None));
        push_description!(map, "float",         Float,          true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Number])));
        push_description!(map, "integer",       Integer,        true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Number])));
        push_description!(map, "character",     Character,      true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Number])));
        push_description!(map, "boolean",       Boolean,        true,   false,  true,   InstructionParameter::new(Some(vec![ParameterType::Keyword])), InstructionParameter::new(None));
        push_description!(map, "type",          Type,           true,   false,  false,  InstructionParameter::new(None));
        push_description!(map, "length",        Length,         true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Container])));
        push_description!(map, "random",        Random,         true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Number])), InstructionParameter::new(Some(vec![ParameterType::Number])));
        push_description!(map, "time",          Time,           true,   false,  false);
        push_description!(map, "input",         Input,          false,  false,  true,   InstructionParameter::new(Some(vec![ParameterType::Literal])));
        push_description!(map, "shell",         Shell,          false,  false,  false);
        push_description!(map, "return",        Return,         true,   false,  true,   InstructionParameter::new(None));
        push_description!(map, "remember",      Remember,       true,   false,  false,  InstructionParameter::new(None));
        push_description!(map, "fuze",          Fuze,           true,   false,  true,   InstructionParameter::new(None));
        push_description!(map, "range",         Range,          true,   false,  true,   InstructionParameter::new(None));
        push_description!(map, "fill",          Fill,           true,   false,  false,  InstructionParameter::new(None), InstructionParameter::new(Some(vec![ParameterType::Literal])), InstructionParameter::new(Some(vec![ParameterType::Integer, ParameterType::Character])));
        push_description!(map, "fill_back",     FillBack,       true,   false,  false,  InstructionParameter::new(None), InstructionParameter::new(Some(vec![ParameterType::Literal])), InstructionParameter::new(Some(vec![ParameterType::Integer, ParameterType::Character])));
        push_description!(map, "print",         Print,          true,   false,  true,   InstructionParameter::new(None));
        push_description!(map, "print_line",    PrintLine,      true,   false,  true,   InstructionParameter::new(None));
        push_description!(map, "error",         Error,          true,   false,  true,   InstructionParameter::new(None));
        push_description!(map, "ensure",        Ensure,         true,   false,  true,   InstructionParameter::new(Some(vec![ParameterType::Keyword])), InstructionParameter::new(None));
        push_description!(map, "add",           Add,            true,   false,  true,   InstructionParameter::new(Some(vec![ParameterType::Number])), InstructionParameter::new(Some(vec![ParameterType::Number])), InstructionParameter::new(Some(vec![ParameterType::Number])));
        push_description!(map, "subtract",      Subtract,       true,   false,  true,   InstructionParameter::new(Some(vec![ParameterType::Number])), InstructionParameter::new(Some(vec![ParameterType::Number])), InstructionParameter::new(Some(vec![ParameterType::Number])));
        push_description!(map, "multiply",      Multiply,       true,   false,  true,   InstructionParameter::new(Some(vec![ParameterType::Number])), InstructionParameter::new(Some(vec![ParameterType::Number])), InstructionParameter::new(Some(vec![ParameterType::Number])));
        push_description!(map, "divide",        Divide,         true,   false,  true,   InstructionParameter::new(Some(vec![ParameterType::Number])), InstructionParameter::new(Some(vec![ParameterType::Number])), InstructionParameter::new(Some(vec![ParameterType::Number])));
        push_description!(map, "modulo",        Modulo,         true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Number])), InstructionParameter::new(Some(vec![ParameterType::Number]))); // floats (?)
        push_description!(map, "logarithm",     Logarithm,      true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Number])), InstructionParameter::new(Some(vec![ParameterType::Number]))); // floats (?)
        push_description!(map, "power",         Power,          true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Number])), InstructionParameter::new(Some(vec![ParameterType::Number]))); // floats (?)
        push_description!(map, "square_root",   SquareRoot,     true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Number])));
        push_description!(map, "negate",        Negate,         true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Number])));
        push_description!(map, "absolute",      Absolute,       true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Number])));
        push_description!(map, "ceiling",       Ceiling,        true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Number])));
        push_description!(map, "round",         Round,          true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Number])));
        push_description!(map, "floor",         Floor,          true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Number])));
        push_description!(map, "sine",          Sine,           true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Number])));
        push_description!(map, "cosine",        Cosine,         true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Number])));
        push_description!(map, "tangent",       Tangent,        true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Number])));
        push_description!(map, "not",           Not,            true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Integer, ParameterType::Character, ParameterType::Boolean])));
        push_description!(map, "and",           And,            true,   false,  true,   InstructionParameter::new(Some(vec![ParameterType::Integer, ParameterType::Character, ParameterType::Boolean])), InstructionParameter::new(Some(vec![ParameterType::Integer, ParameterType::Character, ParameterType::Boolean])), InstructionParameter::new(Some(vec![ParameterType::Integer, ParameterType::Character, ParameterType::Boolean])));
        push_description!(map, "or",            Or,             true,   false,  true,   InstructionParameter::new(Some(vec![ParameterType::Integer, ParameterType::Character, ParameterType::Boolean])), InstructionParameter::new(Some(vec![ParameterType::Integer, ParameterType::Character, ParameterType::Boolean])), InstructionParameter::new(Some(vec![ParameterType::Integer, ParameterType::Character, ParameterType::Boolean])));
        push_description!(map, "xor",           Xor,            true,   false,  true,   InstructionParameter::new(Some(vec![ParameterType::Integer, ParameterType::Character, ParameterType::Boolean])), InstructionParameter::new(Some(vec![ParameterType::Integer, ParameterType::Character, ParameterType::Boolean])), InstructionParameter::new(Some(vec![ParameterType::Integer, ParameterType::Character, ParameterType::Boolean])));
        push_description!(map, "shift_left",    ShiftLeft,      true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Integer, ParameterType::Character])), InstructionParameter::new(Some(vec![ParameterType::Integer, ParameterType::Character])));
        push_description!(map, "shift_right",   ShiftRight,     true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Integer, ParameterType::Character])), InstructionParameter::new(Some(vec![ParameterType::Integer, ParameterType::Character])));
        push_description!(map, "empty",         Empty,          true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Map, ParameterType::List, ParameterType::String])));
        push_description!(map, "flip",          Flip,           true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Container])));
        push_description!(map, "join",          Join,           true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::List])), InstructionParameter::new(Some(vec![ParameterType::Literal])));
        push_description!(map, "split",         Split,          true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Container])), InstructionParameter::new(None), InstructionParameter::new(Some(vec![ParameterType::Boolean])));
        push_description!(map, "uppercase",     Uppercase,      true,   false,  true,   InstructionParameter::new(Some(vec![ParameterType::Literal])));
        push_description!(map, "lowercase",     Lowercase,      true,   false,  true,   InstructionParameter::new(Some(vec![ParameterType::Literal])));
        push_description!(map, "insert",        Insert,         true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Container])), InstructionParameter::new(Some(vec![ParameterType::Selector])), InstructionParameter::new(None));
        push_description!(map, "overwrite",     Overwrite,      true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Container])), InstructionParameter::new(Some(vec![ParameterType::Selector])), InstructionParameter::new(None));
        push_description!(map, "move",          Move,           true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Container])), InstructionParameter::new(Some(vec![ParameterType::Selector])), InstructionParameter::new(Some(vec![ParameterType::Selector])));
        push_description!(map, "push",          Push,           true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Container])), InstructionParameter::new(None));
        push_description!(map, "append",        Append,         true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Container])), InstructionParameter::new(None));
        push_description!(map, "remove",        Remove,         true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Container])), InstructionParameter::new(Some(vec![ParameterType::Selector])));
        push_description!(map, "system",        System,         true,   false,  true,   InstructionParameter::new(Some(vec![ParameterType::String])), InstructionParameter::new(Some(vec![ParameterType::String])));
        push_description!(map, "silent",        Silent,         true,   false,  true,   InstructionParameter::new(Some(vec![ParameterType::String])), InstructionParameter::new(Some(vec![ParameterType::String])));
        push_description!(map, "keys",          Keys,           true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Map])));
        push_description!(map, "values",        Values,         true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Container])));
        push_description!(map, "pairs",         Pairs,          true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Container])));
        push_description!(map, "serialize",     Serialize,      true,   false,  false,  InstructionParameter::new(None));
        push_description!(map, "deserialize",   Deserialize,    true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::String])));
        push_description!(map, "read_file",     ReadFile,       true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::String])));
        push_description!(map, "write_file",    WriteFile,      true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::String])), InstructionParameter::new(Some(vec![ParameterType::String])));
        push_description!(map, "read_map",      ReadMap,        true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::String])));
        push_description!(map, "write_map",     WriteMap,       true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::String])), InstructionParameter::new(Some(vec![ParameterType::Map])));
        push_description!(map, "read_list",     ReadList,       true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::String])));
        push_description!(map, "write_list",    WriteList,      true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::String])), InstructionParameter::new(Some(vec![ParameterType::List])));
        push_description!(map, "modify",        Modify,         true,   false,  true,   InstructionParameter::new(Some(vec![ParameterType::Key, ParameterType::Path])), InstructionParameter::new(None), InstructionParameter::new(None));
        push_description!(map, "call",          Call,           true,   false,  true,   InstructionParameter::new(Some(vec![ParameterType::List])), InstructionParameter::new(None));
        push_description!(map, "call_list",     CallList,       true,   false,  true,   InstructionParameter::new(Some(vec![ParameterType::List])), InstructionParameter::new(Some(vec![ParameterType::List])));
        push_description!(map, "invoke",        Invoke,         true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Keyword])), InstructionParameter::new(Some(vec![ParameterType::List])));
        push_description!(map, "pass",          Pass,           true,   false,  true,   InstructionParameter::new(None), InstructionParameter::new(None));
        push_description!(map, "new_pass",      NewPass,        true,   false,  true,   InstructionParameter::new(Some(vec![ParameterType::Literal])), InstructionParameter::new(None), InstructionParameter::new(None));
        push_description!(map, "merge",         Merge,          true,   false,  true,   InstructionParameter::new(Some(vec![ParameterType::Container])), InstructionParameter::new(Some(vec![ParameterType::Container])), InstructionParameter::new(Some(vec![ParameterType::Container])));
        push_description!(map, "slice",         Slice,          true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Container])), InstructionParameter::new(Some(vec![ParameterType::Selector])), InstructionParameter::new(Some(vec![ParameterType::Selector])));
        push_description!(map, "index",         Index,          true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Container])), InstructionParameter::new(Some(vec![ParameterType::Selector]))); // make index variadic (?)
        push_description!(map, "resolve",       Resolve,        true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Selector, ParameterType::Path])));
        push_description!(map, "replace",       Replace,        true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Container])), InstructionParameter::new(None), InstructionParameter::new(None));
        push_description!(map, "position",      Position,       true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Container])), InstructionParameter::new(None));
        push_description!(map, "iterate",       Iterate,        false,  false,  false,  InstructionParameter::new(Some(vec![ParameterType::Container])));
        push_description!(map, "for",           For,            false,  false,  false,  InstructionParameter::new(Some(vec![ParameterType::Integer])), InstructionParameter::new(Some(vec![ParameterType::Integer])));
        push_description!(map, "if",            If,             false,  false,  true,   InstructionParameter::new(Some(vec![ParameterType::Keyword])), InstructionParameter::new(None));
        push_description!(map, "while",         While,          false,  true,   true,   InstructionParameter::new(Some(vec![ParameterType::Keyword])), InstructionParameter::new(None));
        push_description!(map, "else",          Else,           false,  true,   true,   InstructionParameter::new(None));
        push_description!(map, "end",           End,            false,  false,  true,   InstructionParameter::new(Some(vec![ParameterType::Keyword])));
        push_description!(map, "break",         Break,          false,  false,  true,   InstructionParameter::new(Some(vec![ParameterType::Keyword])));
        push_description!(map, "continue",      Continue,       false,  false,  true,   InstructionParameter::new(Some(vec![ParameterType::Keyword])));
        push_description!(map, "tokenize",      Tokenize,       true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Map])), InstructionParameter::new(Some(vec![ParameterType::String])), InstructionParameter::new(Some(vec![ParameterType::Literal])), InstructionParameter::new(Some(vec![ParameterType::Boolean])));
        push_description!(map, "parse",         Parse,          true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Map])), InstructionParameter::new(Some(vec![ParameterType::Map])), InstructionParameter::new(Some(vec![ParameterType::Map])));
        push_description!(map, "build",         Build,          true,   false,  false,  InstructionParameter::new(Some(vec![ParameterType::Map])), InstructionParameter::new(Some(vec![ParameterType::Map])));
        map
    };
}
