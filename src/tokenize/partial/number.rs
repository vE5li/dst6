use internal::*;
use debug::*;

use tokenize::Token;

fn to_length(option: &Option<VectorString>) -> usize {
    match option {
        Some(some) => some.len(),
        None => 0,
    }
}

struct Format {
    pub variants:       Vec<(Option<VectorString>, VectorString)>,
    suffixes:           Vec<VectorString>,
    digits:             Vec<Character>,
}

impl Format {

    pub fn new() -> Self {
        Self {
            variants:       Vec::new(),
            digits:         Vec::new(),
            suffixes:       Vec::new(),
        }
    }

    pub fn validate(&self) -> Status<()> {
        'validate: for suffix in &self.suffixes {
            for character in suffix.chars() {
                if !self.digits.contains(&character) {
                    continue 'validate;
                }
            }
            return error!(InvalidSuffix, string!(String, suffix.clone()));
        }
        return success!(());
    }

    pub fn add(&mut self, suffix: Option<VectorString>, number_system: VectorString, number_systems: &Map<VectorString, Vec<Character>>) -> Status<()> {
        if let Some(suffix) = &suffix {
            self.suffixes.push(suffix.clone());
        } else {
            let number_system = number_systems.get(&number_system).unwrap();
            for digit in number_system.iter() {
                if !self.digits.contains(digit) {
                    self.digits.push(*digit);
                }
            }
        }

        match self.variants.iter().position(|(variant_suffix, _)| to_length(variant_suffix) <= to_length(&suffix)) {
            Some(position) => self.variants.insert(position, (suffix, number_system)),
            None => self.variants.push((suffix, number_system)),
        }

        return success!(());
    }
}

pub struct NumberTokenizer {
    number_systems:     Map<VectorString, Vec<Character>>,
    formats:            Vec<(Option<VectorString>, Format)>,
    float_delimiter:    Option<VectorString>,
    negative:           Option<VectorString>,
}

impl NumberTokenizer {

    pub fn new(settings: &Data, character_stack: &mut CharacterStack, variant_registry: &mut VariantRegistry) -> Status<Self> {
        ensure!(settings.is_map(), ExpectedFound, expected_list!["map"], settings.clone());
        variant_registry.has_integers = true;

        let mut number_systems = Map::new();
        let mut float_delimiter = None;
        let mut negative = None;
        let mut formats = Vec::new();
        let mut all_digits = Vec::new();

        for (name, digits) in confirm!(index_field!(settings, "system").pairs()).into_iter() {
            let name = unpack_identifier!(&name);
            let mut collected_digits = Vec::new();

            for character in unpack_list!(&digits).into_iter() {
                let character = unpack_character!(&character);
                confirm!(character_stack.register_non_breaking(character));
                collected_digits.push(character);
                if !all_digits.contains(&character) {
                    all_digits.push(character);
                }
            }

            ensure!(collected_digits.len() >= 2, Message, string!("number system needs at least two digits"));
            number_systems.insert(name, collected_digits);
        }

        if let Some(format_lookup) = confirm!(settings.index(&keyword!("format"))) {
            ensure!(format_lookup.is_map(), ExpectedFound, expected_list!["map"], format_lookup);

            for (prefix, group) in confirm!(format_lookup.pairs()).into_iter() {
                ensure!(group.is_map(), ExpectedFound, expected_list!["map"], group);

                let prefix = match prefix.is_keyword() {

                    true => {
                        match unpack_keyword!(&prefix).printable().as_str() {
                            "none" => None,
                            _invalid => return error!(InvalidPrefix, prefix),
                        }
                    },

                    false => {
                        let prefix = unpack_literal!(&prefix);
                        ensure!(!prefix.is_empty(), EmptyLiteral);
                        let first = prefix[0];

                        if !all_digits.contains(&first) {
                            confirm!(character_stack.register_breaking(first));
                            confirm!(character_stack.register_signature(prefix.clone()));
                        } else {
                            confirm!(character_stack.register_pure(&prefix));
                        }
                        Some(prefix)
                    },
                };

                let mut format = Format::new();

                for (suffix, system) in confirm!(group.pairs()).into_iter() {
                    let system_name = unpack_identifier!(&system);

                    if number_systems.get(&system_name).is_none() {
                        return error!(InvalidNumberSystem, system);
                    }

                    match suffix.is_keyword() {

                        true => {
                            match unpack_keyword!(&suffix).printable().as_str() {
                                "none" => confirm!(format.add(None, system_name, &number_systems)),
                                _invalid => return error!(InvalidSuffix, suffix),
                            }
                        },

                        false => {
                            let suffix = unpack_literal!(&suffix);
                            ensure!(!suffix.is_empty(), EmptyLiteral);
                            for character in suffix.chars() {
                                confirm!(character_stack.register_non_breaking(*character));
                            }
                            confirm!(format.add(Some(suffix), system_name, &number_systems))
                        },
                    }
                }

                confirm!(format.validate());

                match formats.iter().position(|(format_prefix, _)| to_length(format_prefix) <= to_length(&prefix)) {
                    Some(position) => formats.insert(position, (prefix, format)),
                    None => formats.push((prefix, format)),
                }
            }
        }

        if let Some((_, format)) = formats.iter().find(|(prefix, _)| prefix.is_none()) {
            if let Some((_suffix, system)) = format.variants.iter().find(|(suffix, _)| suffix.is_none()) {
                for character in number_systems.get(system).unwrap().iter() {
                    confirm!(character_stack.register_signature(character.to_string()));
                }
            }
        }

        if let Some(delimiter) = confirm!(settings.index(&keyword!("float"))) {
            let delimiter = unpack_literal!(&delimiter);
            ensure!(!delimiter.is_empty(), EmptyLiteral);
            confirm!(character_stack.register_breaking(delimiter[0]));
            float_delimiter = Some(delimiter);
            variant_registry.has_floats = true;
        }

        if let Some(literal) = confirm!(settings.index(&keyword!("negative"))) {
            let literal = unpack_literal!(&literal);
            ensure!(!literal.is_empty(), EmptyLiteral);
            confirm!(character_stack.register_breaking(literal[0]));
            confirm!(character_stack.register_signature(literal.clone()));
            negative = Some(literal);
            variant_registry.has_negatives = true;
        }

        return success!(Self {
            number_systems:     number_systems,
            formats:            formats,
            float_delimiter:    float_delimiter,
            negative:           negative,
        });
    }

    fn parse_number(&self, source: &VectorString, float_source: Option<&VectorString>, number_system: &VectorString, negative: bool, positions: &Vec<Position>) -> Option<Token> {
        let number_system = self.number_systems.get(number_system).unwrap();
        let base = number_system.len();
        let mut value = 0;

        'parse: for (index, character) in source.reverse_chars().enumerate() {
            for (digit_index, digit) in number_system.iter().enumerate() {
                if *character == *digit {
                    value += digit_index * base.pow(index as u32);
                    continue 'parse;
                }
            }
            return None;
        }

        if let Some(float_source) = float_source {
            let mut float_value = 0;

            'parse_float: for (index, character) in float_source.reverse_chars().enumerate() {
                for (digit_index, digit) in number_system.iter().enumerate() {
                    if *character == *digit {
                        float_value += digit_index * base.pow(index as u32);
                        continue 'parse_float;
                    }
                }
                return None;
            }

            let temp = float_value as f64 / base.pow(float_source.len() as u32) as f64;
            match negative {
                true => return Some(Token::new(TokenType::Float(-(value as f64 + temp)), positions.clone())), // dirty please fix
                false => return Some(Token::new(TokenType::Float(value as f64 + temp), positions.clone())), // dirty please fix
            }
        }

        match negative {
            true => return Some(Token::new(TokenType::Integer(-(value as i64)), positions.clone())), // dirty please fix
            false => return Some(Token::new(TokenType::Integer(value as i64), positions.clone())), // dirty please fix
        }
    }

    fn try_parse(&self, source: &VectorString, float_source: Option<&VectorString>, format: &Format, negative: bool, positions: &Vec<Position>) -> Option<Token> {
        for (suffix, number_system) in format.variants.iter() {
            if let Some(suffix) = suffix {
                if let Some(float_source) = float_source {
                    if suffix.len() < float_source.len() {
                        let start = float_source.len() - suffix.len();
                        let sliced = float_source.slice_end(start);
                        if sliced == *suffix {
                            if let Some(token) = self.parse_number(&source, Some(&sliced), number_system, negative, positions) {
                                return Some(token);
                            }
                        }
                    }
                } else {
                    if suffix.len() < source.len() {
                        let start = source.len() - suffix.len();
                        let sliced = source.slice_end(start);
                        if sliced == *suffix {
                            if let Some(token) = self.parse_number(&sliced, None, number_system, negative, positions) {
                                return Some(token);
                            }
                        }
                    }
                }
            } else {
                if let Some(float_source) = float_source {
                    if let Some(token) = self.parse_number(&source, Some(&float_source), number_system, negative, positions) {
                        return Some(token);
                    }
                } else {
                    if let Some(token) = self.parse_number(&source, None, number_system, negative, positions) {
                        return Some(token);
                    }
                }
            }
        }

        return None;
    }

    pub fn find(&self, character_stack: &mut CharacterStack, tokens: &mut Vec<Token>, _error: &mut Option<Error>) -> Status<bool> {
        for (prefix, format) in self.formats.iter() {
            match prefix {

                Some(prefix) => {
                    if character_stack.check_string(&prefix) {
                        let negative = match &self.negative {
                            Some(negative) => character_stack.check_string(negative),
                            None => false,
                        };
                        let source = confirm!(character_stack.till_breaking());

                        if let Some(float_delimiter) = &self.float_delimiter {
                            character_stack.save();
                            if character_stack.check_string(float_delimiter) {
                                if let Status::Success(float_source) = character_stack.till_breaking() {
                                    if let Some(token) = self.try_parse(&source, Some(&float_source), format, negative, &character_stack.final_positions()) {
                                        character_stack.drop();
                                        tokens.push(token);
                                        return success!(true);
                                    }
                                }
                            }
                            character_stack.restore();
                        }

                        if let Some(token) = self.try_parse(&source, None, format, negative, &character_stack.final_positions()) {
                            tokens.push(token);
                            return success!(true);
                        }

                        let error = Error::ExpectedImmediate;
                        tokens.push(Token::new(TokenType::Invalid(error), character_stack.final_positions()));
                        character_stack.drop();
                        return success!(true);
                    }
                },

                None => {
                    character_stack.save();

                    let negative = match &self.negative {
                        Some(negative) => character_stack.check_string(negative),
                        None => false,
                    };

                    let source = match character_stack.till_breaking() {
                        Status::Success(source) => source,
                        Status::Error(_) => {
                            character_stack.restore();
                            return success!(false)
                        },
                    };

                    if let Some(float_delimiter) = &self.float_delimiter {
                        character_stack.save();
                        if character_stack.check_string(float_delimiter) {
                            if let Status::Success(float_source) = character_stack.till_breaking() {
                                if let Some(token) = self.try_parse(&source, Some(&float_source), format, negative, &character_stack.final_positions()) {
                                    character_stack.drop();
                                    character_stack.drop();
                                    tokens.push(token);
                                    return success!(true);
                                }
                            }
                        }
                        character_stack.restore();
                    }

                    if let Some(token) = self.try_parse(&source, None, format, negative, &character_stack.final_positions()) {
                        character_stack.drop();
                        tokens.push(token);
                        return success!(true);
                    }

                    if negative {
                        let error = Error::ExpectedImmediate;
                        tokens.push(Token::new(TokenType::Invalid(error), character_stack.final_positions()));
                        character_stack.drop();
                        return success!(true);
                    } else {
                        character_stack.restore();
                    }
                }
            }
        }

        return success!(false);
    }
}
