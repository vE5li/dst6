use internal::*;

pub enum IndexResult {
    Reference(*const Data),
    Literal(*const Data, usize),
    Missed,
}
