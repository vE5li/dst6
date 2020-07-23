mod partial;

use internal::*;
use debug::*;

use self::partial::*;

macro_rules! create {
    ($type:ident, $name:expr, $compiler:expr, $character_stack:expr, $variant_registry:expr) => (
        match guaranteed!($compiler.index(&keyword!("{}_tokenizer", $name))) {
            Some(settings_map) => Some(confirm!($type::new(&settings_map, $character_stack, $variant_registry))),
            None => None,
        }
    );
}

macro_rules! find {
    ($tokenizer:expr, $($arguments:tt)*) => (
        if let Some(tokenizer) = $tokenizer {
            if confirm!(tokenizer.find($($arguments)*)) {
                continue;
            }
        }
    );
}

pub fn tokenize(compiler: &Data, source_string: VectorString, source_file: Option<VectorString>, complete: bool) -> Status<(Vec<Token>, VariantRegistry, Vec<Note>)> {
    let mut character_stack = CharacterStack::new(source_string, source_file);
    let mut variant_registry = VariantRegistry::new();
    let mut token_stream = Vec::new();
    let mut notes = Vec::new();

    let comment_tokenizer = create!(CommentTokenizer, "comment", &compiler, &mut character_stack, &mut variant_registry);
    let number_tokenizer = create!(NumberTokenizer, "number", &compiler, &mut character_stack, &mut variant_registry);
    let string_tokenizer = create!(StringTokenizer, "string", &compiler, &mut character_stack, &mut variant_registry);
    let character_tokenizer = create!(CharacterTokenizer, "character", &compiler, &mut character_stack, &mut variant_registry);
    let operator_tokenizer = create!(OperatorTokenizer, "operator", &compiler, &mut character_stack, &mut variant_registry);
    let keyword_tokenizer = create!(KeywordTokenizer, "keyword", &compiler, &mut character_stack, &mut variant_registry);
    let identifier_tokenizer = create!(IdentifierTokenizer, "identifier", &compiler, &mut character_stack, &mut variant_registry);

    while !character_stack.is_empty() {
        let mut error = None;
        character_stack.start_positions();

        find!(&comment_tokenizer, &mut character_stack, &mut token_stream, &mut notes);
        find!(&number_tokenizer, &mut character_stack, &mut token_stream, &mut error);
        find!(&character_tokenizer, &mut character_stack, &mut token_stream);
        find!(&string_tokenizer, &mut character_stack, &mut token_stream);
        find!(&operator_tokenizer, &mut character_stack, &mut token_stream, complete);
        find!(&keyword_tokenizer, &mut character_stack, &mut token_stream, complete);
        find!(&identifier_tokenizer, &mut character_stack, &mut token_stream, complete, &mut error);

        let word = confirm!(character_stack.till_breaking());
        let positions = character_stack.final_positions();
        let error = error.unwrap_or(Error::UnregisteredCharacter(character!(word[0])));
        token_stream.push(Token::new(TokenType::Invalid(error), positions));
    }

    return success!((token_stream, variant_registry, notes));
}

pub fn call_tokenize(compiler: &Data, source_string: &Data, source_file: &Data, complete: &Data, root: &Data, build: &Data) -> Status<Data> {
    ensure!(source_file.is_string(), Message, string!("source file must be a string"));

    let unpacked_source_string = unpack_string!(source_string);
    let unpacked_source_file = (*source_file != identifier!("none")).then_some(extract_string!(source_file));
    let complete = unpack_boolean!(complete);
    let (token_stream, variant_registry, notes) = confirm!(tokenize(&compiler, unpacked_source_string, unpacked_source_file, complete));

    let mut return_map = Map::new();
    return_map.insert(identifier!("token_stream"), serialize_token_stream(token_stream, source_string, source_file, root, build));
    return_map.insert(identifier!("registry"), variant_registry.serialize());
    return_map.insert(identifier!("notes"), serialize_notes(notes));
    return success!(map!(return_map));
}

fn serialize_notes(notes: Vec<Note>) -> Data {
    return list!(notes.into_iter().map(|note| note.serialize()).collect());
}

fn serialize_token_stream(token_stream: Vec<Token>, source_string: &Data, source_file: &Data, root: &Data, build: &Data) -> Data {
    let token_list = token_stream.into_iter().map(|token| token.serialize(root, build)).collect();

    let mut entry_map = Map::new();
    entry_map.insert(identifier!("source"), source_string.clone());
    entry_map.insert(identifier!("file"), source_file.clone());
    entry_map.insert(identifier!("tokens"), list!(token_list));
    return map!(entry_map);
}
