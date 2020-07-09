use internal::*;
use std::fmt::{ Formatter, Result, Display, Debug };

static SERIALIZE: [&'static str; 95] = [ " ", "!", "\\\"", "#", "$", "%", "&", "\\\'", "(", ")", "*", "+", ",", "-", ".", "/", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", ":", ";", "<", "=", ">", "?", "@", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "[", "\\\\", "]", "^", "_", "`", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "{", "|", "}", "~" ];

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Character {
    data:       u32,
}

impl Character {

    pub fn from_code(code: u32) -> Self {
        Self {
            data:       code,
        }
    }

    pub fn from_char(character: char) -> Self {
        Self {
            data:       character as u32,
        }
    }

    pub fn code(&self) -> u32 {
        return self.data;
    }

    pub fn serialize(&self) -> String {
        match self.data {
            8 => String::from("\\b"),
            9 => String::from("\\t"),
            10 => String::from("\\n"),
            13 => String::from("\\r"),
            27 => String::from("\\e"),
            32..=126 => String::from(SERIALIZE[(self.data - 32) as usize]),
            0..=31 => format!("\\[{}]", self.data),
            other => format!("{}", other),
        }
    }

    pub fn to_string(&self) -> VectorString {
        let mut string = VectorString::new();
        string.push(*self);
        return string;
    }

    pub fn as_char(&self) -> char {
        return self.data as u8 as char; // TODO
    }

    pub fn uppercase(&self) -> Self {
        return Character::from_char(self.as_char().to_ascii_uppercase());
    }

    pub fn lowercase(&self) -> Self {
        return Character::from_char(self.as_char().to_ascii_lowercase());
    }

    pub fn is_uppercase(&self) -> bool {
        return self.as_char().is_uppercase();
    }

    pub fn is_lowercase(&self) -> bool {
        return self.as_char().is_lowercase();
    }

    pub fn is_digit(&self) -> bool {
        match self.data {
            48..=57 => return true,
            _other => return false,
        }
    }

    pub fn is_newline(&self) -> bool {
        return self.as_char() == '\n';
    }
}

impl Debug for Character {

    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        return write!(f, "\'{}\'", self.serialize());
    }
}

impl Display for Character {

    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        return write!(f, "{}", self.as_char());
    }
}

impl Compare for Character {

    fn compare(&self, other: &Self) -> Relation {
        if self.code() == other.code() {
            return Relation::Equal;
        }

        match self.code() < other.code() {
            true => return Relation::Smaller,
            false => return Relation::Bigger,
        }
    }
}
