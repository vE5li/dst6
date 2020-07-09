mod partial;

use self::partial::*;
use internal::*;

macro_rules! partial {
    ($tokenizer:expr, $($arguments:tt)*) => (
        if let Some(tokenizer) = $tokenizer {
            if confirm!(tokenizer.find($($arguments)*)) {
                continue;
            }
        }
    );
}

macro_rules! create_tokenizer {
    ($type:ident, $name:expr, $compiler:expr, $character_stack:expr, $variant_registry:expr) => ({
        let settings_map = guaranteed!($compiler.index(&keyword!(str, "{}_tokenizer", $name)));
        match settings_map {
            Some(settings_map) => Some(confirm!($type::new(&settings_map, $character_stack, $variant_registry))),
            None => None,
        }
    });
}

pub fn tokenize(compiler: &Data, source_string: VectorString, source_file: Option<VectorString>, current_pass: &Option<VectorString>, build: &Data, context: &Data, complete: bool) -> Status<(Vec<Token>, VariantRegistry)> {

    let mut character_stack = CharacterStack::new(source_string, source_file);
    let mut variant_registry = VariantRegistry::new();
    let mut token_stream = Vec::new();

    let comment_tokenizer = create_tokenizer!(CommentTokenizer, "comment", &compiler, &mut character_stack, &mut variant_registry);
    let number_tokenizer = create_tokenizer!(NumberTokenizer, "number", &compiler, &mut character_stack, &mut variant_registry);
    let string_tokenizer = create_tokenizer!(StringTokenizer, "string", &compiler, &mut character_stack, &mut variant_registry);
    let character_tokenizer = create_tokenizer!(CharacterTokenizer, "character", &compiler, &mut character_stack, &mut variant_registry);
    let operator_tokenizer = create_tokenizer!(OperatorTokenizer, "operator", &compiler, &mut character_stack, &mut variant_registry);
    let keyword_tokenizer = create_tokenizer!(KeywordTokenizer, "keyword", &compiler, &mut character_stack, &mut variant_registry);
    let identifier_tokenizer = create_tokenizer!(IdentifierTokenizer, "identifier", &compiler, &mut character_stack, &mut variant_registry);

    while !character_stack.is_empty() {
        let mut error = None;
        character_stack.start_positions();
        partial!(&comment_tokenizer, &mut character_stack, &mut token_stream, complete, current_pass, compiler, build, context);
        partial!(&number_tokenizer, &mut character_stack, &mut token_stream, complete, &mut error);
        partial!(&character_tokenizer, &mut character_stack, &mut token_stream, complete);
        partial!(&string_tokenizer, &mut character_stack, &mut token_stream, complete);
        partial!(&operator_tokenizer, &mut character_stack, &mut token_stream, complete);
        partial!(&keyword_tokenizer, &mut character_stack, &mut token_stream, complete);
        partial!(&identifier_tokenizer, &mut character_stack, &mut token_stream, complete, &mut error);

        if let Some(error) = error {
            let positions = character_stack.final_positions();
            token_stream.push(Token::new(TokenType::Invalid(error), positions));
        } else {
            let positions = character_stack.final_positions();
            let error = Error::UnregisteredCharacter(character!(character_stack.pop().unwrap())); //, character_stack.final_positions()),
            token_stream.push(Token::new(TokenType::Invalid(error), positions));
        }
    }

    return success!((token_stream, variant_registry));
}
