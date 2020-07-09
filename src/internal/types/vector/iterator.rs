use std::clone::Clone;
use super::Vector;

macro_rules! create_iterator {
    ($name:ident, $reverse:expr, $type:ty) => (
        pub struct $name<'a, T: Clone> {
            vector:     $type,
            index:      usize,
        }

        impl<'a, T: Clone> $name<'a, T> {
            pub fn new(vector: $type) -> Self {
                let index = match $reverse {
                    true => vector.len(),
                    false => 0,
                };

                Self {
                    vector:     vector,
                    index:      index,
                }
            }
        }
    );
}

create_iterator!(VectorIterator, false, &'a Vector<T>);
create_iterator!(VectorIntoIterator, false, &'a Vector<T>);
create_iterator!(MutableVectorIterator, false, &'a mut Vector<T>);
create_iterator!(ReverseVectorIterator, true, &'a Vector<T>);
create_iterator!(ReverseMutableVectorIterator, true, &'a mut Vector<T>);

impl<'a, T: Clone> Iterator for VectorIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.vector.length {
            return None;
        }
        let item = self.vector.index(self.index);
        self.index += 1;
        return Some(item);
    }
}

impl<'a, T: Clone> Iterator for VectorIntoIterator<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.vector.length {
            return None;
        }
        let item = self.vector.index(self.index);
        self.index += 1;
        return Some(item.clone());
    }
}

impl<'a, T: Clone> Iterator for MutableVectorIterator<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.vector.length {
            return None;
        }
        //let item = self.vector.index_mut(self.index);
        let item = self.vector.index(self.index);
        let item = unsafe { &mut *(item as *const T as *mut T) };
        self.index += 1;
        return Some(item);
    }
}

impl<'a, T: Clone> Iterator for ReverseVectorIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == 0 {
            return None;
        }
        self.index -= 1;
        let item = self.vector.index(self.index);
        return Some(item);
    }
}

impl<'a, T: Clone> Iterator for ReverseMutableVectorIterator<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == 0 {
            return None;
        }
        //let item = self.vector.index_mut(self.index);
        self.index -= 1;
        let item = self.vector.index(self.index);
        let item = unsafe { &mut *(item as *const T as *mut T) };
        return Some(item);
    }
}
