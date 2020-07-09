use internal::*;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Comment(VectorString),
    Keyword(VectorString),
    Operator(VectorString),
    Identifier(VectorString),
    TypeIdentifier(VectorString),
    String(VectorString),
    Character(Character),
    Integer(i64),
    Float(f64),
    Invalid(Error),
    Ignored,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub position: Vec<Position>,
}

impl Token {

    pub fn new(token_type: TokenType, position: Vec<Position>) -> Self {
        Self {
            token_type: token_type,
            position: position,
        }
    }

    pub fn parsable(&self) -> bool {
        match &self.token_type {
            TokenType::Comment(..) => false,
            TokenType::Invalid(..) => panic!(),
            TokenType::Ignored => panic!(),
            _ => true,
        }
    }

    pub fn to_location(&self) -> Data {
        match &self.token_type {
            TokenType::Comment(..) => panic!(),
            TokenType::Operator(operator) => return Data::Identifier(format_vector!("operator:{}", operator)),
            TokenType::Keyword(keyword) => return Data::Identifier(format_vector!("keyword:{}", keyword)),
            TokenType::Identifier(..) => return identifier!(str, "identifier"),
            TokenType::TypeIdentifier(..) => return identifier!(str, "type_identifier"),
            TokenType::Character(..) => return identifier!(str, "character"),
            TokenType::String(..) => return identifier!(str, "string"),
            TokenType::Integer(..) => return identifier!(str, "integer"),
            TokenType::Float(..) => return identifier!(str, "float"),
            TokenType::Invalid(..) => panic!(),
            TokenType::Ignored => panic!(),
        }
    }
}
