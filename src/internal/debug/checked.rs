use internal::*;

pub struct Checked<T> {
    inner:  Option<T>,
    name:   &'static str,
    set:    bool,
}

impl<T> Checked<T> {

    pub fn some(name: &'static str, inner: T) -> Self {
        Self {
            inner:  Some(inner),
            name:   name,
            set:    false,
        }
    }

    pub fn none(name: &'static str) -> Self {
        Self {
            inner:  None,
            name:   name,
            set:    false,
        }
    }

    pub fn into_inner(self) -> Option<T> {
        self.inner
    }

    pub fn changed(self) -> Option<T> {
        match self.set {
            true => self.inner,
            false => None,
        }
    }

    pub fn update(&mut self, parameters: &mut Vec<T>) -> Status<()> {
        ensure!(!self.set, Message, string!(str, "{} may not be set twice", self.name));
        ensure!(!parameters.is_empty(), Message, string!(str, "parameters expected {}", self.name));
        self.inner = Some(parameters.remove(0));
        self.set = true;
        return success!(());
    }
}
