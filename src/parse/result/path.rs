use internal::*;
use super::Decision;

#[derive(Clone, Debug)]
pub struct Path {
    pub decisions:  Vector<Decision>,
    pub index:      usize,
    pub width:      usize,
    pub confirmed:  bool,
    pub expected:   Option<Data>,
}

impl Path {

    pub fn new(decisions: Vector<Decision>, index: usize, width: usize, confirmed: bool, expected: Option<Data>) -> Self {
        Self {
            decisions:  decisions,
            index:      index,
            width:      width,
            confirmed:  confirmed,
            expected:   expected,
        }
    }

    pub fn evaluate(&self, other: &Path) -> Option<bool> {
        if self.width != other.width {
            return None;
        }

        for index in 0..self.decisions.len() {
            if let Some(result) = self.decisions[index].compare(&other.decisions[index]) {
                return Some(result);
            }
        }

        panic!();
    }
}
