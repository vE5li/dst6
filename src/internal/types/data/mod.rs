mod index;
mod serialize;

use internal::*;
use debug::*;

use self::serialize::*;
use self::index::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Data {
    Map(DataMap),
    Path(SharedVector<Data>),
    List(SharedVector<Data>),
    Identifier(SharedString),
    Keyword(SharedString),
    String(SharedString),
    Character(Character),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

impl Data {

    fn wrapped_index(selector: &Data, biggest: usize) -> Status<Option<usize>> {
        let integer = unpack_integer!(selector);
        ensure!(integer != 0, IndexOutOfBounds, selector.clone(), integer!(biggest as i64));

        if integer > 0 {
            let index = integer as usize;
            if index <= biggest {
                return success!(Some(index - 1));
            }
        } else {
            let index = 1 + (biggest as i64) + integer;
            if index > 0 {
                return success!(Some(index as usize - 1));
            }
        }

        return success!(None);
    }

    pub fn round(&self) -> Status<Data> {
        match self {
            Data::Integer(integer) => return success!(integer!(*integer)),
            Data::Float(float) => return success!(integer!(float.round() as i64)),
            Data::Character(character) => return success!(integer!(character.code() as i64)),
            _invalid => return error!(Message, string!("only numbers may be converted to an integer")),
        }
    }

    pub fn integer(&self) -> Status<Data> {
        match self {
            Data::Integer(integer) => return success!(integer!(*integer)),
            Data::Float(float) => return success!(integer!(*float as i64)),
            Data::Character(character) => return success!(integer!(character.code() as i64)),
            _invalid => return error!(Message, string!("only numbers may be converted to an integer")),
        }
    }

    pub fn float(&self) -> Status<Data> {
        match self {
            Data::Integer(integer) => return success!(float!(*integer as f64)),
            Data::Float(float) => return success!(float!(*float)),
            Data::Character(character) => return success!(float!(character.code() as i64 as f64)),
            _invalid => return error!(Message, string!("only numbers may be converted to an integer")),
        }
    }

    pub fn character(&self) -> Status<Data> {
        match self {
            Data::Integer(integer) => return success!(character!(code, *integer as u32)), // USE try_into AND PANIC ON OVERFLOW
            Data::Float(float) => return success!(character!(code, *float as u32)),
            Data::Character(character) => return success!(character!(character.clone())),
            _invalid => return error!(Message, string!("only numbers may be converted to an integer")),
        }
    }

    pub fn to_string(&self) -> SharedString {
        match self {
            Data::String(ref string) => return string.clone(),
            Data::Character(ref character) => return character.to_string(),
            Data::Keyword(ref keyword) => return keyword.clone(),
            Data::Boolean(ref boolean) => return boolean_to_string!(boolean),
            _other => return self.serialize(),
        }
    }

    pub fn length(&self) -> Status<usize> {
        match self {
            Data::Map(ref map) => return success!(map.len()),
            Data::List(ref items) => return success!(items.len()),
            Data::Path(ref steps) => return success!(steps.len()),
            Data::String(ref string) => return success!(string.len()),
            Data::Identifier(ref identifier) => return success!(identifier.len()),
            Data::Keyword(ref keyword) => return success!(keyword.len()),
            _other => return error!(ExpectedFound, expected_list!["container"], self.clone()),
        }
    }

    pub fn data_type(&self) -> SharedString {
        match self {
            Data::Map(..) => return SharedString::from("map"),
            Data::List(..) => return SharedString::from("list"),
            Data::Path(..) => return SharedString::from("path"),
            Data::Keyword(..) => return SharedString::from("keyword"),
            Data::Identifier(..) => return SharedString::from("identifier"),
            Data::String(..) => return SharedString::from("string"),
            Data::Character(..) => return SharedString::from("character"),
            Data::Integer(..) => return SharedString::from("integer"),
            Data::Float(..) => return SharedString::from("float"),
            Data::Boolean(..) => return SharedString::from("boolean"),
        }
    }

    pub fn empty(&self) -> Status<Data> {
        match self {
            Data::Map(..) => return success!(map!()),
            Data::List(..) => return success!(list!()),
            Data::String(..) => return success!(string!()),
            invalid => return error!(ExpectedFound, expected_list!["map", "list", "string"], invalid.clone()),
        }
    }

    pub fn merge(&self, source: &Data) -> Status<Data> {
        match self {
            Data::Map(data_map) => {
                match source {
                    Data::Map(source_data_map) => {
                        let mut data_map = data_map.clone();
                        for (source_key, source_value) in source_data_map.iter() {
                            if let Some(value) = data_map.get(source_key) {
                                let new_value = match value == source_value {
                                    true => value.clone(),
                                    false => confirm!(value.merge(&source_value)),
                                };
                                data_map.insert(source_key.clone(), new_value);
                                continue;
                            }
                            data_map.insert(source_key.clone(), source_value.clone());
                        }
                        return success!(map!(data_map));
                    },
                    _other => return error!(ExpectedFound, expected_list!["map"], source.clone()),
                }
            },

            Data::List(items) => {
                match source {
                    Data::List(source_items) => return success!(list!(items.iter().cloned().chain(source_items.iter().cloned()).collect())),
                    _other => return error!(ExpectedFound, expected_list!["list"], source.clone()),
                }
            },

            other => {
                ensure!(other == source, Message, string!("conflictig values for merge"));
                return success!(self.clone());
            },
        }
    }

    pub fn absolute(&self) -> Status<Data> {
        match self {
            Data::Integer(self_value) => return success!(integer!(self_value.abs())),
            Data::Character(self_value) => return success!(integer!((self_value.code() as i64).abs())),
            Data::Float(self_value) => return success!(float!(self_value.abs())),
            _other => error!(Message, string!("only numbers may be absoluted")), // find a better word lol
        }
    }

    pub fn negate(&self) -> Status<Data> {
        match self {
            Data::Integer(self_value) => return success!(integer!(-*self_value)),
            Data::Character(self_value) => return success!(integer!(-(self_value.code() as i64))),
            Data::Float(self_value) => return success!(float!(-*self_value)),
            _other => error!(Message, string!("only numbers may be negated")),
        }
    }

    pub fn square_root(&self) -> Status<Data> {
        match self {
            Data::Integer(value) => return success!(float!((*value as f64).sqrt())),
            Data::Float(value) => return success!(float!(value.sqrt())),
            Data::Character(value) => return success!(float!((value.code() as i64 as f64).sqrt())),
            _invalid => return error!(Message, string!("only numbers may be converted to an integer")),
        }
    }

    pub fn power(&self, source: &Data) -> Status<Data> {
        match self {

            Data::Integer(self_value) => {
                match source {
                    Data::Integer(source_value) => return success!(integer!(self_value.pow(*source_value as u32))),
                    Data::Character(source_value) => return success!(integer!(self_value.pow(source_value.code() as u32))),
                    _other => error!(Message, string!("integers may only be raised to the power of other integers or characters")),
                }
            }

            Data::Character(self_value) => {
                match source {
                    Data::Integer(source_value) => return success!(integer!((self_value.code() as i64).pow(*source_value as u32))),
                    Data::Character(source_value) => return success!(integer!((self_value.code() as i64).pow(source_value.code() as u32))),
                    _other => error!(Message, string!("characters may only be raised to the power of other characters or integers")),
                }
            }

            Data::Float(self_value) => {
                match source {
                    Data::Integer(source_value) => return success!(float!(self_value.powf(*source_value as f64))),
                    Data::Character(source_value) => return success!(float!(self_value.powf(source_value.code() as i64 as f64))),
                    Data::Float(source_value) => return success!(float!(self_value.powf(*source_value))),
                    _other => error!(Message, string!("floats may only be raised to the power of other numbers")),
                }
            }

            _other => error!(Message, string!("only integers and characters may be raised to the power")),
        }
    }

    pub fn logarithm(&self, source: &Data) -> Status<Data> {
        match self {

            Data::Integer(self_value) => {
                match source {
                    Data::Integer(source_value) => return success!(float!((*self_value as f64).log(*source_value as f64))),
                    Data::Character(source_value) => return success!(float!((*self_value as f64).log(source_value.code() as i64 as f64))),
                    Data::Float(source_value) => return success!(float!((*self_value as f64).log(*source_value))),
                    _other => error!(Message, string!("integers may only be raised to the power of other integers or characters")),
                }
            }

            Data::Character(self_value) => {
                match source {
                    Data::Integer(source_value) => return success!(float!((self_value.code() as i64 as f64).log(*source_value as f64))),
                    Data::Character(source_value) => return success!(float!((self_value.code() as i64 as f64).log(source_value.code() as i64 as f64))),
                    Data::Float(source_value) => return success!(float!((self_value.code() as i64 as f64).log(*source_value))),
                    _other => error!(Message, string!("characters may only be raised to the power of other characters or integers")),
                }
            }

            Data::Float(self_value) => {
                match source {
                    Data::Integer(source_value) => return success!(float!(self_value.log(*source_value as f64))),
                    Data::Character(source_value) => return success!(float!(self_value.log(source_value.code() as i64 as f64))),
                    Data::Float(source_value) => return success!(float!(self_value.log(*source_value))),
                    _other => error!(Message, string!("floats may only be raised to the power of other numbers")),
                }
            }

            _other => error!(Message, string!("only integers and characters may be raised to the power")),
        }
    }

    pub fn ceiling(&self) -> Status<Data> {
        match self {
            Data::Integer(value) => return success!(integer!(*value)),
            Data::Float(value) => return success!(integer!(value.ceil() as i64)),
            Data::Character(value) => return success!(integer!(value.code() as i64)),
            _invalid => return error!(Message, string!("only numbers may be converted to an integer")),
        }
    }

    pub fn floor(&self) -> Status<Data> {
        match self {
            Data::Integer(value) => return success!(integer!(*value)),
            Data::Float(value) => return success!(integer!(value.floor() as i64)),
            Data::Character(value) => return success!(integer!(value.code() as i64)),
            _invalid => return error!(Message, string!("only numbers may be converted to an integer")),
        }
    }

    pub fn sine(&self) -> Status<Data> {
        match self {
            Data::Integer(value) => return success!(float!((*value as f64).sin())),
            Data::Float(value) => return success!(float!(value.sin())),
            Data::Character(value) => return success!(float!((value.code() as i64 as f64).sin())),
            _invalid => return error!(Message, string!("only numbers may be converted to an integer")),
        }
    }

    pub fn cosine(&self) -> Status<Data> {
        match self {
            Data::Integer(value) => return success!(float!((*value as f64).cos())),
            Data::Float(value) => return success!(float!(value.cos())),
            Data::Character(value) => return success!(float!((value.code() as i64 as f64).cos())),
            _invalid => return error!(Message, string!("only numbers may be converted to an integer")),
        }
    }

    pub fn tangent(&self) -> Status<Data> {
        match self {
            Data::Integer(value) => return success!(float!((*value as f64).tan())),
            Data::Float(value) => return success!(float!(value.tan())),
            Data::Character(value) => return success!(float!((value.code() as i64 as f64).tan())),
            _invalid => return error!(Message, string!("only numbers may be converted to an integer")),
        }
    }

    pub fn and(&self, source: &Data) -> Status<Data> {
        match self {

            Data::Integer(self_value) => {
                match source {
                    Data::Integer(source_value) => return success!(integer!(self_value & source_value)),
                    Data::Character(source_value) => return success!(integer!(self_value & (source_value.code() as i64))),
                    _other => error!(Message, string!("integers may only be anded with other integers or characters")),
                }
            }

            Data::Character(self_value) => {
                match source {
                    Data::Integer(source_value) => return success!(integer!((self_value.code() as i64) & source_value)),
                    Data::Character(source_value) => return success!(integer!((self_value.code() as i64) & (source_value.code() as i64))),
                    _other => error!(Message, string!("characters may only be anded with other characters or integers")),
                }
            }

            Data::Boolean(self_value) => {
                match source {
                    Data::Boolean(source_value) => return success!(boolean!(*self_value && *source_value)),
                    _other => error!(Message, string!("booleans may only be anded with other booleans")),
                }
            }

            _other => error!(Message, string!("only integers, characters and booleans may be anded")),
        }
    }

    pub fn or(&self, source: &Data) -> Status<Data> {
        match self {

            Data::Integer(self_value) => {
                match source {
                    Data::Integer(source_value) => return success!(integer!(self_value | source_value)),
                    Data::Character(source_value) => return success!(integer!(self_value | (source_value.code() as i64))),
                    _other => error!(Message, string!("integers may only be ored with other integers or characters")),
                }
            }

            Data::Character(self_value) => {
                match source {
                    Data::Integer(source_value) => return success!(integer!((self_value.code() as i64) | source_value)),
                    Data::Character(source_value) => return success!(integer!((self_value.code() as i64) | (source_value.code() as i64))),
                    _other => error!(Message, string!("characters may only be ored with other characters or integers")),
                }
            }

            Data::Boolean(self_value) => {
                match source {
                    Data::Boolean(source_value) => return success!(boolean!(*self_value || *source_value)),
                    _other => error!(Message, string!("booleans may only be ored with other booleans")),
                }
            }

            _other => error!(Message, string!("only integers, characters and booleans may be ored")),
        }
    }

    pub fn xor(&self, source: &Data) -> Status<Data> {
        match self {

            Data::Integer(self_value) => {
                match source {
                    Data::Integer(source_value) => return success!(integer!(self_value ^ source_value)),
                    Data::Character(source_value) => return success!(integer!(self_value ^ (source_value.code() as i64))),
                    _other => error!(Message, string!("integers may only be xored with other integers or characters")),
                }
            }

            Data::Character(self_value) => {
                match source {
                    Data::Integer(source_value) => return success!(integer!((self_value.code() as i64) ^ source_value)),
                    Data::Character(source_value) => return success!(integer!((self_value.code() as i64) ^ (source_value.code() as i64))),
                    _other => error!(Message, string!("characters may only be xored with other characters or integers")),
                }
            }

            Data::Boolean(self_value) => {
                match source {
                    Data::Boolean(source_value) => return success!(boolean!(*self_value ^ *source_value)),
                    _other => error!(Message, string!("booleans may only be xored with other booleans")),
                }
            }

            _other => error!(Message, string!("only integers, characters and booleans may be xored")),
        }
    }

    pub fn not(&self) -> Status<Data> {
        match self {
            Data::Integer(self_value) => return success!(integer!(!*self_value)),
            Data::Character(self_value) => return success!(integer!(!(self_value.code() as i64))),
            Data::Boolean(self_value) => return success!(boolean!(!*self_value)),
            _other => error!(Message, string!("only integers, characters and booleans may be noted")),
        }
    }

    pub fn shift_left(&self, source: &Data) -> Status<Data> {
        match self {

            Data::Integer(self_value) => {
                match source {
                    Data::Integer(source_value) => return success!(integer!(self_value << source_value)),
                    Data::Character(source_value) => return success!(integer!(self_value << (source_value.code() as i64))),
                    _other => error!(Message, string!("only integers and characters may be shifted")),
                }
            }

            Data::Character(self_value) => {
                match source {
                    Data::Integer(source_value) => return success!(integer!((self_value.code() as i64) << source_value)),
                    Data::Character(source_value) => return success!(integer!((self_value.code() as i64) << (source_value.code() as i64))),
                    _other => error!(Message, string!("only integers and characters may be shifted")),
                }
            }

            _other => error!(Message, string!("only integers and characters may be shifted")),
        }
    }

    pub fn shift_right(&self, source: &Data) -> Status<Data> {
        match self {

            Data::Integer(self_value) => {
                match source {
                    Data::Integer(source_value) => return success!(integer!(self_value >> source_value)),
                    Data::Character(source_value) => return success!(integer!(self_value >> (source_value.code() as i64))),
                    _other => error!(Message, string!("only integers and characters may be shifted")),
                }
            }

            Data::Character(self_value) => {
                match source {
                    Data::Integer(source_value) => return success!(integer!((self_value.code() as i64) >> source_value)),
                    Data::Character(source_value) => return success!(integer!((self_value.code() as i64) >> (source_value.code() as i64))),
                    _other => error!(Message, string!("only integers and characters may be shifted")),
                }
            }

            _other => error!(Message, string!("only integers and characters may be shifted")),
        }
    }

    pub fn add(&self, source: &Data) -> Status<Data> {
        match self {

            Data::Integer(self_value) => {
                match source {
                    Data::Integer(source_value) => return success!(integer!(self_value + source_value)),
                    Data::Character(source_value) => return success!(integer!(self_value + (source_value.code() as i64))),
                    Data::Float(source_value) => return success!(float!((*self_value as f64) + source_value)),
                    _other => error!(Message, string!("only numbers may be added")),
                }
            }

            Data::Character(self_value) => {
                match source {
                    Data::Integer(source_value) => return success!(integer!((self_value.code() as i64) + source_value)),
                    Data::Character(source_value) => return success!(integer!((self_value.code() as i64) + (source_value.code() as i64))),
                    Data::Float(source_value) => return success!(float!((self_value.code() as i64 as f64) + source_value)),
                    _other => error!(Message, string!("only numbers may be added")),
                }
            }

            Data::Float(self_value) => {
                match source {
                    Data::Integer(source_value) => return success!(float!(self_value + (*source_value as f64))),
                    Data::Character(source_value) => return success!(float!(self_value + (source_value.code() as i64 as f64))),
                    Data::Float(source_value) => return success!(float!(self_value + source_value)),
                    _other => error!(Message, string!("only numbers may be added")),
                }
            }

            _other => error!(Message, string!("only numbers may be added")),
        }
    }

    pub fn subtract(&self, source: &Data) -> Status<Data> {
        match self {

            Data::Integer(self_value) => {
                match source {
                    Data::Integer(source_value) => return success!(integer!(self_value - source_value)),
                    Data::Character(source_value) => return success!(integer!(self_value - (source_value.code() as i64))),
                    Data::Float(source_value) => return success!(float!((*self_value as f64) - source_value)),
                    _other => error!(Message, string!("only numbers may be subtracted")),
                }
            }

            Data::Character(self_value) => {
                match source {
                    Data::Integer(source_value) => return success!(integer!((self_value.code() as i64) - source_value)),
                    Data::Character(source_value) => return success!(integer!((self_value.code() as i64) - (source_value.code() as i64))),
                    Data::Float(source_value) => return success!(float!((self_value.code() as i64 as f64) - source_value)),
                    _other => error!(Message, string!("only numbers may be subtracted")),
                }
            }

            Data::Float(self_value) => {
                match source {
                    Data::Integer(source_value) => return success!(float!(self_value - (*source_value as f64))),
                    Data::Character(source_value) => return success!(float!(self_value - (source_value.code() as i64 as f64))),
                    Data::Float(source_value) => return success!(float!(self_value - source_value)),
                    _other => error!(Message, string!("only numbers may be subtracted")),
                }
            }

            _other => error!(Message, string!("only numbers may be subtracted")),
        }
    }

    pub fn multiply(&self, source: &Data) -> Status<Data> {
        match self {

            Data::Integer(self_value) => {
                match source {
                    Data::Integer(source_value) => return success!(integer!(self_value * source_value)),
                    Data::Character(source_value) => return success!(integer!(self_value * (source_value.code() as i64))),
                    Data::Float(source_value) => return success!(float!((*self_value as f64) * source_value)),
                    _other => error!(Message, string!("only numbers may be multiplied")),
                }
            }

            Data::Character(self_value) => {
                match source {
                    Data::Integer(source_value) => return success!(integer!((self_value.code() as i64) * source_value)),
                    Data::Character(source_value) => return success!(integer!((self_value.code() as i64) * (source_value.code() as i64))),
                    Data::Float(source_value) => return success!(float!((self_value.code() as i64 as f64) * source_value)),
                    _other => error!(Message, string!("only numbers may be multiplied")),
                }
            }

            Data::Float(self_value) => {
                match source {
                    Data::Integer(source_value) => return success!(float!(self_value * (*source_value as f64))),
                    Data::Character(source_value) => return success!(float!(self_value * (source_value.code() as i64 as f64))),
                    Data::Float(source_value) => return success!(float!(self_value * source_value)),
                    _other => error!(Message, string!("only numbers may be multiplied")),
                }
            }

            _other => error!(Message, string!("only numbers may be multiplied")),
        }
    }

    pub fn divide(&self, source: &Data) -> Status<Data> {
        match self {

            Data::Integer(self_value) => {
                match source {
                    Data::Integer(source_value) => return success!(integer!(self_value / source_value)),
                    Data::Character(source_value) => return success!(integer!(self_value / (source_value.code() as i64))),
                    Data::Float(source_value) => return success!(float!((*self_value as f64) / source_value)),
                    _other => error!(Message, string!("only numbers may be divided")),
                }
            }

            Data::Character(self_value) => {
                match source {
                    Data::Integer(source_value) => return success!(integer!((self_value.code() as i64) / source_value)),
                    Data::Character(source_value) => return success!(integer!((self_value.code() as i64) / (source_value.code() as i64))),
                    Data::Float(source_value) => return success!(float!((self_value.code() as i64 as f64) / source_value)),
                    _other => error!(Message, string!("only numbers may be divided")),
                }
            }

            Data::Float(self_value) => {
                match source {
                    Data::Integer(source_value) => return success!(float!(self_value / (*source_value as f64))),
                    Data::Character(source_value) => return success!(float!(self_value / (source_value.code() as i64 as f64))),
                    Data::Float(source_value) => return success!(float!(self_value / source_value)),
                    _other => error!(Message, string!("only numbers may be divided")),
                }
            }

            _other => error!(Message, string!("only numbers may be divided")),
        }
    }

    pub fn modulo(&self, source: &Data) -> Status<Data> {
        match self {

            Data::Integer(self_value) => {
                match source {
                    Data::Integer(source_value) => return success!(integer!(self_value % source_value)),
                    Data::Character(source_value) => return success!(integer!(self_value % (source_value.code() as i64))),
                    Data::Float(source_value) => return success!(float!((*self_value as f64) % source_value)),
                    _other => error!(Message, string!("modulo operation may only be performed on numbers")),
                }
            }

            Data::Character(self_value) => {
                match source {
                    Data::Integer(source_value) => return success!(integer!((self_value.code() as i64) % source_value)),
                    Data::Character(source_value) => return success!(integer!((self_value.code() as i64) % (source_value.code() as i64))),
                    Data::Float(source_value) => return success!(float!((self_value.code() as i64 as f64) % source_value)),
                    _other => error!(Message, string!("modulo operation may only be performed on numbers")),
                }
            }

            Data::Float(self_value) => {
                match source {
                    Data::Integer(source_value) => return success!(float!(self_value % (*source_value as f64))),
                    Data::Character(source_value) => return success!(float!(*self_value % (source_value.code() as i64 as f64))),
                    Data::Float(source_value) => return success!(float!(self_value % source_value)),
                    _other => error!(Message, string!("modulo operation may only be performed on numbers")),
                }
            }

            _other => error!(Message, string!("modulo operation may only be performed on numbers")),
        }
    }

    fn realation_size(&self) -> Status<usize> {
        match self {
            Data::Integer(integer) => return success!(*integer as usize),
            Data::Character(character) => return success!(character.code() as usize),
            Data::Float(float) => return success!(*float as usize),
            Data::Map(map) => return success!(map.len()),
            Data::List(items) => return success!(items.len()),
            Data::Path(steps) => return success!(steps.len()),
            Data::String(string) => return success!(string.len()),
            Data::Identifier(identifier) => return success!(identifier.len()),
            Data::Keyword(keyword) => return success!(keyword.len()),
            Data::Boolean(..) => return error!(Message, string!("booleans may not be comapred by size")),
        }
    }

    pub fn bigger(&self, other: &Data) -> Status<bool> {
        return success!(confirm!(self.realation_size()) > confirm!(other.realation_size()));
    }

    pub fn smaller(&self, other: &Data) -> Status<bool> {
        return success!(confirm!(self.realation_size()) < confirm!(other.realation_size()));
    }

    pub fn contains(&self, source: &Data) -> Status<bool> {
        match self {
            Data::Map(map) => return success!(map.contains(source)),
            Data::Path(steps) => return success!(steps.contains(source)),
            Data::List(items) => return success!(items.contains(source)),
            Data::String(string) => return success!(string.contains(&unpack_literal!(source))),
            Data::Identifier(identifier) => return success!(identifier.contains(&unpack_literal!(source))),
            Data::Keyword(keyword) => return success!(keyword.contains(&unpack_literal!(source))),
            _invalid => return error!(ExpectedFound, expected_list!["container"], self.clone()),
        }
    }

    pub fn flip(&self) -> Status<Data> {
        match self {

            Data::Path(steps) => return success!(path!(steps.flip())),

            Data::List(items) => return success!(list!(items.flip())),

            Data::String(string) => return success!(string!(String, string.flip())),

            Data::Identifier(identifier) => return success!(identifier!(String, identifier.flip())),

            Data::Keyword(keyword) => return success!(keyword!(String, keyword.flip())),

            _ => return error!(ExpectedFound, expected_list!["container"], self.clone()), // FIX MEEEE
        }
    }

    pub fn position(&self, source: &Data) -> Status<Data> {
        match self {

            Data::Map(map) => return success!(list!(map.position(source))),

            Data::Path(steps) => return success!(list!(steps.position(source).iter().map(|index| integer!(*index as i64 + 1)).collect())),

            Data::List(steps) => return success!(list!(steps.position(source).iter().map(|index| integer!(*index as i64 + 1)).collect())),

            Data::String(string) => return success!(list!(string.position(&unpack_literal!(source)).iter().map(|index| integer!(*index as i64 + 1)).collect())),

            Data::Identifier(identifier) => return success!(list!(identifier.position(&unpack_literal!(source)).iter().map(|index| integer!(*index as i64 + 1)).collect())),

            Data::Keyword(keyword) => return success!(list!(keyword.position(&unpack_literal!(source)).iter().map(|index| integer!(*index as i64 + 1)).collect())),

            _invalid => return error!(ExpectedFound, expected_list!["container"], self.clone()),
        }
    }

    pub fn split(&self, source: &Data, void: &Data) -> Status<Data> {
        // maybe put this somewhere else
        let void = unpack_boolean!(void);

        match self {

            Data::Path(steps) => {
                let mut pieces = SharedVector::new();
                for piece in steps.split(source, void).into_iter() {
                    // ensure piece is at least 2 long
                    pieces.push(path!(piece));
                }
                return success!(list!(pieces));
            },

            Data::List(items) => {
                let pieces = items.split(source, void).into_iter().map(|piece| list!(piece)).collect();
                return success!(list!(pieces));
            },

            Data::String(string) => {
                let literal = unpack_literal!(source);
                ensure!(!literal.is_empty(), Message, string!("empty literal"));
                let pieces = string.split(&literal, void).into_iter().map(|piece| string!(String, piece)).collect();
                return success!(list!(pieces));
            },

            Data::Identifier(identifier) => {
                let literal = unpack_literal!(source);
                ensure!(!literal.is_empty(), Message, string!("empty literal"));
                let mut pieces = SharedVector::new();
                for piece in identifier.split(&literal, void).into_iter() {
                    // ensure piece is at least 1 long
                    // ensure piece is at pure
                    pieces.push(identifier!(String, piece));
                }
                return success!(list!(pieces));
            },

            Data::Keyword(keyword) => {
                let literal = unpack_literal!(source);
                ensure!(!literal.is_empty(), Message, string!("empty literal"));
                let mut pieces = SharedVector::new();
                for piece in keyword.split(&literal, void).into_iter() {
                    // ensure piece is at least 1 long
                    // ensure piece is at pure
                    pieces.push(keyword!(String, piece));
                }
                return success!(list!(pieces));
            },

            _invalid => return error!(ExpectedFound, expected_list!["container"], self.clone()),
        }
    }

    pub fn is_uppercase(&self) -> Status<bool> {
        match self {

            Data::String(string) => return success!(string.is_uppercase()),

            Data::Identifier(identifier) => return success!(identifier.is_uppercase()),

            Data::Keyword(keyword) => return success!(keyword.is_uppercase()),

            Data::Character(character) => return success!(character.is_uppercase()),

            _invalid => return error!(ExpectedFound, expected_list!["literal"], self.clone()),
        }
    }

    pub fn is_lowercase(&self) -> Status<bool> {
        match self {

            Data::String(string) => return success!(string.is_lowercase()),

            Data::Identifier(identifier) => return success!(identifier.is_lowercase()),

            Data::Keyword(keyword) => return success!(keyword.is_lowercase()),

            Data::Character(character) => return success!(character.is_lowercase()),

            _invalid => return error!(ExpectedFound, expected_list!["literal"], self.clone()),
        }
    }

    pub fn replace(&self, from: &Data, to: &Data) -> Status<Data> {
        match self {

            Data::Map(map) => return success!(map!(map.replace(from, to))),

            Data::Path(steps) => return success!(path!(steps.replace(from, to))),

            Data::List(items) => return success!(list!(items.replace(from, to))),

            Data::String(string) => return success!(string!(String, string.replace(&unpack_literal!(from), &unpack_literal!(to)))),

            Data::Identifier(identifier) => return success!(identifier!(String, identifier.replace(&unpack_literal!(from), &unpack_literal!(to)))),

            Data::Keyword(keyword) => return success!(keyword!(String, keyword.replace(&unpack_literal!(from), &unpack_literal!(to)))),

            _invalid => return error!(ExpectedFound, expected_list!["container"], self.clone()),
        }
    }

    pub fn index_reference(&self, selector: &Data, mutable: bool, create: bool) -> Status<IndexResult> {
        let mut_self = self as *const Data as *mut Data;
        match unsafe { &mut *mut_self } {

            Data::Map(map) => {
                match selector {

                    Data::Path(steps) => {
                        let mut last = self as *const Data;
                        for (step_index, step) in steps.iter().enumerate() {
                            match unsafe { confirm!((*last).index_reference(&step, mutable, create && step_index == steps.len() - 1)) } {
                                IndexResult::Reference(reference) => last = reference,
                                IndexResult::Literal(reference, index) => {
                                    ensure!(step_index == steps.len() - 1, Message, string!("cannot index a character"));
                                    return success!(IndexResult::Literal(reference, index));
                                }
                                IndexResult::Missed => return success!(IndexResult::Missed),
                            }
                        }
                        return success!(IndexResult::Reference(last));
                    },

                    _other => {
                        if mutable {
                            if let Some(entry) = map.get_mut(selector) {
                                return success!(IndexResult::Reference(entry as *const Data));
                            }
                        } else {
                            if let Some(entry) = map.get(selector) {
                                return success!(IndexResult::Reference(entry as *const Data));
                            }
                        }

                        if create {
                            map.insert(selector.clone(), integer!(0));
                            let entry = map.get_mut(selector).unwrap();
                            return success!(IndexResult::Reference(entry as *const Data));
                        }

                        return success!(IndexResult::Missed);
                    },
                }
            },

            Data::List(items) => {
                match selector {

                    Data::Path(steps) => {
                        let mut last = self as *const Data;
                        for (step_index, step) in steps.iter().enumerate() {
                            match unsafe { confirm!((*last).index_reference(&step, mutable, create && step_index == steps.len() - 1)) } {
                                IndexResult::Reference(reference) => last = reference,
                                IndexResult::Literal(reference, index) => {
                                    ensure!(step_index == steps.len() - 1, Message, string!("cannot index a character"));
                                    return success!(IndexResult::Literal(reference, index));
                                }
                                IndexResult::Missed => return success!(IndexResult::Missed),
                            }
                        }
                        return success!(IndexResult::Reference(last));
                    },

                    _other => {
                        if mutable {
                            match confirm!(Data::wrapped_index(selector, items.len())) {
                                Some(selector) => return success!(IndexResult::Reference((&mut items[selector]) as *const Data)),
                                None => return success!(IndexResult::Missed),
                            }
                        } else {
                            match confirm!(Data::wrapped_index(selector, items.len())) {
                                Some(selector) => return success!(IndexResult::Reference((&items[selector]) as *const Data)),
                                None => return success!(IndexResult::Missed),
                            }
                        }
                    }
                }
            },

            Data::Path(steps) => {
                match selector {

                    Data::Path(steps) => {
                        let mut last = self as *const Data;
                        for (step_index, step) in steps.iter().enumerate() {
                            match unsafe { confirm!((*last).index_reference(&step, mutable, create && step_index == steps.len() - 1)) } {
                                IndexResult::Reference(reference) => last = reference,
                                IndexResult::Literal(reference, index) => {
                                    ensure!(step_index == steps.len() - 1, Message, string!("cannot index a character"));
                                    return success!(IndexResult::Literal(reference, index));
                                }
                                IndexResult::Missed => return success!(IndexResult::Missed),
                            }
                        }
                        return success!(IndexResult::Reference(last));
                    },

                    _other => {
                        if mutable {
                            match confirm!(Data::wrapped_index(selector, steps.len())) {
                                Some(selector) => return success!(IndexResult::Reference((&mut steps[selector]) as *const Data)),
                                None => return success!(IndexResult::Missed),
                            }
                        } else {
                            match confirm!(Data::wrapped_index(selector, steps.len())) {
                                Some(selector) => return success!(IndexResult::Reference((&steps[selector]) as *const Data)),
                                None => return success!(IndexResult::Missed),
                            }
                        }
                    }
                }
            }

            Data::String(string) => {
                match confirm!(Data::wrapped_index(selector, string.len())) {
                    Some(selector) => return success!(IndexResult::Literal(self as *const Data, selector)),
                    None => return success!(IndexResult::Missed),
                }
            }

            Data::Identifier(identifier) => {
                match confirm!(Data::wrapped_index(selector, identifier.len())) {
                    Some(selector) => return success!(IndexResult::Literal(self as *const Data, selector)),
                    None => return success!(IndexResult::Missed),
                }
            }

            Data::Keyword(keyword) => {
                match confirm!(Data::wrapped_index(selector, keyword.len())) {
                    Some(selector) => return success!(IndexResult::Literal(self as *const Data, selector)),
                    None => return success!(IndexResult::Missed),
                }
            }

            _other => return error!(ExpectedFound, expected_list!["container"], self.clone()),
        }
    }

    pub fn index(&self, selector: &Data) -> Status<Option<Data>> {
        unsafe {
            match confirm!(self.index_reference(selector, false, false)) {
                IndexResult::Reference(reference) => return success!(Some((*reference).clone())),
                IndexResult::Literal(reference, index) => {
                    match &*reference {
                        Data::Keyword(keyword) => return success!(Some(character!(keyword[index]))),
                        Data::Identifier(identifier) => return success!(Some(character!(identifier[index]))),
                        Data::String(string) => return success!(Some(character!(string[index]))),
                        _invalid => panic!(),
                    }
                }
                IndexResult::Missed => return success!(None),
            }
        }
    }

    pub fn slice(&self, start: &Data, end: &Data) -> Status<Data> {
        match self {

            Data::Path(steps) => {
                let start_index = confirm!(Data::wrapped_index(start, steps.len()));
                let start_index = expect!(start_index, IndexOutOfBounds, start.clone(), integer!(steps.len() as i64));
                let end_index = confirm!(Data::wrapped_index(end, steps.len()));
                let end_index = expect!(end_index, IndexOutOfBounds, end.clone(), integer!(steps.len() as i64));
                return success!(path!(steps.slice(start_index, end_index)));
            },

            Data::List(items) => {
                let start_index = confirm!(Data::wrapped_index(start, items.len()));
                let start_index = expect!(start_index, IndexOutOfBounds, start.clone(), integer!(items.len() as i64));
                let end_index = confirm!(Data::wrapped_index(end, items.len()));
                let end_index = expect!(end_index, IndexOutOfBounds, end.clone(), integer!(items.len() as i64));
                return success!(list!(items.slice(start_index, end_index)));
            },

            Data::String(string) => {
                let start_index = confirm!(Data::wrapped_index(start, string.len()));
                let start_index = expect!(start_index, IndexOutOfBounds, start.clone(), integer!(string.len() as i64));
                let end_index = confirm!(Data::wrapped_index(end, string.len()));
                let end_index = expect!(end_index, IndexOutOfBounds, end.clone(), integer!(string.len() as i64));
                return success!(string!(String, string.slice(start_index, end_index)));
            },

            Data::Identifier(identifier) => {
                let start_index = confirm!(Data::wrapped_index(start, identifier.len()));
                let start_index = expect!(start_index, IndexOutOfBounds, start.clone(), integer!(identifier.len() as i64));
                let end_index = confirm!(Data::wrapped_index(end, identifier.len()));
                let end_index = expect!(end_index, IndexOutOfBounds, end.clone(), integer!(identifier.len() as i64));
                return success!(identifier!(String, identifier.slice(start_index, end_index)));
            },

            Data::Keyword(keyword) => {
                let start_index = confirm!(Data::wrapped_index(start, keyword.len()));
                let start_index = expect!(start_index, IndexOutOfBounds, start.clone(), integer!(keyword.len() as i64));
                let end_index = confirm!(Data::wrapped_index(end, keyword.len()));
                let end_index = expect!(end_index, IndexOutOfBounds, end.clone(), integer!(keyword.len() as i64));
                return success!(keyword!(String, keyword.slice(start_index, end_index)));
            },

            _ => return error!(ExpectedFound, expected_list!["container"], self.clone()), // wrong if map is not valid
        }
    }

    pub fn insert(&self, selector: &Data, value: Data) -> Status<Data> {
        match self {

            Data::Map(map) => {
                let mut new_map = map.clone();
                match new_map.insert(selector.clone(), value) {
                    Some(previous_data) => return error!(InexplicitOverwrite, selector.clone(), previous_data),
                    None => return success!(map!(new_map)),
                }
            },

            Data::Path(steps) => {
                let index = confirm!(Data::wrapped_index(selector, steps.len() + 1));
                let index = expect!(index, IndexOutOfBounds, selector.clone(), integer!(steps.len() as i64));
                let mut new_steps = steps.clone();
                match index == new_steps.len() {
                    true => new_steps.push(value),
                    false => new_steps.insert(index, value),
                }
                return success!(path!(new_steps));
            },

            Data::List(items) => {
                let index = confirm!(Data::wrapped_index(selector, items.len() + 1));
                let index = expect!(index, IndexOutOfBounds, selector.clone(), integer!(items.len() as i64));
                let mut new_items = items.clone();
                match index == new_items.len() {
                    true => new_items.push(value),
                    false => new_items.insert(index, value),
                }
                return success!(list!(new_items));
            },

            Data::String(string) => {
                let index = confirm!(Data::wrapped_index(selector, string.len() + 1));
                let index = expect!(index, IndexOutOfBounds, selector.clone(), integer!(string.len() as i64));
                let literal = unpack_literal!(&value);
                let mut new_string = string.clone();
                match index == new_string.len() {
                    true => new_string.push_str(&literal),
                    false => new_string.insert_str(index, &literal),
                }
                return success!(string!(String, new_string));
            },

            Data::Identifier(identifier) => {
                let index = confirm!(Data::wrapped_index(selector, identifier.len() + 1));
                let index = expect!(index, IndexOutOfBounds, selector.clone(), integer!(identifier.len() as i64));
                let literal = unpack_literal!(&value);
                let mut new_identifier = identifier.clone();
                match index == new_identifier.len() {
                    true => new_identifier.push_str(&literal),
                    false => new_identifier.insert_str(index, &literal),
                }
                return success!(identifier!(String, new_identifier));
            },

            Data::Keyword(keyword) => {
                let index = confirm!(Data::wrapped_index(selector, keyword.len() + 1));
                let index = expect!(index, IndexOutOfBounds, selector.clone(), integer!(keyword.len() as i64));
                let literal = unpack_literal!(&value);
                let mut new_keyword = keyword.clone();
                match index == new_keyword.len() {
                    true => new_keyword.push_str(&literal),
                    false => new_keyword.insert_str(index, &literal),
                }
                return success!(keyword!(String, new_keyword));
            },

            _ => return error!(ExpectedFound, expected_list!["container"], self.clone()),

        }
    }

    pub fn overwrite(&self, selector: &Data, value: Data) -> Status<Data> {
        match self {

            Data::Map(map) => {
                let mut new_map = map.clone();
                new_map.insert(selector.clone(), value);
                return success!(map!(new_map));
            },

            Data::Path(steps) => {
                let index = confirm!(Data::wrapped_index(selector, steps.len() + 1));
                let index = expect!(index, IndexOutOfBounds, selector.clone(), integer!(steps.len() as i64));
                let mut new_steps = steps.clone();
                match index == new_steps.len() {
                    true => new_steps.push(value),
                    false => new_steps[index] = value,
                }
                return success!(path!(new_steps));
            },

            Data::List(items) => {
                let index = confirm!(Data::wrapped_index(selector, items.len() + 1));
                let index = expect!(index, IndexOutOfBounds, selector.clone(), integer!(items.len() as i64));
                let mut new_items = items.clone();
                match index == new_items.len() {
                    true => new_items.push(value),
                    false => new_items[index] = value,
                }
                return success!(list!(new_items));
            },

            Data::String(string) => {
                let index = confirm!(Data::wrapped_index(selector, string.len() + 1));
                let index = expect!(index, IndexOutOfBounds, selector.clone(), integer!(string.len() as i64));
                let mut new_string = string.clone();
                let literal = unpack_literal!(&value);
                if index == new_string.len() {
                    new_string.push_str(&literal);
                } else {
                    new_string.remove(index);
                    new_string.insert_str(index, &literal);
                }
                return success!(string!(String, new_string));
            },

            Data::Identifier(identifier) => {
                let index = confirm!(Data::wrapped_index(selector, identifier.len() + 1));
                let index = expect!(index, IndexOutOfBounds, selector.clone(), integer!(identifier.len() as i64));
                let mut new_identifier = identifier.clone();
                let literal = unpack_literal!(&value);
                if index == new_identifier.len() {
                    new_identifier.push_str(&literal);
                } else {
                    new_identifier.remove(index);
                    new_identifier.insert_str(index, &literal);
                }
                return success!(identifier!(String, new_identifier));
            },

            Data::Keyword(keyword) => {
                let index = confirm!(Data::wrapped_index(selector, keyword.len() + 1));
                let index = expect!(index, IndexOutOfBounds, selector.clone(), integer!(keyword.len() as i64));
                let mut new_keyword = keyword.clone();
                let literal = unpack_literal!(&value);
                if index == new_keyword.len() {
                    new_keyword.push_str(&literal);
                } else {
                    new_keyword.remove(index);
                    new_keyword.insert_str(index, &literal);
                }
                return success!(keyword!(String, new_keyword));
            },

            _ => return error!(ExpectedFound, expected_list!["container"], self.clone()),

        }
    }

    pub fn keys(&self) -> Status<Data> {
        match self {
            Data::Map(map) => return success!(list!(map.keys().cloned().collect())),
            invalid => return error!(ExpectedFound, expected_list!["container"], invalid.clone()),
        }
    }

    pub fn values(&self) -> Status<Data> {
        match self {
            Data::Map(map) => return success!(list!(map.values().cloned().collect())),
            Data::List(items) => return success!(list!(items.iter().cloned().collect())),
            Data::Path(steps) => return success!(list!(steps.iter().cloned().collect())),
            Data::String(string) => return success!(list!(string.chars().map(|character| character!(*character)).collect())),
            Data::Identifier(identifier) => return success!(list!(identifier.chars().map(|character| character!(*character)).collect())),
            Data::Keyword(keyword) => return success!(list!(keyword.chars().map(|character| character!(*character)).collect())),
            invalid => return error!(ExpectedFound, expected_list!["container"], invalid.clone()),
        }
    }

    pub fn pairs(&self) -> Status<Vec<(Data, Data)>> {
        let mut pairs = Vec::new();
        match self {

            Data::Map(map) => pairs = map.clone().drain().collect(),

            Data::List(items) => {
                for (index, item) in items.iter().enumerate() {
                    let index = integer!(index as i64 + 1);
                    pairs.push((index, item.clone()));
                }
            },

            Data::Path(steps) => {
                for (index, step) in steps.iter().enumerate() {
                    let index = integer!(index as i64 + 1);
                    pairs.push((index, step.clone()));
                }
            },

            Data::String(string) => {
                for (index, character) in string.chars().enumerate() {
                    let index = integer!(index as i64 + 1);
                    pairs.push((index, character!(character.clone())));
                }
            },

            Data::Identifier(identifier) => {
                for (index, character) in identifier.chars().enumerate() {
                    let index = integer!(index as i64 + 1);
                    pairs.push((index, character!(character.clone())));
                }
            },

            Data::Keyword(keyword) => {
                for (index, character) in keyword.chars().enumerate() {
                    let index = integer!(index as i64 + 1);
                    pairs.push((index, character!(character.clone())));
                }
            },

            _ => return error!(ExpectedFound, expected_list!["container"], self.clone()),
        }
        return success!(pairs);
    }

    pub fn pass(&self, pass: &Pass, root: &Data, build: &Data) -> Status<Data> {
        match self {

            Data::Map(map) => {
                if let Some(mut pass_map) = confirm!(self.index(&keyword!("pass"))) {
                    if let Some(pass_handlers) = confirm!(pass_map.index(&pass.name)) {
                        confirm!(pass_map.set_entry(&pass.name, list!(), true)); // THINK ABOUT THIS SOME MORE
                        let mut new_self = confirm!(self.overwrite(&keyword!("pass"), pass_map));

                        for pass_handler_path in unpack_list!(&pass_handlers).into_iter() {
                            let function_map = index_field!(root, "function");
                            let pass_handler = index!(&function_map, &pass_handler_path);
                            let mut parameters: SharedVector<Data> = pass.parameters.iter().cloned().collect();
                            parameters.insert(0, new_self.clone());
                            let last = confirm!(function(&pass_handler, parameters, &Some(pass.clone()), root, build));
                            new_self = expect!(last, Message, string!("pass didnt return a value"));
                        }

                        return success!(new_self);
                    }
                }

                let mut new_map = Map::new();
                for (key, value) in map.iter() {
                    new_map.insert(key.clone(), confirm!(value.pass(pass, root, build)));
                }
                return success!(map!(new_map));
            },

            Data::List(items) => {
                let mut new_items = SharedVector::new();
                for item in items.iter() {
                    new_items.push(confirm!(item.pass(pass, root, build)));
                }
                return success!(list!(new_items));
            },

            other => return success!(other.clone()),
        }
    }

    pub fn remove(&self, selector: &Data) -> Status<Data> {
        match self {

            Data::Map(map) => {
                let mut new_map = map.clone();
                match new_map.remove(&selector) {
                    Some(..) => return success!(map!(new_map)),
                    None => return error!(MissingEntry, selector.clone()),
                }
            },

            Data::Path(steps) => {
                let index = confirm!(Data::wrapped_index(selector, steps.len()));
                let index = expect!(index, IndexOutOfBounds, selector.clone(), integer!(steps.len() as i64));
                let mut new_steps = steps.clone();
                new_steps.remove(index);
                ensure!(new_steps.len() >= 2, Message, string!("path needs at least two steps"));
                return success!(path!(new_steps));
            },

            Data::List(items) => {
                let index = confirm!(Data::wrapped_index(selector, items.len()));
                let index = expect!(index, IndexOutOfBounds, selector.clone(), integer!(items.len() as i64));
                let mut new_items = items.clone();
                new_items.remove(index);
                return success!(list!(new_items));
            },

            Data::String(string) => {
                let index = confirm!(Data::wrapped_index(selector, string.len()));
                let index = expect!(index, IndexOutOfBounds, selector.clone(), integer!(string.len() as i64));
                let mut new_string = string.clone();
                new_string.remove(index);
                return success!(string!(String, new_string));
            },

            Data::Identifier(identifier) => {
                let index = confirm!(Data::wrapped_index(selector, identifier.len()));
                let index = expect!(index, IndexOutOfBounds, selector.clone(), integer!(identifier.len() as i64));
                let mut new_identifier = identifier.clone();
                new_identifier.remove(index);
                ensure!(!new_identifier.is_empty(), Message, string!("identifier may not be empty"));
                return success!(identifier!(String, new_identifier));
            },

            Data::Keyword(keyword) => {
                let index = confirm!(Data::wrapped_index(selector, keyword.len()));
                let index = expect!(index, IndexOutOfBounds, selector.clone(), integer!(keyword.len() as i64));
                let mut new_keyword = keyword.clone();
                new_keyword.remove(index);
                ensure!(!new_keyword.is_empty(), Message, string!("keyword may not be empty"));
                return success!(keyword!(String, new_keyword));
            },

            _ => return error!(ExpectedFound, expected_list!["container"], self.clone()),
        }
    }

    pub fn serialize(&self) -> SharedString {
        match self {
            Data::Map(map) => serialize_map(map),
            Data::List(items) => serialize_list(items),
            Data::Path(steps) => serialize_path(steps),
            Data::Integer(integer) => SharedString::from(&integer.to_string()),
            Data::Float(float) => serialize_float(*float),
            Data::Identifier(identifier) => identifier.clone(),
            Data::Keyword(keyword) => format_shared!("#{}", keyword),
            Data::String(string) => serialize_literal(string, '\"'),
            Data::Character(character) => serialize_literal(&character.to_string(), '\''),
            Data::Boolean(boolean) => format_shared!("${}", boolean_to_string!(boolean)),
        }
    }

    pub fn set_entry(&mut self, key: &Data, data: Data, overwrite: bool) -> Status<bool> {
        if let Data::Map(ref mut map) = self {
            if let Some(entry) = map.get_mut(&key) {
                if overwrite {
                    *entry = data;
                }
                return success!(true);
            } else {
                map.insert(key.clone(), data);
                return success!(false);
            }
        }
        return error!(Message, string!("set_entry must be called on a map"));
    }

    pub fn modify(&self, path: Option<&Data>, data: Data) -> Status<()> {
        if let Some(path) = path {
            match confirm!(self.index_reference(path, true, true)) {

                IndexResult::Reference(reference) => {
                    let reference = reference as *mut Data;
                    unsafe { *reference = data; }
                    return success!(());
                }

                IndexResult::Literal(reference, index) => {
                    let new_value = unpack_character!(&data);
                    let reference = reference as *mut Data;
                    match unsafe { &mut *reference } {
                        Data::Keyword(keyword) => keyword[index] = new_value, // MAKE SURE THIS IS PURE
                        Data::Identifier(identifier) => identifier[index] = new_value, // MAKE SURE THIS IS PURE
                        Data::String(string) => string[index] = new_value,
                        _invalid => panic!(),
                    }
                    return success!(());
                }

                IndexResult::Missed => return error!(Message, string!("missing entry from modify")),
            }
        } else {
            let reference = self as *const Data as *mut Data;
            unsafe { *reference = data; }
            return success!(());
        }
    }

    pub fn is_map(&self) -> bool {
        match self {
            Data::Map(..) => return true,
            _other => return false,
        };
    }

    pub fn is_list(&self) -> bool {
        match self {
            Data::List(..) => return true,
            _other => return false,
        };
    }

    pub fn is_path(&self) -> bool {
        match self {
            Data::Path(..) => return true,
            _other => return false,
        };
    }

    pub fn is_string(&self) -> bool {
        match self {
            Data::String(..) => return true,
            _other => return false,
        };
    }

    pub fn is_character(&self) -> bool {
        match self {
            Data::Character(..) => return true,
            _other => return false,
        };
    }

    pub fn is_identifier(&self) -> bool {
        match self {
            Data::Identifier(..) => return true,
            _other => return false,
        };
    }

    pub fn is_keyword(&self) -> bool {
        match self {
            Data::Keyword(..) => return true,
            _other => return false,
        };
    }

    pub fn is_integer(&self) -> bool {
        match self {
            Data::Integer(..) => return true,
            _other => return false,
        };
    }

    pub fn is_float(&self) -> bool {
        match self {
            Data::Float(..) => return true,
            _other => return false,
        };
    }

    pub fn is_boolean(&self) -> bool {
        match self {
            Data::Boolean(..) => return true,
            _other => return false,
        };
    }

    pub fn is_key(&self) -> bool {
        match self {
            Data::Identifier(_) => return true,
            Data::Keyword(_) => return true,
            Data::String(_) => return true,
            Data::Character(_) => return true,
            Data::Boolean(_) => return true,
            _other => return false,
        };
    }

    pub fn is_container(&self) -> bool {
        match self {
            Data::Map(_) => return true,
            Data::List(_) => return true,
            Data::Path(_) => return true,
            Data::String(_) => return true,
            Data::Identifier(_) => return true,
            Data::Keyword(_) => return true,
            _other => return false,
        };
    }

    pub fn is_literal(&self) -> bool {
        match self {
            Data::String(_) => return true,
            Data::Character(_) => return true,
            Data::Identifier(_) => return true,
            Data::Keyword(_) => return true,
            _other => return false,
        };
    }

    pub fn is_selector(&self) -> bool {
        match self {
            Data::Identifier(_) => return true,
            Data::Keyword(_) => return true,
            Data::String(_) => return true,
            Data::Character(_) => return true,
            Data::Integer(_) => return true,
            Data::Boolean(_) => return true,
            _other => return false,
        };
    }

    pub fn is_number(&self) -> bool {
        match self {
            Data::Integer(_) => return true,
            Data::Float(_) => return true,
            Data::Character(_) => return true,
            _other => return false,
        };
    }

    pub fn is_location(&self) -> bool {
        match self {
            Data::Path(_) => return true,
            Data::Identifier(_) => return true,
            Data::Keyword(_) => return true,
            Data::String(_) => return true,
            Data::Character(_) => return true,
            Data::Integer(_) => return true,
            Data::Boolean(_) => return true,
            _other => return false,
        };
    }
}

impl Compare for Data {

    fn compare(&self, other: &Self) -> Relation {
        match self {

            Data::Map(_map) => panic!("maps may not be used as keys"),

            Data::List(_items) => panic!("lists may not be used as keys"),

            Data::Float(_float) => panic!("floats may not be used as keys"),

            Data::Path(steps) => {
                match other {
                    Data::Map(_map) => panic!("maps may not be used as keys"),
                    Data::List(_items) => panic!("lists may not be used as keys"),
                    Data::Float(_float) => panic!("floats may not be used as keys"),
                    Data::Path(other_steps) => {
                        let mut index = 0;
                        loop {
                            if steps.len() <= index {
                                match other_steps.len() <= index {
                                    true => return Relation::Equal,
                                    false => return Relation::Smaller,
                                }
                            }

                            if other_steps.len() <= index {
                                return Relation::Bigger;
                            }

                            let result = steps[index].compare(&other_steps[index]);
                            if result == Relation::Equal {
                                index += 1;
                                continue;
                            }
                            return result;
                        }
                    }
                    _other => return Relation::Bigger,
                }
            },

            Data::Identifier(identifier) => {
                match other {
                    Data::Map(_value) => panic!("maps may not be used as keys"),
                    Data::List(_value) => panic!("lists may not be used as keys"),
                    Data::Float(_value) => panic!("floats may not be used as keys"),
                    Data::Path(_value) => return Relation::Smaller,
                    Data::Identifier(other_identifier) => return identifier.compare(other_identifier),
                    Data::Keyword(_value) => return Relation::Smaller,
                    Data::String(_value) => return Relation::Smaller,
                    _other => return Relation::Bigger,
                }
            },

            Data::Keyword(keyword) => {
                match other {
                    Data::Map(_value) => panic!("maps may not be used as keys"),
                    Data::List(_value) => panic!("lists may not be used as keys"),
                    Data::Float(_value) => panic!("floats may not be used as keys"),
                    Data::Path(_value) => return Relation::Smaller,
                    Data::Keyword(other_keyword) => return keyword.compare(other_keyword),
                    Data::String(_value) => return Relation::Smaller,
                    _other => return Relation::Bigger,
                }
            },

            Data::String(string) => {
                match other {
                    Data::Map(_value) => panic!("maps may not be used as keys"),
                    Data::List(_value) => panic!("lists may not be used as keys"),
                    Data::Float(_value) => panic!("floats may not be used as keys"),
                    Data::Path(_value) => return Relation::Smaller,
                    Data::String(other_string) => return string.compare(other_string),
                    _other => return Relation::Bigger,
                }
            },

            Data::Character(character) => {
                match other {
                    Data::Map(_value) => panic!("maps may not be used as keys"),
                    Data::List(_value) => panic!("lists may not be used as keys"),
                    Data::Float(_value) => panic!("floats may not be used as keys"),
                    Data::Character(other_character) => return character.compare(other_character),
                    Data::Boolean(_value) => return Relation::Bigger,
                    Data::Integer(_value) => return Relation::Bigger,
                    _other => return Relation::Smaller,
                }
            },

            Data::Boolean(boolean) => {
                match other {
                    Data::Map(_value) => panic!("maps may not be used as keys"),
                    Data::List(_value) => panic!("lists may not be used as keys"),
                    Data::Float(_value) => panic!("floats may not be used as keys"),
                    Data::Boolean(other_boolean) => {
                        match boolean == other_boolean {
                            true => return Relation::Equal,
                            false => return Relation::from_boolean(*other_boolean),
                        }
                    },
                    _other => return Relation::Smaller,
                }
            },

            Data::Integer(integer) => {
                match other {
                    Data::Map(_value) => panic!("maps may not be used as keys"),
                    Data::List(_value) => panic!("lists may not be used as keys"),
                    Data::Float(_value) => panic!("floats may not be used as keys"),
                    Data::Integer(other_integer) => {
                        match integer == other_integer {
                            true => return Relation::Equal,
                            false => return Relation::from_boolean(integer < other_integer),
                        }
                    },
                    Data::Character(_value) => return Relation::Bigger,
                    _other => return Relation::Smaller,
                }
            },
        }
    }
}
