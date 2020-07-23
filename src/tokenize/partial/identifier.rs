use internal::*;
use debug::*;

use tokenize::Token;

pub struct IdentifierTokenizer {
    rules:      Rules,
    doubles:    Vec<VectorString>,
}

impl IdentifierTokenizer {

    pub fn new(settings: &Data, character_stack: &mut CharacterStack, variant_registry: &mut VariantRegistry) -> Status<Self> {
        ensure!(settings.is_map(), ExpectedFound, expected_list!["map"], settings.clone());
        let mut rules = Rules::new();
        let mut doubles = Vec::new();

        if let Some(prefix_list) = confirm!(settings.index(&keyword!("prefix"))) {
            for prefix in unpack_list!(&prefix_list).into_iter() {
                let prefix = unpack_identifier!(&prefix);
                confirm!(character_stack.register_pure(&prefix));
                confirm!(rules.add(prefix, Action::Map(VectorString::from("identifier"))));
            }
        }

        if let Some(type_prefix_list) = confirm!(settings.index(&keyword!("type_prefix"))) {
            for type_prefix in unpack_list!(&type_prefix_list).into_iter() {
                let type_prefix = unpack_identifier!(&type_prefix);
                confirm!(character_stack.register_pure(&type_prefix));

                match rules.has_mapping_to(&type_prefix, "identifier") {
                    true => doubles.push(type_prefix),
                    false => confirm!(rules.add(type_prefix, Action::Map(VectorString::from("type_identifier")))),
                }
            }
        }

        if let Some(invalid_identifiers) = confirm!(settings.index(&keyword!("invalid"))) {
            for identifier in unpack_list!(&invalid_identifiers).into_iter() {
                let identifier = unpack_identifier!(&identifier);
                confirm!(character_stack.register_pure(&identifier));
                confirm!(rules.add(identifier, Action::Invalid));
            }
        }

        if let Some(ignored_identifiers) = confirm!(settings.index(&keyword!("ignored"))) {
            for identifier in unpack_list!(&ignored_identifiers).into_iter() {
                let identifier = unpack_identifier!(&identifier);
                confirm!(character_stack.register_pure(&identifier));
                confirm!(rules.add(identifier, Action::Ignored));
            }
        }

        variant_registry.set_rules(rules.clone());

        return success!(Self {
            rules:      rules,
            doubles:    doubles,
        });
    }

    pub fn find(&self, character_stack: &mut CharacterStack, tokens: &mut Vec<Token>, complete: bool, error: &mut Option<Error>) -> Status<bool> {
        character_stack.save();
        let word = confirm!(character_stack.till_breaking());

        if !character_stack.is_pure(&word) {
            character_stack.restore();
            return success!(false);
        }

        let mut buffer = word.clone();
        loop {
            if let Some((matched, action)) = self.rules.check_prefix(&buffer) {
                match action {

                    Action::Map(variant) => {
                        match variant.printable().as_str() {

                            "identifier" => {
                                if self.doubles.contains(&matched) {
                                    buffer = buffer.chars().skip(matched.len()).cloned().collect();
                                    continue;
                                }
                                tokens.push(Token::new(TokenType::Identifier(word), character_stack.final_positions()));
                                character_stack.drop();
                                return success!(true);
                            },

                            "type_identifier" => {
                                if self.doubles.contains(&matched) {
                                    buffer = buffer.chars().skip(matched.len()).cloned().collect();
                                    continue;
                                }
                                tokens.push(Token::new(TokenType::TypeIdentifier(word), character_stack.final_positions()));
                                character_stack.drop();
                                return success!(true);
                            },

                            _invalid => panic!(),
                        }
                    },

                    Action::Invalid => {
                        let error = Error::InvalidToken(identifier!("identifier"), string!(String, matched));
                        tokens.push(Token::new(TokenType::Invalid(error), character_stack.final_positions()));
                        character_stack.drop();
                        return success!(true);
                    },

                    Action::Ignored => {
                        if complete {
                            tokens.push(Token::new(TokenType::Ignored, character_stack.final_positions()));
                        }
                        character_stack.drop();
                        return success!(true);
                    },
                }
            } else {
                if word != buffer {
                    *error = Some(Error::AmbiguousIdentifier(string!(String, buffer)));
                    character_stack.drop();
                    return success!(true);
                } else {
                    character_stack.restore();
                    return success!(false);
                }
            }
        }
    }
}
