use internal::*;
use debug::*;

use super::comma_seperated_list;

#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub enum Error {
    Tag(Data, Box<Error>),
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

    pub fn display(self, root: &Option<&Data>, build: &Data) -> SharedString {
        match self {
            Error::Tag(tag, error)                                 => return format_hook!(root, build, "tag", vector![tag, string!(String, error.display(root, build))], "{} -> {}", tag.serialize(), error.display(root, build)),
            Error::Message(message)                                => return format_hook!(root, build, "message", vector![message], "{}", extract_string!(&message)),
            Error::InvalidItemCount(specified, received)           => return format_hook!(root, build, "invalid_item_count", vector![specified, received], "{} items specified; found {}", extract_integer!(&specified), extract_integer!(&received)),
            Error::InvalidCondition(condition)                     => return format_hook!(root, build, "invalid_condition", vector![condition], "invalid condition {}", extract_keyword!(&condition)),
            Error::UnexpectedToken(token)                          => return format_hook!(root, build, "unexpected_token", vector![token], "unexpected token {}", token.serialize()), // DEBUG SERIALIZE
            Error::InvalidToken(token_type, token)                 => return format_hook!(root, build, "invalid_token", vector![token_type, token], "invalid {} {}", extract_identifier!(&token_type), extract_literal!(&token)),
            Error::InvalidTokenType(token_type)                    => return format_hook!(root, build, "invalid_token_type", vector![token_type], "invalid token type {}", extract_identifier!(&token_type)),
            Error::InvalidLocation(location)                       => return format_hook!(root, build, "invalid_location", vector![location], "invalid location {}", extract_keyword!(&location)),
            Error::Expected(expected)                              => return format_hook!(root, build, "expected", vector![expected], "expected {}", comma_seperated_list(&extract_list!(expected))),
            Error::ExpectedFound(expected, found)                  => return format_hook!(root, build, "expected_found", vector![expected, found], "expected {}; found {}", comma_seperated_list(&extract_list!(expected)), found.serialize()),
            Error::InvalidType(invalid_type)                       => return format_hook!(root, build, "invalid_type", vector![invalid_type], "invalid type {}", extract_identifier!(&invalid_type)),
            Error::InvalidPieceType(piece_type)                    => return format_hook!(root, build, "invalid_piece_type", vector![piece_type], "invalid piece type {}", extract_keyword!(&piece_type)),
            Error::UnregisteredCharacter(character)                => return format_hook!(root, build, "unregistered_character", vector![character], "unregistered character {}", extract_character!(&character)),
            Error::DuplicateSignature(signature)                   => return format_hook!(root, build, "duplicate_signature", vector![signature], "duplicate signature {}", extract_string!(&signature)),
            Error::DuplicateBreaking(character)                    => return format_hook!(root, build, "duplicate_breaking", vector![character], "duplicate definition of breaking character \'{}\'", extract_character!(&character)), // {:?}
            Error::DuplicateNonBreaking(character)                 => return format_hook!(root, build, "duplicate_non_breaking", vector![character], "duplicate definition of non breaking character \'{}\'", extract_character!(&character)), // {:?}
            Error::ExpectedIdentifierType(found)                   => return format_hook!(root, build, "expected_identifier_type", vector![found], "expected identifier type (possible values are identifier and type_identifier); found {}", extract_identifier!(&found)),
            Error::EmptyLiteral                                    => return format_hook!(root, build, "emtpy_literal", SharedVector::new(), "empty literal"),
            Error::InvalidCharacterLength(found)                   => return format_hook!(root, build, "invalid_character_length", vector![found], "character \'{}\' may only be one byte in length", extract_string!(&found)),
            Error::InvalidPathLength(found)                        => return format_hook!(root, build, "invalid_path_length", vector![found], "path {} needs at least 2 steps", found.serialize()),
            Error::NothingToParse                                  => return format_hook!(root, build, "nothing_to_parse", SharedVector::new(), "nothing to parse"),
            Error::NoPreviousReturn                                => return format_hook!(root, build, "no_previous_return", SharedVector::new(), "previous function did not return anything"),
            Error::InvalidVariadic(number)                         => return format_hook!(root, build, "invalid_variadic", vector![number], "parameter {} may not be variadic (only the last parameter may be variadic)", extract_integer!(&number)),
            Error::UnexpectedCompilerFunction(function)            => return format_hook!(root, build, "unexpected_compiler_function", vector![function], "unexpected compiler function {}", function.serialize()),
            Error::ExpectedCondition                               => return format_hook!(root, build, "expected_condition", SharedVector::new(), "expected condition"),
            Error::ExpectedConditionFound(found)                   => return format_hook!(root, build, "expected_condition_found", vector![found], "expected condition; found {}", found.serialize()), // DEBUG SERIALIZE
            Error::ExpectedParameter(number, expected)             => return format_hook!(root, build, "expected_parameter", vector![number, expected], "parameter {} expected {}", extract_integer!(&number), comma_seperated_list(&extract_list!(&expected))),
            Error::ExpectedParameterFound(number, expected, found) => return format_hook!(root, build, "expected_parameter_found", vector![number, expected, found], "parameter {} expected {}; found {}", extract_integer!(&number), comma_seperated_list(&extract_list!(&expected)), found.serialize()),
            Error::UnexpectedParameter(parameter)                  => return format_hook!(root, build, "unexpected_parameter", vector![parameter], "unexpected parameter {}", parameter.serialize()), // DEBUG SERIALIZE (?)
            Error::UnterminatedEscapeSequence                      => return format_hook!(root, build, "unterminated_escape_sequence", SharedVector::new(), "unterminated escape sequence"),
            Error::InvalidEscapeSequence(sequence)                 => return format_hook!(root, build, "invalid_escape_sequence", vector![sequence], "invalid escape sequence {}", sequence.serialize()),
            Error::ExpectedReturn(expected)                        => return format_hook!(root, build, "expected_return", vector![expected], "expected function to return {}", comma_seperated_list(&extract_list!(&expected))),
            Error::ExpectedReturnFound(expected, found)            => return format_hook!(root, build, "expected_return_found", vector![expected], "expected function to return {}; found {}", comma_seperated_list(&extract_list!(&expected)), found.serialize()),
            Error::InexplicitOverwrite(selector, previous)         => return format_hook!(root, build, "inexplicit_overwrite", vector![selector, previous], "{} has previous value {}", selector.serialize(), previous.serialize()),
            Error::MissingEntry(key)                               => return format_hook!(root, build, "missing_entry", vector![key], "missing entry {}", key.serialize()),
            Error::UnclosedScope                                   => return format_hook!(root, build, "unclosed_scope", SharedVector::new(), "unclosed scope"),
            Error::ExpectedLocation                                => return format_hook!(root, build, "expected_location", SharedVector::new(), "expected location"),
            Error::ExpectedLocationFound(found)                    => return format_hook!(root, build, "expected_location_found", vector![found], "expected location; found {}", found.serialize()), // DEBUG SERIALIZE
            Error::ExpectedImmediate                               => return format_hook!(root, build, "expected_immediate", SharedVector::new(), "expected immediate"),
            Error::UnexpectedImmediate(found)                      => return format_hook!(root, build, "unexpected_immediate", vector![found], "unexpected immediate {}", found.serialize()), // DEBUG SERIALIZE
            Error::InvalidCompilerFunction(function)               => return format_hook!(root, build, "invalid_compiler_function", vector![function], "invalid compiler function {}", function.serialize()),
            Error::MissingFile(filename)                           => return format_hook!(root, build, "missing_file", vector![filename], "missing file {}", extract_string!(&filename)), // SERIALIZE (?)
            Error::UnterminatedToken(token_type)                   => return format_hook!(root, build, "unterminated_token", vector![token_type], "unterminated token {}", extract_identifier!(&token_type)),
            Error::ExpectedBooleanFound(found)                     => return format_hook!(root, build, "expected_boolean_found", vector![found], "expected boolean (possible values are true and false); found {}", found.serialize()), // DEBUG SERIALIZE
            Error::IndexOutOfBounds(selector, biggest)             => return format_hook!(root, build, "index_out_of_bounds", vector![selector, biggest], "smallest index is 1, biggest is {}; found {}", extract_integer!(&biggest), extract_integer!(&selector)),
            Error::InvalidPrefix(prefix)                           => return format_hook!(root, build, "invalid_prefix", vector![prefix], "invalid prefix {}", prefix.serialize()), // DEBUG SERIALIZE
            Error::InvalidSuffix(suffix)                           => return format_hook!(root, build, "invalid_suffix", vector![suffix], "invalid suffix {}", suffix.serialize()), // DEBUG SERIALIZE
            Error::InvalidNumber(system)                           => return format_hook!(root, build, "invalid_number", vector![system], "invalid {} number", extract_identifier!(&system)),
            Error::ExpectedWord                                    => return format_hook!(root, build, "expected_word", SharedVector::new(), "expected word"),
            Error::ExpectedWordFound(found)                        => return format_hook!(root, build, "expected_word_found", vector![found], "expected word; found {}", found.serialize()), // DEBUG SERIALIZE (?)
            Error::InvalidNumberSystem(system)                     => return format_hook!(root, build, "invalid_number_system", vector![system], "invalid number system {}", extract_identifier!(system)),
            Error::AmbiguousIdentifier(identifier)                 => return format_hook!(root, build, "ambiguous_identifier", vector![identifier], "ambiguous identifier {}; could be identifier and type identifier", extract_identifier!(identifier)),
        }
    }
}
