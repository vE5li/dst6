mod node;
mod iterator;

use internal::*;
use self::node::{ Node, Branch };
use std::cmp::PartialEq;
use std::fmt::{ Formatter, Result, Display, Debug };

pub use self::iterator::*;

pub type DataMap = Map<Data, Data>;

pub struct Map<K: Compare, T> {
    root:       Branch<K, T>,
    size:       usize,
}

#[allow(dead_code)]
impl<K: Compare + Clone, T: Clone> Map<K, T> {

    pub fn new() -> Self {
        Self {
            root:       None,
            size:       0,
        }
    }

    pub fn insert(&mut self, key: K, value: T) -> Option<T> {
        let previous_data = Node::insert(&mut self.root, key, value);
        if previous_data.is_none() {
            self.size += 1;
        }
        return previous_data;
    }

    pub fn remove(&mut self, key: &K) -> Option<T> {
        if self.size == 0 {
            return None;
        }
        self.size -= 1;
        return Node::remove(&mut self.root, key);
    }

    pub fn get(&self, key: &K) -> Option<&T> {
        return Node::get(&self.root, key);
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut T> {
        return Node::get_mut(&mut self.root, key);
    }

    pub fn contains_key(&self, key: &K) -> bool {
        return self.get(key).is_some();
    }

    pub fn len(&self) -> usize {
        return self.size;
    }

    pub fn iter(&self) -> MapIterator<K, T> {
        return MapIterator::new(&self.root, self.size);
    }

    pub fn iter_mut(&mut self) -> MutableMapIterator<K, T> {
        return MutableMapIterator::new(&mut self.root, self.size);
    }

    pub fn keys(&self) -> MapKeyIterator<K, T> {
        return MapKeyIterator::new(&self.root, self.size);
    }

    pub fn values(&self) -> MapValueIterator<K, T> {
        return MapValueIterator::new(&self.root, self.size);
    }

    pub fn values_mut(&mut self) -> MutableMapValueIterator<K, T> {
        return MutableMapValueIterator::new(&mut self.root, self.size);
    }

    pub fn drain(&mut self) -> MapDrainIterator<K, T> {
        return MapDrainIterator::new(&self.root, self.size);
    }
}

unsafe impl<K: Compare, T: Send> Send for Map<K, T> {}

unsafe impl<K: Compare, T: Sync> Sync for Map<K, T> {}

#[allow(dead_code)]
impl<K: Compare + Clone, T: PartialEq + Clone> Map<K, T> {

    pub fn contains(&self, compare: &T) -> bool {
        return self.values().find(|item| **item == *compare).is_some()
    }

    pub fn replace(&self, from: &T, to: &T) -> Self {
        let mut new_map = self.clone();
        for item in new_map.values_mut() {
            if *item == *from {
                *item = to.clone();
            }
        }
        return new_map;
    }

    pub fn position(&self, compare: &T) -> Vector<K> {
        let mut positions = Vector::new();
        for (key, value) in self.iter() {
            if *value == *compare {
                positions.push(key.clone());
            }
        }
        return positions;
    }

    fn compare_branch(&self, other_root: &Branch<K, T>) -> bool {
        return Node::compare_branch(&self.root, other_root);
    }
}

impl<K: Compare, T> Clone for Map<K, T> {

    fn clone(&self) -> Self {
        if let Some(root) = self.root {
            unsafe { (*root.as_ptr()).counter += 1 };
        }
        Self {
            root:       self.root.clone(),
            size:       self.size,
        }
    }
}

impl<K: Compare, T> Drop for Map<K, T> {

    fn drop(&mut self) {
        Node::drop(&mut self.root);
    }
}

impl<K: Compare + Clone, T: PartialEq + Clone> PartialEq for Map<K, T> {

    fn eq(&self, other: &Self) -> bool {
        //match self.len() == other.len() {
        //    true => return other.compare_branch(&self.root),
        //    false => return false,
        //}

        match self.len() == other.len() { // TODO:
            true => {
                let mut self_iter = self.iter();
                let mut other_iter = other.iter();

                for _index in 0..self.len() {
                    let (self_key, self_value) = self_iter.next().unwrap();
                    let (other_key, other_value) = other_iter.next().unwrap();

                    if self_key.compare(other_key) != Relation::Equal {
                        return false;
                    }

                    if *self_value != *other_value {
                        return false;
                    }
                }
                return true;
            },
            false => return false,
        }
    }
}

impl<K: Compare + Clone, T: PartialEq + Clone> Eq for Map<K, T> { }

impl<K: Compare, T> Debug for Map<K, T> {

    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        return write!(f, "<implement>"); // TODO:
    }
}

impl<K: Compare, T> Display for Map<K, T> {

    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        return write!(f, "<implement>"); // TODO:
    }
}
