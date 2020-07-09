use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relation {
    Smaller,
    Bigger,
    Equal,
}

impl Relation {

    pub fn from_boolean(value: bool) -> Self {
        match value {
            true => return Relation::Smaller,
            false => return Relation::Bigger,
        }
    }

    pub fn into_ordering(&self) -> Ordering {
        match self {
            Relation::Smaller => return Ordering::Less,
            Relation::Bigger => return Ordering::Greater,
            Relation::Equal => return Ordering::Equal,
        }
    }
}

pub trait Compare {

    fn compare(&self, other: &Self) -> Relation;
}
