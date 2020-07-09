use internal::*;
use tokenize::Token;

pub struct CommentTokenizer {
    delimiters:     Vec<(VectorString, VectorString)>,
    notes:          Vec<(VectorString, Data)>,
}

impl CommentTokenizer {

    pub fn new(settings: &Data, character_stack: &mut CharacterStack, variant_registry: &mut VariantRegistry) -> Status<Self> {
        variant_registry.has_comments = true;

        ensure!(settings.is_map(), ExpectedFound, expected_list!["map"], settings.clone());
        let mut delimiters = Vec::new();
        let mut notes = Vec::new();

        if let Some(line_comment) = confirm!(settings.index(&keyword!(str, "line_comment"))) {
            let delimiter = unpack_literal!(&line_comment);
            ensure!(!delimiter.is_empty(), EmptyLiteral);
            confirm!(character_stack.register_breaking(delimiter.first().unwrap()));
            confirm!(character_stack.register_signature(delimiter.clone()));
            delimiters.push((delimiter, VectorString::from("\n")));
        }

        if let Some(block_comment) = confirm!(settings.index(&keyword!(str, "block_comment"))) {
            let delimiter_list = unpack_list!(&block_comment);
            ensure!(delimiter_list.len() == 2, InvalidItemCount, integer!(2), integer!(delimiter_list.len() as i64));
            let start_delimiter = unpack_literal!(&delimiter_list[0]);
            let end_delimiter = unpack_literal!(&delimiter_list[1]);
            ensure!(!start_delimiter.is_empty(), EmptyLiteral);
            ensure!(!end_delimiter.is_empty(), EmptyLiteral);
            confirm!(character_stack.register_breaking(start_delimiter.first().unwrap()));
            confirm!(character_stack.register_signature(start_delimiter.clone()));
            push_by_length!(delimiters, start_delimiter, end_delimiter);
        }

        if let Some(notes_lookup) = confirm!(settings.index(&keyword!(str, "note"))) {
            ensure!(notes_lookup.is_map(), ExpectedFound, expected_list!["map"], notes_lookup.clone());
            for (note_keyword, note_type) in confirm!(notes_lookup.pairs()).into_iter() {
                let note_keyword = unpack_literal!(&note_keyword);
                ensure!(!note_keyword.is_empty(), EmptyLiteral);
                ensure!(note_type.is_identifier(), ExpectedFound, expected_list!["identifier"], note_type.clone());
                push_by_length!(notes, note_keyword, note_type);
            }
        }

        return success!(Self {
            delimiters:     delimiters,
            notes:          notes,
        });
    }

    pub fn find(&self, character_stack: &mut CharacterStack, tokens: &mut Vec<Token>, complete: bool, current_pass: &Option<VectorString>, root: &Data, build: &Data, context: &Data) -> Status<bool> {
        for (start_delimiter, end_delimiter) in self.delimiters.iter() {
            if character_stack.check_string(&start_delimiter) {
                let mut comment_string = VectorString::new();

                while !character_stack.check_string(&end_delimiter) {
                    if character_stack.is_empty() {
                        let error = Error::UnterminatedToken(identifier!(str, "comment"));
                        tokens.push(Token::new(TokenType::Invalid(error), character_stack.final_positions()));
                        return success!(true);
                    }
                    comment_string.push(character_stack.pop().unwrap());
                }

                if !complete {
                    for (note_keyword, note_type) in self.notes.iter() { // find multiple flags of the same type in a single comment
                        if let Some(start_position) = comment_string.find(note_keyword) {
                            let offset = start_position + note_keyword.len();
                            let sliced = comment_string.slice_end(offset);
                            let note_message = match sliced.find(&VectorString::from("\n")) { // MAKE THIS WORK FOR BLOCK COMMENTS
                                Some(end_position) => sliced.slice(0, end_position),
                                None => sliced,
                            };

                            let formatter_function_path = path!(vector![keyword!(str, "function"), keyword!(str, "note")]);
                            match confirm!(root.index(&formatter_function_path)) {
                                Some(formatter_function) => { confirm!(function(&formatter_function, vector![note_type.clone(), string!(note_message.clone())], current_pass, root, build, context)); }, // TODO:
                                None => println!("{}: {}", unpack_identifier!(note_type), note_message),
                            }
                        }
                    }
                }

                tokens.push(Token::new(TokenType::Comment(comment_string), character_stack.final_positions()));
                return success!(true);
            }
        }
        return success!(false);
    }
}
