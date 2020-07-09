use internal::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Map(VectorString),
    Invalid,
    Ignored,
}

impl Action {

    pub fn is_mapped_to(&self, string: &str) -> bool {
        match self {
            Action::Map(map) => return map.printable() == string,
            _ => return false,
        }
    }
}
