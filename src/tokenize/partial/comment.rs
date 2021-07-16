use internal::*;
use debug::*;

use tokenize::Token;

pub struct CommentTokenizer {
    delimiters:     Vec<(SharedString, SharedString)>,
    notes:          Vec<(SharedString, Data)>,
}

impl CommentTokenizer {

    pub fn new(settings: &Data, character_stack: &mut CharacterStack, variant_registry: &mut VariantRegistry) -> Status<Self> {
        ensure!(settings.is_map(), ExpectedFound, expected_list!["map"], settings.clone());
        variant_registry.has_comments = true;
        let mut delimiters = Vec::new();
        let mut notes = Vec::new();

        if let Some(line_comment) = confirm!(settings.index(&keyword!("line_comments"))) {
            for delimiter in unpack_list!(&line_comment).iter() {
                let delimiter = unpack_literal!(delimiter);
                ensure!(!delimiter.is_empty(), EmptyLiteral);
                confirm!(character_stack.register_breaking(delimiter[0]));
                confirm!(character_stack.register_signature(delimiter.clone()));
                delimiters.push((delimiter, SharedString::from("\n")));
            }
        }

        if let Some(block_comment) = confirm!(settings.index(&keyword!("block_comments"))) {
            for delimiter_list in unpack_list!(&block_comment).iter() {
                let delimiter_list = unpack_list!(delimiter_list);
                ensure!(delimiter_list.len() == 2, InvalidItemCount, integer!(2), integer!(delimiter_list.len() as i64));
                let start_delimiter = unpack_literal!(&delimiter_list[0]);
                let end_delimiter = unpack_literal!(&delimiter_list[1]);
                ensure!(!start_delimiter.is_empty(), EmptyLiteral);
                ensure!(!end_delimiter.is_empty(), EmptyLiteral);
                confirm!(character_stack.register_breaking(start_delimiter[0]));
                confirm!(character_stack.register_signature(start_delimiter.clone()));
                push_by_length!(delimiters, start_delimiter, end_delimiter);
            }
        }

        if let Some(notes_lookup) = confirm!(settings.index(&keyword!("notes"))) {
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

    pub fn find(&self, character_stack: &mut CharacterStack, tokens: &mut Vec<Token>, notes: &mut Vec<Note>) -> Status<bool> {
        for (start_delimiter, end_delimiter) in self.delimiters.iter() {
            if character_stack.check_string(&start_delimiter) {
                let mut comment_string = SharedString::new();
                let mut note = None;

                'find: while !character_stack.check_string(&end_delimiter) {
                    if character_stack.is_empty() {
                        let error = Error::UnterminatedToken(identifier!("comment"));
                        tokens.push(Token::new(TokenType::Invalid(error), character_stack.final_positions()));
                        return success!(true);
                    }

                    if note.is_none() {
                        for (note_keyword, note_type) in self.notes.iter() {
                            if character_stack.check_string(note_keyword) {
                                let position = character_stack.current_position();
                                note = Some(Note::new(note_type.clone(), SharedString::new(), position));
                                comment_string.push_str(note_keyword);
                                continue 'find;
                            }
                        }
                    }

                    let character = character_stack.pop().unwrap();
                    if character.is_newline() {
                        if let Some(note) = note.take() {
                            notes.push(note);
                        }
                    } else {
                        if let Some(note) = &mut note {
                            note.message.push(character);
                            note.position.length += 1;
                        }
                    }
                    comment_string.push(character);
                }

                if let Some(note) = note {
                    notes.push(note);
                }

                tokens.push(Token::new(TokenType::Comment(comment_string), character_stack.final_positions()));
                return success!(true);
            }
        }
        return success!(false);
    }
}
