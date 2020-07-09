use internal::*;
use tokenize::Token;

pub struct KeywordTokenizer {
    rules:      Rules,
}

impl KeywordTokenizer {

    pub fn new(settings: &Data, character_stack: &mut CharacterStack, variant_registry: &mut VariantRegistry) -> Status<Self> {
        ensure!(settings.is_map(), ExpectedFound, expected_list!["map"], settings.clone());
        let mut rules = Rules::new();

        if let Some(translate_lookup) = confirm!(settings.index(&keyword!(str, "translate"))) {
            ensure!(translate_lookup.is_map(), ExpectedFound, expected_list!["map"], translate_lookup);
            for (from, to) in confirm!(translate_lookup.pairs()).into_iter() {
                let from = unpack_identifier!(&from);
                let to = unpack_identifier!(&to);
                variant_registry.register_keyword(to.clone());
                confirm!(character_stack.register_pure(&from));
                confirm!(rules.add(from, Action::Map(to)));
            }
        }

        if let Some(invalid_list) = confirm!(settings.index(&keyword!(str, "invalid"))) {
            for keyword in unpack_list!(&invalid_list).into_iter() {
                let keyword = unpack_identifier!(&keyword);
                confirm!(character_stack.register_pure(&keyword));
                confirm!(rules.add(keyword, Action::Invalid));
            }
        }

        if let Some(ignored_list) = confirm!(settings.index(&keyword!(str, "ignored"))) {
            for keyword in unpack_list!(&ignored_list).into_iter() {
                let keyword = unpack_identifier!(&keyword);
                confirm!(character_stack.register_pure(&keyword));
                confirm!(rules.add(keyword, Action::Ignored));
            }
        }

        return success!(Self {
            rules:          rules,
        });
    }

    pub fn find(&self, character_stack: &mut CharacterStack, tokens: &mut Vec<Token>, complete: bool) -> Status<bool> {
        character_stack.save();

        let word = confirm!(character_stack.till_breaking());
        if !character_stack.is_pure(&word) {
            character_stack.restore();
            return success!(false);
        }

        if let Some((matched, action)) = self.rules.check_word(&word) {
            character_stack.drop();

            match action {

                Action::Map(keyword) => {
                    tokens.push(Token::new(TokenType::Keyword(keyword), character_stack.final_positions()));
                    return success!(true);
                },

                Action::Invalid => {
                    let error = Error::InvalidToken(identifier!(str, "keyword"), string!(matched));
                    tokens.push(Token::new(TokenType::Invalid(error), character_stack.final_positions()));
                    return success!(true);
                },

                Action::Ignored => {
                    if complete {
                        tokens.push(Token::new(TokenType::Ignored, character_stack.final_positions()));
                    }
                    return success!(true);
                },
            }
        }

        character_stack.restore();
        return success!(false);
    }
}
