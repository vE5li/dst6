use internal::*;
use debug::*;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Comment(SharedString),
    Keyword(SharedString),
    Operator(SharedString),
    Identifier(SharedString),
    TypeIdentifier(SharedString),
    String(SharedString),
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

    pub fn serialize_position(&self) -> Data {
        return list!(self.position.iter().map(|position| position.serialize_partial()).collect());
    }

    pub fn serialize(self, root: &Data, build: &Data) -> Data {
        let serialized_positions = self.serialize_position();
        match self.token_type {
            TokenType::Comment(comment) => return list!(vector![keyword!("comment"), string!(String, comment), serialized_positions]),
            TokenType::Operator(operator) => return list!(vector![keyword!("operator"), string!(String, operator), serialized_positions]),
            TokenType::Keyword(keyword) => return list!(vector![keyword!("keyword"), identifier!(String, keyword), serialized_positions]),
            TokenType::Identifier(identifier) => return list!(vector![keyword!("identifier"), identifier!(String, identifier), serialized_positions]),
            TokenType::TypeIdentifier(type_identifier) => return list!(vector![keyword!("type_identifier"), identifier!(String, type_identifier), serialized_positions]),
            TokenType::Character(character) => return list!(vector![keyword!("character"), character!(character), serialized_positions]),
            TokenType::String(string) => return list!(vector![keyword!("string"), string!(String, string), serialized_positions]),
            TokenType::Integer(integer) => return list!(vector![keyword!("integer"), integer!(integer), serialized_positions]),
            TokenType::Float(float) => return list!(vector![keyword!("float"), float!(float), serialized_positions]),
            TokenType::Invalid(error) => return list!(vector![keyword!("invalid"), string!(String, error.display(&Some(root), build)), serialized_positions]),
            TokenType::Ignored => return list!(vector![keyword!("ignored"), serialized_positions]),
        };
    }

    pub fn deserialize(serialized: &Data, file: &Option<SharedString>, source: &SharedString) -> Status<Self> {

        let mut source_list = unpack_list!(serialized);
        let token_type = unpack_keyword!(&source_list.remove(0));
        let token_type = match token_type.serialize().as_str() {
            "comment" => TokenType::Comment(unpack_string!(&source_list.remove(0))),
            "operator" => TokenType::Operator(unpack_string!(&source_list.remove(0))),
            "keyword" => TokenType::Keyword(unpack_identifier!(&source_list.remove(0))),
            "identifier" => TokenType::Identifier(unpack_identifier!(&source_list.remove(0))),
            "type_identifier" => TokenType::TypeIdentifier(unpack_identifier!(&source_list.remove(0))),
            "character" => TokenType::Character(unpack_character!(&source_list.remove(0))),
            "string" => TokenType::String(unpack_string!(&source_list.remove(0))),
            "integer" => TokenType::Integer(unpack_integer!(&source_list.remove(0))),
            "float" => TokenType::Float(unpack_float!(&source_list.remove(0))),
            "invalid" => TokenType::Invalid(Error::Message(source_list.remove(0))),
            "ignored" => TokenType::Ignored,
            invalid => return error!(string!("invalid token type {}", invalid)),
        };

        let mut positions = Vec::new();
        for position in unpack_list!(&source_list.remove(0)).iter() {
            positions.push(confirm!(Position::deserialize_partial(position, file, source)));
        }

        return success!(Self::new(token_type, positions));
    }

    pub fn parsable(&self) -> bool {
        match &self.token_type {
            TokenType::Invalid(..) => panic!("cannot parse invalid tokens"),
            TokenType::Comment(..) => false,
            TokenType::Ignored => false,
            _other => true,
        }
    }

    pub fn to_location(&self) -> Data {
        match &self.token_type {
            TokenType::Comment(..) => panic!(),
            TokenType::Operator(operator) => return Data::Identifier(format_shared!("operator:{}", operator)),
            TokenType::Keyword(keyword) => return Data::Identifier(format_shared!("keyword:{}", keyword)),
            TokenType::Identifier(..) => return identifier!("identifier"),
            TokenType::TypeIdentifier(..) => return identifier!("type_identifier"),
            TokenType::Character(..) => return identifier!("character"),
            TokenType::String(..) => return identifier!("string"),
            TokenType::Integer(..) => return identifier!("integer"),
            TokenType::Float(..) => return identifier!("float"),
            TokenType::Invalid(..) => panic!(),
            TokenType::Ignored => panic!(),
        }
    }
}
