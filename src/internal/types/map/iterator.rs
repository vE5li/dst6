use super::node::{ Node, Branch };
use internal::Compare;
use std::ptr::NonNull;
use std::marker::PhantomData;
use std::clone::Clone;

fn add_to_list<K: Compare + Clone, T: Clone>(branch: &Branch<K, T>, list: &mut Vec<NonNull<Node<K, T>>>) {
    if let Some(node_pointer) = branch.clone() {
        let node = unsafe { node_pointer.as_ref() };
        add_to_list(&node.left, list);
        list.push(node_pointer);
        add_to_list(&node.right, list);
    }
}

fn add_to_list_mut<K: Compare + Clone, T: Clone>(branch: &mut Branch<K, T>, list: &mut Vec<NonNull<Node<K, T>>>) {
    *branch = Node::single_reference(branch.clone());
    if let Some(mut node_pointer) = branch {
        let copied_node_pointer = node_pointer;
        let node = unsafe { node_pointer.as_mut() };
        add_to_list_mut(&mut node.left, list);
        list.push(copied_node_pointer);
        add_to_list_mut(&mut node.right, list);
    }
}

macro_rules! create_iterator {
    ($name:ident, $adder:ident, $adder_type:ty) => (
        #[derive(Debug)]
        pub struct $name<'a, K: Compare, T> {
            values:     Vec<NonNull<Node<K, T>>>,
            phantom:    PhantomData<&'a K>,
            index:      usize,
        }

        impl<'a, K: Compare + Clone, T: Clone> $name<'a, K, T> {
            pub fn new(root: $adder_type, size: usize) -> Self {
                let mut values = Vec::with_capacity(size);
                $adder(root, &mut values);

                Self {
                    values:     values,
                    phantom:    PhantomData,
                    index:      0,
                }
            }
        }
    );
}

create_iterator!(MapIterator, add_to_list, &Branch<K, T>);
create_iterator!(MapDrainIterator, add_to_list, &Branch<K, T>);
create_iterator!(MapKeyIterator, add_to_list, &Branch<K, T>);
create_iterator!(MapValueIterator, add_to_list, &Branch<K, T>);
create_iterator!(MutableMapIterator, add_to_list_mut, &mut Branch<K, T>);
create_iterator!(MutableMapValueIterator, add_to_list_mut, &mut Branch<K, T>);

impl<'a, K: Compare, T: 'a> Iterator for MapIterator<'a, K, T> {
    type Item = (&'a K, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.values.len() {
            return None;
        }
        let value = unsafe { (&(*self.values[self.index].as_ptr()).key, &(*self.values[self.index].as_ptr()).value) };
        self.index += 1;
        return Some(value);
    }
}

impl<'a, K: Compare + Clone, T: Clone> Iterator for MapDrainIterator<'a, K, T> {
    type Item = (K, T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.values.len() {
            return None;
        }
        let value = unsafe { (self.values[self.index].as_ref().key.clone(), self.values[self.index].as_ref().value.clone()) };
        self.index += 1;
        return Some(value);
    }
}

impl<'a, K: Compare, T: 'a> Iterator for MapKeyIterator<'a, K, T> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.values.len() {
            return None;
        }
        let value = unsafe { &(*self.values[self.index].as_ptr()).key };
        self.index += 1;
        return Some(value);
    }
}

impl<'a, K: Compare, T: 'a> Iterator for MapValueIterator<'a, K, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.values.len() {
            return None;
        }
        let value = unsafe { &(*self.values[self.index].as_ptr()).value };
        self.index += 1;
        return Some(value);
    }
}

impl<'a, K: Compare, T: 'a> Iterator for MutableMapIterator<'a, K, T> {
    type Item = (&'a K, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.values.len() {
            return None;
        }
        let value = unsafe { (&(*self.values[self.index].as_ptr()).key, &mut (*self.values[self.index].as_ptr()).value) };
        self.index += 1;
        return Some(value);
    }
}

impl<'a, K: Compare, T: 'a> Iterator for MutableMapValueIterator<'a, K, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.values.len() {
            return None;
        }
        let value = unsafe { &mut (*self.values[self.index].as_ptr()).value };
        self.index += 1;
        return Some(value);
    }
}
