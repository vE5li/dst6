use internal::*;
use debug::*;

#[derive(Debug)]
pub struct CharacterStack {
    source:         SharedString,
    save_states:    Vec<(usize, Vec<Position>)>,
    index:          usize,
    file:           Option<SharedString>,
    breaking:       Vec<Character>,
    non_breaking:   Vec<Character>,
    signature:      Vec<SharedString>,
    positions:      Vec<Position>,
}

impl CharacterStack {

    pub fn new(source: SharedString, file_path: Option<SharedString>) -> Self {
        let non_breaking = vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

        let mut breaking = Vec::new();
        for character in 0..33 {
            breaking.push(character as u8 as char);
        }
        breaking.append(&mut vec!['.', '-', ':', '!', '#', '[', ']', '{', '}', '\'', '\"', 127 as char]);

        Self {
            positions:      vec![Position::new(file_path.clone(), source.clone(), 0, 1, 1, 0)],
            source:         source,
            save_states:    Vec::new(),
            index:          0,
            file:           file_path,
            breaking:       breaking.into_iter().map(|character| Character::from_char(character)).collect(), // unly, please fix
            non_breaking:   non_breaking.into_iter().map(|character| Character::from_char(character)).collect(), // unly, please fix
            signature:      Vec::new(),
        }
    }

    pub fn derive(&self, source: SharedString, file_path: Option<SharedString>) -> Self {
        Self {
            positions:      vec![Position::new(file_path.clone(), source.clone(), 0, 1, 1, 0)],
            source:         source,
            save_states:    Vec::new(),
            index:          0,
            file:           file_path,
            breaking:       self.breaking.clone(),
            non_breaking:   self.non_breaking.clone(),
            signature:      self.signature.clone(),
        }
    }

    pub fn current_position(&self) -> Position {
        let position = self.positions.last().unwrap();
        return Position::new(self.file.clone(), self.source.clone(), position.index, position.line, position.character + position.length, 0);
    }

    pub fn start_positions(&mut self) {
        let position = self.positions.last().unwrap();
        self.positions = vec![Position::new(self.file.clone(), self.source.clone(), self.index, position.line, position.character + position.length, 0)];
    }

    pub fn final_positions(&self) -> Vec<Position> {
        let mut positions = self.positions.clone();
        positions.retain(|position| position.length != 0);
        return positions;
    }

    pub fn save(&mut self) {
        self.save_states.push((self.index, self.positions.clone()));
    }

    pub fn restore(&mut self) {
        let (index, positions) = self.save_states.pop().unwrap();
        self.index = index;
        self.positions = positions;
    }

    pub fn drop(&mut self) {
        self.save_states.pop().unwrap();
    }

    pub fn advance(&mut self, offset: usize) {
        for _offset in 0..offset {
            if self.source.len() > self.index {
                self.positions.last_mut().unwrap().length += 1;
                if self.source[self.index].as_char() == '\n' {
                    let line = self.positions.last().unwrap().line;
                    self.positions.push(Position::new(self.file.clone(), self.source.clone(), self.index + 1, line + 1, 1, 0));
                }
            }
            self.index += 1;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.source.len() <= self.index
    }

    pub fn peek(&self, offset: usize) -> Option<Character> {
        match self.source.len() > self.index + offset {
            true => return Some(self.source[self.index + offset]),
            false => return None,
        }
    }

    pub fn pop(&mut self) -> Option<Character> {
        if self.is_empty() {
            return None;
        }
        let character = self.source[self.index];
        self.advance(1);
        return Some(character);
    }

    pub fn till_breaking(&mut self) -> Status<SharedString> {
        let first_character = expect!(self.pop(), ExpectedWord); // TODO better error
        let mut word = first_character.to_string();

        if self.is_breaking(first_character) {
            return error!(ExpectedWordFound, Data::String(word));
        }

        while let Some(character) = self.peek(0) {
            match self.is_breaking(character) {
                true => break,
                false => {
                    word.push(character);
                    self.advance(1);
                },
            }
        }

        return success!(word);
    }

    pub fn check(&mut self, compare: char) -> bool {
        if let Some(character) = self.peek(0) {
            if character == Character::from_char(compare) {
                self.advance(1);
                return true
            }
        }
        false
    }

    pub fn check_string(&mut self, compare: &SharedString) -> bool {
        if self.index + compare.len() > self.source.len() {
            return false;
        }

        for (index, character) in compare.chars().enumerate() {
            if self.source[self.index + index] != *character {
                return false;
            }
        }

        self.advance(compare.len());
        return true;
    }

    pub fn register_non_breaking(&mut self, character: Character) -> Status<()> {
        if self.breaking.contains(&character) {
            return error!(DuplicateNonBreaking, character!(character));
        }
        if !self.non_breaking.contains(&character) {
            self.non_breaking.push(character);
        }
        return success!(());
    }

    pub fn register_breaking(&mut self, character: Character) -> Status<()> {
        if self.non_breaking.contains(&character) {
            return error!(DuplicateBreaking, character!(character));
        }
        if !self.breaking.contains(&character) {
            self.breaking.push(character);
        }
        return success!(());
    }

    pub fn register_signature(&mut self, signature: SharedString) -> Status<()> {
        if !self.signature.contains(&signature) {
            self.signature.push(signature);
        } else {
            return error!(DuplicateSignature, Data::String(signature));
        }
        return success!(());
    }

    pub fn register_pure(&mut self, literal: &SharedString) -> Status<()> {
        for character in literal.chars() {
            confirm!(self.register_non_breaking(*character));
        }
        return success!(());
    }

    pub fn is_breaking(&self, compare: Character) -> bool {
        return self.breaking.contains(&compare);
    }

    pub fn is_pure(&self, compare: &SharedString) -> bool {
        return compare.chars().find(|character| self.is_breaking(**character)).is_none();
    }
}
