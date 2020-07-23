use super::Error;

pub enum Status<T> {
    Success(T),
    Error(Error),
}
