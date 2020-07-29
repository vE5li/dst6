use internal::*;
use debug::*;

use tokenize::Token;

pub struct StringTokenizer {
    delimiter:      (SharedString, SharedString),
    replace:        Vec<(SharedString, SharedString)>,
}

impl StringTokenizer {

    pub fn new(settings: &Data, character_stack: &mut CharacterStack, variant_registry: &mut VariantRegistry) -> Status<Self> {
        ensure!(settings.is_map(), ExpectedFound, expected_list!["map"], settings.clone());
        variant_registry.has_strings = true;
        let mut replace = Vec::new();

        let delimiter = index_field!(settings, "delimiter");
        let delimiter_list = unpack_list!(&delimiter);
        ensure!(delimiter_list.len() == 2, InvalidItemCount, integer!(2), integer!(delimiter_list.len() as i64));
        let start_delimiter = unpack_literal!(&delimiter_list[0]);
        let end_delimiter = unpack_literal!(&delimiter_list[1]);

        ensure!(!start_delimiter.is_empty(), EmptyLiteral);
        ensure!(!end_delimiter.is_empty(), EmptyLiteral);
        confirm!(character_stack.register_breaking(start_delimiter[0]));
        confirm!(character_stack.register_signature(start_delimiter.clone()));

        if let Some(replace_lookup) = confirm!(settings.index(&keyword!("replace"))) {
            ensure!(replace_lookup.is_map(), ExpectedFound, expected_list!["map"], replace_lookup);

            for (from, to) in confirm!(replace_lookup.pairs()).into_iter() {
                let from = unpack_literal!(&from);
                let to = unpack_literal!(&to);
                ensure!(!from.is_empty(), EmptyLiteral);
                push_by_length!(replace, from, to);
            }
        }

        return success!(Self {
            delimiter:      (start_delimiter, end_delimiter),
            replace:        replace,
        });
    }

    pub fn find(&self, character_stack: &mut CharacterStack, tokens: &mut Vec<Token>) -> Status<bool> {
        if character_stack.check_string(&self.delimiter.0) {
            let mut string = SharedString::new();
            'check: while !character_stack.check_string(&self.delimiter.1) {

                if character_stack.is_empty() {
                    let error = Error::UnterminatedToken(identifier!("string"));
                    tokens.push(Token::new(TokenType::Invalid(error), character_stack.final_positions()));
                    return success!(true);
                }

                for (from, to) in self.replace.iter() {
                    if character_stack.check_string(&from) {
                        string.push_str(to);
                        continue 'check;
                    }
                }

                string.push(character_stack.pop().unwrap());
            }

            tokens.push(Token::new(TokenType::String(string), character_stack.final_positions()));
            return success!(true);
        }

        return success!(false);
    }
}
