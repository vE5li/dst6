use internal::*;

#[derive(Debug, Clone)]
pub enum Flow {
    While(Vector<Data>, Option<Data>, usize),
    For(i64, i64, i64, usize),
    IndexIteration(Vec<Data>, usize),
    Condition(bool),
}
