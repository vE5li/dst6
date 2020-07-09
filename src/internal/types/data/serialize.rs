use internal::*;

pub fn serialize_float(value: f64) -> VectorString {
    let mut string = value.to_string();
    if !string.contains(".") {
        string.push_str(".0");
    }
    return VectorString::from(&string);
}

pub fn serialize_literal(source: &VectorString, delimiter: char) -> VectorString {
    return format_vector!("{}{}{}", delimiter, source.serialize(), delimiter);
}

pub fn serialize_map(source: &DataMap) -> VectorString {
    let mut string = VectorString::from("{");

    for (key, value) in source.iter() {
        string.push(Character::from_char(' '));
        string.push_str(&key.serialize());
        string.push(Character::from_char(' '));
        string.push_str(&value.serialize());
    }
    string.push(Character::from_char(' '));
    string.push(Character::from_char('}'));

    return string;
}

pub fn serialize_list(source: &Vector<Data>) -> VectorString {
    let mut string = VectorString::from("[");

    for item in source.iter() {
        string.push(Character::from_char(' '));
        string.push_str(&item.serialize());
    }
    string.push(Character::from_char(' '));
    string.push(Character::from_char(']'));

    return string;
}

pub fn serialize_path(source: &Vector<Data>) -> VectorString {
    let mut string = VectorString::new();

    for (index, step) in source.iter().enumerate() {
        string.push_str(&step.serialize());
        if index != source.len() - 1 {
            string.push(Character::from_char(':'));
        }
    }

    return string;
}
