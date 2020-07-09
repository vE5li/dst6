use super::{ comma_seperated_list, expanded_list };
use internal::*;

#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub enum Error {
    Tag(Data, Box<Error>),
    Compiler(Vector<Error>),
    Tokenizer(Vector<Error>),
    Parser(Vector<Error>),
    Builder(Vector<Error>),
    Execute(Data, Box<Error>),
    Message(Data),
    InvalidItemCount(Data, Data),
    InvalidCondition(Data),
    UnexpectedToken(Data),
    InvalidToken(Data, Data),
    InvalidTokenType(Data),
    InvalidLocation(Data),
    Expected(Data),
    ExpectedFound(Data, Data),
    InvalidPieceType(Data),
    UnregisteredCharacter(Data),
    DuplicateSignature(Data),
    DuplicateBreaking(Data),
    DuplicateNonBreaking(Data),
    ExpectedIdentifierType(Data),
    EmptyLiteral,
    InvalidCharacterLength(Data),
    InvalidPathLength(Data),
    ExpectedReturn(Data),
    ExpectedReturnFound(Data, Data),
    ExpectedParameter(Data, Data),
    ExpectedParameterFound(Data, Data, Data),
    ExpectedCondition,
    ExpectedConditionFound(Data),
    InexplicitOverwrite(Data, Data),
    MissingEntry(Data),
    InvalidType(Data),
    InvalidVariadic(Data),
    UnexpectedParameter(Data),
    UnclosedScope,
    InvalidCompilerFunction(Data),
    UnexpectedCompilerFunction(Data),
    ExpectedLocation,
    ExpectedLocationFound(Data),
    ExpectedImmediate,
    UnexpectedImmediate(Data),
    MissingFile(Data),
    UnterminatedToken(Data),
    NoPreviousReturn,
    ExpectedBooleanFound(Data),
    IndexOutOfBounds(Data, Data),
    NothingToParse,
    UnterminatedEscapeSequence,
    InvalidEscapeSequence(Data),
    InvalidPrefix(Data),
    InvalidSuffix(Data),
    InvalidNumber(Data), // add actual number (?)
    ExpectedWord,
    ExpectedWordFound(Data),
    InvalidNumberSystem(Data),
    AmbiguousIdentifier(Data),
}

impl Error {

    pub fn display(self, root: &Option<&Data>, build: &Data, context: &Data) -> VectorString {
        match self {
            Error::Tag(tag, error)                                 => return format_hook!(root, build, context, "tag", vector![tag, string!(error.display(root, build, context))], "{} -> {}", tag.serialize(), error.display(root, build, context)),
            Error::Compiler(errors)                                => return format_hook!(root, build, context, "compiler", vector![list!(errors.into_iter().map(|error| string!(error.display(root, build, context))).collect())], "{}", expanded_list(errors)),
            Error::Tokenizer(errors)                               => return format_hook!(root, build, context, "tokenizer", vector![list!(errors.into_iter().map(|error| string!(error.display(root, build, context))).collect())], "{}", expanded_list(errors)),
            Error::Parser(errors)                                  => return format_hook!(root, build, context, "parser", vector![list!(errors.into_iter().map(|error| string!(error.display(root, build, context))).collect())], "{}", expanded_list(errors)),
            Error::Builder(errors)                                 => return format_hook!(root, build, context, "builder", vector![list!(errors.into_iter().map(|error| string!(error.display(root, build, context))).collect())], "{}", expanded_list(errors)),
            Error::Execute(function, error)                        => return format_hook!(root, build, context, "call", vector![function, string!(error.display(root, build, context))], "{} :: {}", function.serialize(), error.display(root, build, context)), // DEBUG SERIALIZE
            Error::Message(message)                                => return format_hook!(root, build, context, "message", vector![message], "{}", extract_string!(&message)),
            Error::InvalidItemCount(specified, received)           => return format_hook!(root, build, context, "invalid_item_count", vector![specified, received], "{} items specified; found {}", extract_integer!(&specified), extract_integer!(&received)),
            Error::InvalidCondition(condition)                     => return format_hook!(root, build, context, "invalid_condition", vector![condition], "invalid condition {}", extract_keyword!(&condition)),
            Error::UnexpectedToken(token)                          => return format_hook!(root, build, context, "unexpected_token", vector![token], "unexpected token {}", token.serialize()), // DEBUG SERIALIZE
            Error::InvalidToken(token_type, token)                 => return format_hook!(root, build, context, "invalid_token", vector![token_type, token], "invalid {} {}", extract_identifier!(&token_type), extract_literal!(&token)),
            Error::InvalidTokenType(token_type)                    => return format_hook!(root, build, context, "invalid_token_type", vector![token_type], "invalid token type {}", extract_identifier!(&token_type)),
            Error::InvalidLocation(location)                       => return format_hook!(root, build, context, "invalid_location", vector![location], "invalid location {}", extract_keyword!(&location)),
            Error::Expected(expected)                              => return format_hook!(root, build, context, "expected", vector![expected], "expected {}", comma_seperated_list(&extract_list!(expected))),
            Error::ExpectedFound(expected, found)                  => return format_hook!(root, build, context, "expected_found", vector![expected, found], "expected {}; found {}", comma_seperated_list(&extract_list!(expected)), found.serialize()),
            Error::InvalidType(invalid_type)                       => return format_hook!(root, build, context, "invalid_type", vector![invalid_type], "invalid type {}", extract_identifier!(&invalid_type)),
            Error::InvalidPieceType(piece_type)                    => return format_hook!(root, build, context, "invalid_piece_type", vector![piece_type], "invalid piece type {}", extract_keyword!(&piece_type)),
            Error::UnregisteredCharacter(character)                => return format_hook!(root, build, context, "unregistered_character", vector![character], "unregistered character {}", extract_character!(&character)),
            Error::DuplicateSignature(signature)                   => return format_hook!(root, build, context, "duplicate_signature", vector![signature], "duplicate signature {}", extract_string!(&signature)),
            Error::DuplicateBreaking(character)                    => return format_hook!(root, build, context, "duplicate_breaking", vector![character], "duplicate definition of breaking character \'{}\'", extract_character!(&character)), // {:?}
            Error::DuplicateNonBreaking(character)                 => return format_hook!(root, build, context, "duplicate_non_breaking", vector![character], "duplicate definition of non breaking character \'{}\'", extract_character!(&character)), // {:?}
            Error::ExpectedIdentifierType(found)                   => return format_hook!(root, build, context, "expected_identifier_type", vector![found], "expected identifier type (possible values are identifier and type_identifier); found {}", extract_identifier!(&found)),
            Error::EmptyLiteral                                    => return format_hook!(root, build, context, "emtpy_literal", Vector::new(), "empty literal"),
            Error::InvalidCharacterLength(found)                   => return format_hook!(root, build, context, "invalid_character_length", vector![found], "character \'{}\' may only be one byte in length", extract_string!(&found)),
            Error::InvalidPathLength(found)                        => return format_hook!(root, build, context, "invalid_path_length", vector![found], "path {} needs at least 2 steps", found.serialize()),
            Error::NothingToParse                                  => return format_hook!(root, build, context, "nothing_to_parse", Vector::new(), "nothing to parse"),
            Error::NoPreviousReturn                                => return format_hook!(root, build, context, "no_previous_return", Vector::new(), "previous function did not return anything"),
            Error::InvalidVariadic(number)                         => return format_hook!(root, build, context, "invalid_variadic", vector![number], "parameter {} may not be variadic (only the last parameter may be variadic)", extract_integer!(&number)),
            Error::UnexpectedCompilerFunction(function)            => return format_hook!(root, build, context, "unexpected_compiler_function", vector![function], "unexpected compiler function {}", function.serialize()),
            Error::ExpectedCondition                               => return format_hook!(root, build, context, "expected_condition", Vector::new(), "expected condition"),
            Error::ExpectedConditionFound(found)                   => return format_hook!(root, build, context, "expected_condition_found", vector![found], "expected condition; found {}", found.serialize()), // DEBUG SERIALIZE
            Error::ExpectedParameter(number, expected)             => return format_hook!(root, build, context, "expected_parameter", vector![number, expected], "parameter {} expected {}", extract_integer!(&number), comma_seperated_list(&extract_list!(&expected))),
            Error::ExpectedParameterFound(number, expected, found) => return format_hook!(root, build, context, "expected_parameter_found", vector![number, expected, found], "parameter {} expected {}; found {}", extract_integer!(&number), comma_seperated_list(&extract_list!(&expected)), found.serialize()),
            Error::UnexpectedParameter(parameter)                  => return format_hook!(root, build, context, "unexpected_parameter", vector![parameter], "unexpected parameter {}", parameter.serialize()), // DEBUG SERIALIZE (?)
            Error::UnterminatedEscapeSequence                      => return format_hook!(root, build, context, "unterminated_escape_sequence", Vector::new(), "unterminated escape sequence"),
            Error::InvalidEscapeSequence(sequence)                 => return format_hook!(root, build, context, "invalid_escape_sequence", vector![sequence], "invalid escape sequence {}", sequence.serialize()),
            Error::ExpectedReturn(expected)                        => return format_hook!(root, build, context, "expected_return", vector![expected], "expected function to return {}", comma_seperated_list(&extract_list!(&expected))),
            Error::ExpectedReturnFound(expected, found)            => return format_hook!(root, build, context, "expected_return_found", vector![expected], "expected function to return {}; found {}", comma_seperated_list(&extract_list!(&expected)), found.serialize()),
            Error::InexplicitOverwrite(selector, previous)         => return format_hook!(root, build, context, "inexplicit_overwrite", vector![selector, previous], "{} has previous value {}", selector.serialize(), previous.serialize()),
            Error::MissingEntry(key)                               => return format_hook!(root, build, context, "missing_entry", vector![key], "missing entry {}", key.serialize()),
            Error::UnclosedScope                                   => return format_hook!(root, build, context, "unclosed_scope", Vector::new(), "unclosed scope"),
            Error::ExpectedLocation                                => return format_hook!(root, build, context, "expected_location", Vector::new(), "expected location"),
            Error::ExpectedLocationFound(found)                    => return format_hook!(root, build, context, "expected_location_found", vector![found], "expected location; found {}", found.serialize()), // DEBUG SERIALIZE
            Error::ExpectedImmediate                               => return format_hook!(root, build, context, "expected_immediate", Vector::new(), "expected immediate"),
            Error::UnexpectedImmediate(found)                      => return format_hook!(root, build, context, "unexpected_immediate", vector![found], "unexpected immediate {}", found.serialize()), // DEBUG SERIALIZE
            Error::InvalidCompilerFunction(function)               => return format_hook!(root, build, context, "invalid_compiler_function", vector![function], "invalid compiler function {}", function.serialize()),
            Error::MissingFile(filename)                           => return format_hook!(root, build, context, "missing_file", vector![filename], "missing file {}", extract_string!(&filename)), // SERIALIZE (?)
            Error::UnterminatedToken(token_type)                   => return format_hook!(root, build, context, "unterminated_token", vector![token_type], "unterminated token {}", extract_identifier!(&token_type)),
            Error::ExpectedBooleanFound(found)                     => return format_hook!(root, build, context, "expected_boolean_found", vector![found], "expected boolean (possible values are true and false); found {}", found.serialize()), // DEBUG SERIALIZE
            Error::IndexOutOfBounds(selector, biggest)             => return format_hook!(root, build, context, "index_out_of_bounds", vector![selector, biggest], "smallest index is 1, biggest is {}; found {}", extract_integer!(&biggest), extract_integer!(&selector)),
            Error::InvalidPrefix(prefix)                           => return format_hook!(root, build, context, "invalid_prefix", vector![prefix], "invalid prefix {}", prefix.serialize()), // DEBUG SERIALIZE
            Error::InvalidSuffix(suffix)                           => return format_hook!(root, build, context, "invalid_suffix", vector![suffix], "invalid suffix {}", suffix.serialize()), // DEBUG SERIALIZE
            Error::InvalidNumber(system)                           => return format_hook!(root, build, context, "invalid_number", vector![system], "invalid {} number", extract_identifier!(&system)),
            Error::ExpectedWord                                    => return format_hook!(root, build, context, "expected_word", Vector::new(), "expected word"),
            Error::ExpectedWordFound(found)                        => return format_hook!(root, build, context, "expected_word_found", vector![found], "expected word; found {}", found.serialize()), // DEBUG SERIALIZE (?)
            Error::InvalidNumberSystem(system)                     => return format_hook!(root, build, context, "invalid_number_system", vector![system], "invalid number system {}", extract_identifier!(system)),
            Error::AmbiguousIdentifier(identifier)                 => return format_hook!(root, build, context, "ambiguous_identifier", vector![identifier], "ambiguous identifier {}; could be identifier and type identifier", extract_identifier!(identifier)),
        }
    }
}
