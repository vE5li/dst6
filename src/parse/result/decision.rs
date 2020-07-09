use internal::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    Template(Data),
    Filter(usize),
    Flavor(usize),
    List,
    Next,
    End,
}

impl Decision {

    pub fn compare(&self, other: &Decision) -> Option<bool> {
        match self {

            Decision::Filter(self_index) => {
                if let Decision::Filter(other_index) = other {
                    match self_index == other_index {
                        true => return None,
                        false => return Some(self_index > other_index),
                    }
                }
                panic!();
            }

            Decision::Flavor(self_index) => {
                if let Decision::Flavor(other_index) = other {
                    match self_index == other_index {
                        true => return None,
                        false => return Some(self_index > other_index),
                    }
                }
                panic!();
            }

            Decision::Next => {
                match other {
                    Decision::Next => return None,
                    Decision::End => return Some(true),
                    _other => panic!(),
                }
            }

            Decision::End => {
                match other {
                    Decision::End => return None,
                    Decision::Next => return Some(false),
                    _other => panic!(),
                }
            }

            _other => {
                if self != other {
                    panic!();
                }
                return None;
            }
        }
    }
}
