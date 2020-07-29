mod decision;
mod path;

use internal::*;

pub use self::path::Path;
pub use self::decision::Decision;

#[derive(Clone, Debug)]
pub enum MatchResult {
    //MatchedUntil(Vec<Path>, Location), // for matched lists
    Matched(SharedVector<Path>),
    //BestMatch(Location),
    Missed,
}

impl MatchResult {

    pub fn from(paths: SharedVector<Path>) -> Self {
        match paths.is_empty() {
            true => return MatchResult::Missed,
            false => return MatchResult::Matched(paths),
        }
    }

    //pub fn matched(&self) -> bool {
    //    match self {
    //        MatchResult::Matched(..) => return true,
    //        MatchResult::Missed => return false,
    //    }
    //}

    pub fn update(self, paths: &mut SharedVector<Path>) {
        if let MatchResult::Matched(new_paths) = self {
            paths.append(&new_paths);
        }
    }

    //pub fn combine(&mut self, result: MatchResult) {
    //    if let MatchResult::Matched(paths) = self {
    //        result.update(paths);
    //    } else {
    //        *self = result;
    //    }
    //}
}
