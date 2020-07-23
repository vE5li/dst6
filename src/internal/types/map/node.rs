use internal::*;

use super::super::allocator::*;
use std::clone::Clone;
use std::cmp::max;

pub type Branch<K, T> = Option<NonNull<Node<K, T>>>;

pub struct Node<K: Compare, T> {
    pub left:       Branch<K, T>,
    pub right:      Branch<K, T>,
    pub counter:    usize,
    pub height:     usize,
    pub key:        K,
    pub value:      T,
}

impl<K: Compare + Clone, T: Clone> Node<K, T> {

    pub fn new(key: K, value: T) -> Self {
        Self {
            left:       None,
            right:      None,
            counter:    1,
            height:     1,
            key:        key,
            value:      value,
        }
    }

    fn increment_clone(branch: &Branch<K, T>) -> Branch<K, T> {
        if let Some(mut pointer) = branch.clone() {
            let node = unsafe { pointer.as_mut() };
            node.counter += 1;
            return Some(pointer);
        }
        return None;
    }

    fn swap(&mut self, value: T) -> T { // optimize
        let previous = self.value.clone();
        self.value = value;
        return previous;
    }

    fn get_height(branch: &Branch<K, T>) -> usize {
        match branch {
            Some(node_pointer) => return unsafe { node_pointer.as_ref().height },
            None => return 0,
        }
    }

    fn rotate_left(branch: &mut Branch<K, T>) {
        if let Some(mut node_pointer) = branch.clone() {
            let mut node = unsafe { node_pointer.as_mut() };
            let right_branch = Self::single_reference(node.right.clone());

            if let Some(mut right_node_pointer) = right_branch.clone() {
                let right_node = unsafe { right_node_pointer.as_mut() };
                let new_right_branch = right_node.left.clone();
                right_node.left = branch.clone();
                node.right = new_right_branch;
                *branch = right_branch.clone();
            }
        }
    }

    fn rotate_right(branch: &mut Branch<K, T>) {
        if let Some(mut node_pointer) = branch.clone() {
            let mut node = unsafe { node_pointer.as_mut() };
            let left_branch = Self::single_reference(node.left.clone());

            if let Some(mut left_node_pointer) = left_branch.clone() {
                let left_node = unsafe { left_node_pointer.as_mut() };
                let new_left_branch = left_node.right.clone();
                left_node.right = branch.clone();
                node.left = new_left_branch;
                *branch = left_branch.clone();
            }
        }
    }

    fn get_balance(branch: &Branch<K, T>) -> isize {
        if let Some(node_pointer) = branch {
            let node = unsafe { node_pointer.as_ref() };
            return (Self::get_height(&node.left) as isize) - (Self::get_height(&node.right) as isize);
        }
        return 0;
    }

    fn smallest_subnote(branch: &mut Branch<K, T>) -> Branch<K, T> {
        *branch = Self::single_reference(branch.clone());
        if let Some(mut node_pointer) = branch {
            let node = unsafe { node_pointer.as_mut() };
            return Self::smallest_subnote(&mut node.left);
        }
        return branch.clone();
    }

    pub fn insert(branch: &mut Branch<K, T>, key: K, value: T) -> Option<T> {
        *branch = Self::single_reference(branch.clone());
        if let Some(mut node_pointer) = branch {
            let node = unsafe { node_pointer.as_mut() };
            let previous = match node.key.compare(&key) {
                Relation::Bigger => Self::insert(&mut node.left, key.clone(), value),
                Relation::Smaller => Self::insert(&mut node.right, key.clone(), value),
                Relation::Equal => Some(node.swap(value)),
            };

            match previous {
                Some(previous) => return Some(previous),
                None => node.height = 1 + max(Self::get_height(&node.left), Self::get_height(&node.right)),
            }
        } else {
            let pointer = allocate!(Node<K, T>);
            let new_node = Node::new(key, value);
            unsafe { write(pointer.as_ptr(), new_node) };
            *branch = Some(pointer);
            return None;
        }

        let balance = Self::get_balance(branch);
        let mut node_pointer = branch.clone().unwrap();
        let node = unsafe { node_pointer.as_mut() };

        if balance > 1 {
            node.left = Self::single_reference(node.left.clone());
            let left_key = unsafe { node.left.clone().unwrap().as_ref().key.clone() };
            if left_key.compare(&key) == Relation::Smaller {
                Self::rotate_left(&mut node.left);
                Self::rotate_right(branch);
            } else {
                Self::rotate_right(branch);
            }
        }

        if balance < -1 {
            node.right = Self::single_reference(node.right.clone());
            let right_key = unsafe { node.right.clone().unwrap().as_ref().key.clone() };
            if right_key.compare(&key) == Relation::Bigger {
                Self::rotate_right(&mut node.right);
                Self::rotate_left(branch);
            } else {
                Self::rotate_left(branch);
            }
        }

        return None;
    }

    pub fn remove(branch: &mut Branch<K, T>, key: &K) -> Option<T> {
        *branch = Self::single_reference(branch.clone());
        let previous = if let Some(mut node_pointer) = branch.clone() {
            let copied_node_pointer = node_pointer;
            let node = unsafe { node_pointer.as_mut() };
            match node.key.compare(&key) {
                Relation::Bigger => Self::remove(&mut node.left, key),
                Relation::Smaller => Self::remove(&mut node.right, key),
                Relation::Equal => {
                    let value = node.value.clone();

                    if let Some(left_node_pointer) = node.left.clone() {
                        if node.right.is_some() {
                            let successor_branch = Self::smallest_subnote(&mut node.right);
                            if let Some(successor_pointer) = successor_branch {
                                let successor_node = unsafe { successor_pointer.as_ref() };
                                node.key = successor_node.key.clone();
                                Self::remove(&mut node.right, &successor_node.key); // make this different maybe
                            }
                        } else {
                            *branch = Some(left_node_pointer);
                            unsafe { deallocate!(copied_node_pointer, Node<K, T>) };
                        }
                    } else {
                        if let Some(right_node_pointer) = node.right.clone() {
                            *branch = Some(right_node_pointer);
                            unsafe { deallocate!(copied_node_pointer, Node<K, T>) };
                        } else {
                            *branch = None;
                            unsafe { deallocate!(copied_node_pointer, Node<K, T>) };
                            return Some(value);
                        }
                    }

                    Some(value)
                },
            }
        } else {
            None
        };

        if previous.is_some() {
            if let Some(mut node_pointer) = branch.clone() {
                let node = unsafe { node_pointer.as_mut() };
                node.height = 1 + max(Self::get_height(&node.left), Self::get_height(&node.right));
                let balance = Self::get_balance(branch);

                if balance > 1 {
                    if Self::get_balance(&node.left) < 0 {
                        Self::rotate_left(&mut node.left);
                        Self::rotate_right(branch);
                    } else {
                        Self::rotate_right(branch);
                    }
                }

                if balance < -1 {
                    if Self::get_balance(&node.right) > 0 {
                        Self::rotate_right(&mut node.right);
                        Self::rotate_left(branch);
                    } else {
                        Self::rotate_left(branch);
                    }
                }
            }
        }

        return previous;
    }

    pub fn get<'a>(branch: &Branch<K, T>, key: &K) -> Option<&'a T> { // make this block the map
        if let Some(node_pointer) = branch {
            let node = unsafe { node_pointer.as_ref() };
            match node.key.compare(&key) {
                Relation::Bigger => return Self::get(&node.left, key),
                Relation::Smaller => return Self::get(&node.right, key),
                Relation::Equal => return unsafe { Some(&*(&node.value as *const T)) },
            };
        }
        return None;
    }

    pub fn get_mut<'a>(branch: &mut Branch<K, T>, key: &K) -> Option<&'a mut T> { // make this block the map
        *branch = Self::single_reference(branch.clone());
        if let Some(mut node_pointer) = branch {
            let node = unsafe { node_pointer.as_mut() };
            match node.key.compare(&key) {
                Relation::Bigger => return Self::get_mut(&mut node.left, key),
                Relation::Smaller => return Self::get_mut(&mut node.right, key),
                Relation::Equal => return unsafe { Some(&mut *(&mut node.value as *mut T)) },
            };
        }
        return None;
    }

    pub fn single_reference(branch: Branch<K, T>) -> Branch<K, T> {
        if let Some(mut node_pointer) = branch {
            let node = unsafe { node_pointer.as_mut() };
            if node.counter == 1 {
                return Some(node_pointer);
            }

            node.counter -= 1;
            let node_pointer = allocate!(Node<K, T>);
            unsafe { write(node_pointer.as_ptr(), node.clone()) };
            return Some(node_pointer);
        }
        return None;
    }
}

impl<K: Compare + Clone, T: PartialEq + Clone> Node<K, T> {

    pub fn compare_branch(branch: &Branch<K, T>, other_branch: &Branch<K, T>) -> bool {
        if let Some(node_pointer) = branch {
            if let Some(other_node_pointer) = other_branch {
                if node_pointer == other_node_pointer {
                    return true;
                }

                unsafe {
                    let node = node_pointer.as_ref();
                    let other_node = other_node_pointer.as_ref();
                    return node == other_node;  // only do this if the value and key of the nodes match -> otherwise two maps with the same keys and values might be mismatched
                }
            }
            return false;
        }
        return other_branch.is_none();
    }
}

impl<K: Compare, T> Node<K, T> {

    pub fn drop(branch: &mut Branch<K, T>) {
        if let Some(mut node_pointer) = branch {
            let node = unsafe { node_pointer.as_mut() };
            node.counter -= 1;
            if node.counter == 0 {
                Node::drop(&mut node.left);
                Node::drop(&mut node.right);
                unsafe { deallocate!(node_pointer, Node<K, T>) };
            }
        }
    }
}

impl<K: Compare + Clone, T: PartialEq + Clone> PartialEq for Node<K, T> {

    fn eq(&self, other: &Self) -> bool {
        if self.height != other.height || self.value != other.value {
            return false;
        }

        if self.key.compare(&other.key) != Relation::Equal {
            return false;
        }

        return Self::compare_branch(&self.left, &other.left) && Self::compare_branch(&self.right, &other.right);
    }
}

impl<K: Compare + Clone, T: Clone> Clone for Node<K, T> {

    fn clone(&self) -> Self {
        Self {
            left:       Self::increment_clone(&self.left),
            right:      Self::increment_clone(&self.right),
            counter:    1,
            height:     self.height,
            key:        self.key.clone(),
            value:      self.value.clone(),
        }
    }
}
