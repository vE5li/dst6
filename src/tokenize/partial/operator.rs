use internal::*;
use debug::*;

use tokenize::Token;

pub struct OperatorTokenizer {
    rules:      Rules,
}

impl OperatorTokenizer {

    pub fn new(settings: &Data, character_stack: &mut CharacterStack, variant_registry: &mut VariantRegistry) -> Status<Self> {
        ensure!(settings.is_map(), ExpectedFound, expected_list!["map"], settings.clone());
        let mut rules = Rules::new();

        if let Some(translate_lookup) = confirm!(settings.index(&keyword!("translate"))) {
            ensure!(translate_lookup.is_map(), ExpectedFound, expected_list!["map"], translate_lookup.clone());
            
            for (from, to) in confirm!(translate_lookup.pairs()).into_iter() {
                let from = unpack_literal!(&from);
                let to = unpack_identifier!(&to);
                ensure!(!from.is_empty(), EmptyLiteral);
                variant_registry.register_operator(to.clone());
                confirm!(character_stack.register_breaking(from[0]));
                confirm!(character_stack.register_signature(from.clone()));
                confirm!(rules.add(from, Action::Map(to)));
            }
        }

        if let Some(invalid_operators) = confirm!(settings.index(&keyword!("invalid"))) {
            for operator in unpack_list!(&invalid_operators).into_iter() {
                let operator = unpack_literal!(&operator);
                ensure!(!operator.is_empty(), EmptyLiteral);
                confirm!(character_stack.register_breaking(operator[0]));
                confirm!(character_stack.register_signature(operator.clone()));
                confirm!(rules.add(operator, Action::Invalid));
            }
        }

        if let Some(ignored_operators) = confirm!(settings.index(&keyword!("ignored"))) {
            for operator in unpack_list!(&ignored_operators).into_iter() {
                let operator = unpack_literal!(&operator);
                ensure!(!operator.is_empty(), EmptyLiteral);
                confirm!(character_stack.register_breaking(operator[0]));
                confirm!(character_stack.register_signature(operator.clone()));
                confirm!(rules.add(operator, Action::Ignored));
            }
        }

        return success!(Self {
            rules:          rules,
        });
    }

    pub fn find(&self, character_stack: &mut CharacterStack, tokens: &mut Vec<Token>, complete: bool) -> Status<bool> {
        if let Some((matched, action)) = self.rules.check_stack(character_stack) {
            match action {

                Action::Map(operator) => {
                    tokens.push(Token::new(TokenType::Operator(operator), character_stack.final_positions()));
                    return success!(true);
                },

                Action::Invalid => {
                    let error = Error::InvalidToken(identifier!("operator"), string!(String, matched));
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

        return success!(false);
    }
}
