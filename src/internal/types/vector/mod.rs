mod iterator;

use std::ops::{ Index, IndexMut };
use std::fmt::{ Formatter, Result, Display, Debug };
use std::iter::{ FromIterator, Iterator };
use std::cmp::{ PartialEq, Eq };
use std::sync::Mutex;
use std::rc::Rc;

pub use self::iterator::*;

macro_rules! get_vector {
    ($shared: expr) => { $shared.vector.lock().unwrap() }
}

macro_rules! get_vector_mut {
    ($shared: expr) => { Rc::get_mut(&mut $shared.vector).unwrap().get_mut().unwrap() }
}

#[derive(Clone)]
pub struct SharedVector<T> {
    vector: Rc<Mutex<Vec<T>>>,
}

#[allow(dead_code)]
impl<T: Clone> SharedVector<T> {

    pub fn new() -> Self {
        let vector = Rc::new(Mutex::new(Vec::new()));
        SharedVector {
            vector: vector,
        }
    }

    fn single_reference(&mut self) {
        if Rc::strong_count(&self.vector) > 1 {
            let cloned_vector = self.vector.lock().unwrap().clone();
            let new_vector = Rc::new(Mutex::new(cloned_vector));
            self.vector = new_vector;
        }
    }

    pub fn push(&mut self, value: T) {
        self.single_reference();
        let vector = get_vector_mut!(self);
        vector.push(value);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.single_reference();
        let vector = get_vector_mut!(self);
        return vector.pop();
    }

    pub fn remove(&mut self, index: usize) -> T {
        self.single_reference();
        let vector = get_vector_mut!(self);
        let item = vector[index].clone();
        vector.remove(index);
        return item;
    }

    pub fn insert(&mut self, index: usize, item: T) {
        self.single_reference();
        let vector = get_vector_mut!(self);
        vector.insert(index, item);
    }

    pub fn append(&mut self, source: &SharedVector<T>) {
        for item in source.iter() {
            self.push(item.clone());
        }
    }

    pub fn clear(&mut self) {
        self.single_reference();
        let vector = get_vector_mut!(self);
        vector.clear();
    }

    pub fn len(&self) -> usize {
        let vector = get_vector!(self);
        return vector.len();
    }

    pub fn is_empty(&self) -> bool {
        return self.len() == 0;
    }

    pub fn slice(&self, start: usize, end: usize) -> Self {
        let mut sliced = Self::new();
        if start > end {
            panic!("vector range too small");
        }
        if end >= self.len() {
            panic!("vector slice out of range");
        }
        for index in start..=end {
            sliced.push(self[index].clone());
        }
        return sliced;
    }

    pub fn slice_end(&self, start: usize) -> Self {
        return self.slice(start, self.len() - 1);
    }

    pub fn flip(&self) -> Self {
        return self.reverse_iter().cloned().collect();
    }

    pub fn retain<F>(&mut self, f: F) where F: FnMut(&T) -> bool {
        self.single_reference();
        let vector = get_vector_mut!(self);
        vector.retain(f);
    }

    pub fn iter(&self) -> VectorIterator<T> {
        return VectorIterator::new(self);
    }

    pub fn into_iter(&self) -> VectorIntoIterator<T> {
        return VectorIntoIterator::new(self);
    }

    pub fn iter_mut(&mut self) -> MutableVectorIterator<T> {
        return MutableVectorIterator::new(self);
    }

    pub fn reverse_iter(&self) -> ReverseVectorIterator<T> {
        return ReverseVectorIterator::new(self);
    }

    pub fn reverse_iter_mut(&mut self) -> ReverseMutableVectorIterator<T> {
        return ReverseMutableVectorIterator::new(self);
    }

    pub fn transfer(&mut self) -> Self {
        let cloned = self.clone();
        self.clear();
        return cloned;
    }
}

impl<T: Clone + PartialEq> SharedVector<T> {

    pub fn contains(&self, compare: &T) -> bool {
        let vector = get_vector!(self);
        return vector.contains(compare);
    }

    pub fn split(&self, compare: &T, void: bool) -> Vec<Self> {
        let mut pieces = Vec::new();
        let mut buffer = Self::new();

        for item in self.iter() {
            if *item == *compare {
                if !void || !buffer.is_empty() {
                    pieces.push(buffer.transfer());
                }
                continue;
            }
            buffer.push(item.clone());
        }

        if !buffer.is_empty() {
            pieces.push(buffer);
        }

        return pieces;
    }

    pub fn replace(&self, from: &T, to: &T) -> Self {
        let mut replaced = self.clone();
        for item in replaced.iter_mut() {
            if *item == *from {
                *item = to.clone();
            }
        }
        return replaced;
    }

    pub fn position(&self, compare: &T) -> Vec<usize> {
        let mut positions = Vec::new();
        for (index, item) in self.iter().enumerate() {
            if *item == *compare {
                positions.push(index);
            }
        }
        return positions;
    }
}

impl<T: Clone> FromIterator<T> for SharedVector<T> {

    fn from_iter<I: IntoIterator<Item = T>>(iterator: I) -> SharedVector<T> {
        let mut vector = SharedVector::new();
        for item in iterator {
            vector.push(item);
        }
        return vector;
    }
}

impl<T: Clone + PartialEq> PartialEq for SharedVector<T> {

    fn eq(&self, other: &Self) -> bool {
        match Rc::as_ptr(&self.vector) == Rc::as_ptr(&other.vector) {
            true => return true,
            false => return *self.vector.lock().unwrap() == *other.vector.lock().unwrap(),
        }
    }
}

impl<T: Clone + Eq> Eq for SharedVector<T> { }

impl<T: Clone> Index<usize> for SharedVector<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        let vector = get_vector!(self);
        let item_pointer = &vector[index];
        return unsafe { &*(item_pointer as *const T) };
    }
}

impl<T: Clone> IndexMut<usize> for SharedVector<T> {

    fn index_mut(&mut self, index: usize) -> &mut T {
        self.single_reference();
        let vector = get_vector_mut!(self);
        let item_pointer = &mut vector[index];
        return unsafe { &mut *(item_pointer as *mut T) };
    }
}

impl<T: Clone + Debug> Debug for SharedVector<T> {

    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let string = self.iter().map(|item| format!("{:?}", item)).collect::<Vec<String>>().join(", ");
        return write!(f, "[{}]", string);
    }
}

impl<T: Clone + Display> Display for SharedVector<T> {

    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let string = self.iter().map(|item| format!("{}", item)).collect::<Vec<String>>().join(", ");
        return write!(f, "[{}]", string);
    }
}
