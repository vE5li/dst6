use internal::*;

#[derive(Clone, Debug)]
pub struct Pass {
    pub name: Data,
    pub parameters: Vec<Data>,
}

impl Pass {

    pub fn new(name: Data, parameters: Vec<Data>) -> Self {
        Self {
            name: name,
            parameters: parameters,
        }
    }
}
