mod character;

use internal::*;

use std::fmt::{ Formatter, Result, Display, Debug };
use std::ops::{ Index, IndexMut };
use std::iter::FromIterator;

pub use self::character::Character;

#[derive(Clone, PartialEq, Eq)]
pub struct VectorString {
    data:       Vector<Character>,
}

#[allow(dead_code)]
impl VectorString {

    pub fn new() -> Self {
        Self {
            data:       Vector::new(),
        }
    }

    pub fn from(source: &str) -> Self {
        Self {
            data:       source.chars().map(|character| Character::from_char(character)).collect(),
        }
    }

    pub fn from_data(data: Vector<Character>) -> Self {
        Self {
            data:       data,
        }
    }

    fn check_from(&self, index: usize, sample: &VectorString) -> bool {
        for (offset, character) in sample.chars().enumerate() {
            if self.data.len() - index < sample.len() {
                return false;
            }

            if self.data[index + offset] != *character {
                return false;
            }
        }
        return true;
    }

    pub fn serialize(&self) -> String {
        return self.data.iter().map(|character| character.serialize()).collect();
    }

    pub fn printable(&self) -> String {
        return self.data.iter().map(|character| character.as_char()).collect();
    }

    pub fn push(&mut self, character: Character) {
        self.data.push(character);
    }

    pub fn pop(&mut self) -> Option<Character> {
        return self.data.pop();
    }

    pub fn push_str(&mut self, source: &VectorString) {
        for character in source.chars() {
            self.data.push(*character);
        }
    }

    pub fn insert(&mut self, index: usize, character: Character) {
        self.data.insert(index, character);
    }

    pub fn insert_str(&mut self, index: usize, source: &VectorString) {
        for character in source.reverse_chars() {
            self.data.insert(index, *character);
        }
    }

    pub fn len(&self) -> usize {
        return self.data.len();
    }

    pub fn chars(&self) -> VectorIterator<Character> {
        return self.data.iter();
    }

    pub fn reverse_chars(&self) -> ReverseVectorIterator<Character> {
        return self.data.reverse_iter();
    }

    pub fn contains(&self, sample: &VectorString) -> bool {
        for start in 0..self.data.len() {
            if self.check_from(start, sample) {
                return true;
            }
        }
        return false;
    }

    pub fn find(&self, sample: &VectorString) -> Option<usize> {
        for start in 0..self.data.len() {
            if self.check_from(start, sample) {
                return Some(start);
            }
        }
        return None;
    }

    pub fn is_empty(&self) -> bool {
        return self.data.is_empty();
    }

    pub fn split(&self, sample: &VectorString, void: bool) -> Vec<Self> {
        let mut pieces = Vec::new();
        let mut buffer = VectorString::new();
        let mut start = 0;

        while start < self.data.len() {
            if self.check_from(start, sample) {
                if !void || !buffer.is_empty() {
                    pieces.push(buffer.clone());
                    buffer.clear();
                }
                start += sample.len();
            } else {
                buffer.push(self.data[start]);
                start += 1;
            }
        }

        if !void || !buffer.is_empty() {
            pieces.push(buffer);
        }

        return pieces;
    }

    pub fn slice(&self, start: usize, end: usize) -> Self {
        Self {
            data:       self.data.slice(start, end),
        }
    }

    pub fn slice_end(&self, start: usize) -> Self {
        Self {
            data:       self.data.slice_end(start),
        }
    }

    pub fn first(&self) -> Option<Character> {
        return self.chars().next().cloned();
    }

    pub fn uppercase(&self) -> Self {
        return self.data.iter().map(|character| character.uppercase()).collect();
    }

    pub fn lowercase(&self) -> Self {
        return self.data.iter().map(|character| character.lowercase()).collect();
    }

    pub fn is_uppercase(&self) -> bool {
        return self.data.iter().find(|character| !character.is_uppercase()).is_none();
    }

    pub fn is_lowercase(&self) -> bool {
        return self.data.iter().find(|character| !character.is_lowercase()).is_none();
    }

    pub fn remove(&mut self, index: usize) -> Character {
        return self.data.remove(index);
    }

    pub fn replace(&self, from: &VectorString, to: &VectorString) -> Self {
        let mut string = VectorString::new();
        let mut start = 0;

        while start < self.data.len() {
            if self.check_from(start, from) {
                string.push_str(to);
                start += from.len();
            } else {
                string.push(self.data[start]);
                start += 1;
            }
        }

        return string;
    }

    pub fn position(&self, sample: &VectorString) -> Vec<usize> {
        let mut positions = Vec::new();
        for start in 0..self.data.len() {
            if self.check_from(start, sample) {
                positions.push(start);
            }
        }
        return positions;
    }

    pub fn flip(&self) -> Self {
        return VectorString::from_data(self.data.flip());
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}

impl FromIterator<Character> for VectorString {

    fn from_iter<I: IntoIterator<Item = Character>>(iterator: I) -> VectorString {
        let mut string = VectorString::new();
        for character in iterator {
            string.push(character);
        }
        return string;
    }
}

impl FromIterator<VectorString> for VectorString {

    fn from_iter<I: IntoIterator<Item = VectorString>>(iterator: I) -> VectorString {
        let mut string = VectorString::new();
        for source in iterator {
            string.push_str(&source);
        }
        return string;
    }
}

impl Index<usize> for VectorString {
    type Output = Character;

    fn index(&self, index: usize) -> &Character {
        return self.data.index(index);
    }
}

impl IndexMut<usize> for VectorString {

    fn index_mut(&mut self, index: usize) -> &mut Character {
        return self.data.index_mut(index);
    }
}

impl Debug for VectorString {

    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        return write!(f, "\"{}\"", self.serialize());
    }
}

impl Display for VectorString {

    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        return write!(f, "{}", self.printable());
    }
}

impl Compare for VectorString {

    fn compare(&self, other: &Self) -> Relation {
        let mut index = 0;
        loop {
            if self.len() <= index {
                match other.len() <= index {
                    true => return Relation::Equal,
                    false => return Relation::Smaller,
                }
            }

            if other.len() <= index {
                return Relation::Bigger;
            }

            if self[index] == other[index] {
                index += 1;
                continue;
            }

            return self[index].compare(&other[index]);
        }
    }
}
