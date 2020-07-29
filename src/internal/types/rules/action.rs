use internal::*;
use debug::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Map(SharedString),
    Invalid,
    Ignored,
}

impl Action {

    pub fn serialize(self) -> Data {
        match self {
            Action::Map(mapped) => return string!(String, mapped),
            Action::Invalid => return keyword!("invalid"),
            Action::Ignored => return keyword!("ignored"),
        }
    }

    pub fn deserialize(serialized: &Data) -> Status<Self> {

        if serialized.is_string() {
            let mapped = unpack_string!(serialized);
            return success!(Action::Map(mapped));
        }

        if *serialized == keyword!("invalid") {
            return success!(Action::Invalid);
        }

        if *serialized == keyword!("ignored") {
            return success!(Action::Ignored);
        }

        return error!(Message, string!("invalid action type"));
    }

    pub fn is_mapped_to(&self, string: &str) -> bool {
        match self {
            Action::Map(mapped) => return mapped.printable() == string,
            _ => return false,
        }
    }
}
