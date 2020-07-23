#[macro_use]
mod block;
mod iterator;

use self::block::*;
use super::allocator::*;
use std::ops::{ Index, IndexMut, Drop };
use std::fmt::{ Formatter, Result, Display, Debug };
use std::iter::FromIterator;
use std::clone::Clone;
use std::cmp::{ PartialEq, Eq };

pub use self::iterator::*;

pub struct Vector<T> {
    lists:      Vec<NonNull<BlockList<T>>>,
    length:     usize,
    capacity:   usize,
}

#[allow(dead_code)]
impl<T: Clone> Vector<T> {

    pub fn new() -> Self {
        Vector {
            lists:      Vec::new(),
            length:     0,
            capacity:   0,
        }
    }

    fn append_block_list(&mut self) -> NonNull<BlockList<T>> {
        unsafe {
            let mut block_list = allocate!(BlockList<T>);
            block_list.as_mut().counter = 1;
            self.lists.push(block_list);
            return block_list;
        }
    }

    fn append_block(&mut self) -> NonNull<Block<T>> {
        let offset = self.capacity % BLOCK_LIST_SIZE;
        let block_list = match offset == 0 {
            true => self.append_block_list(),
            false => self.single_reference_list(self.capacity / BLOCK_LIST_SIZE),
        };

        unsafe {
            let mut block = allocate!(Block<T>);
            let address = block_list.as_ref().blocks.as_ptr() as *mut NonNull<Block<T>>;

            block.as_mut().counter = 1;
            write(address.add(offset), block);
            self.capacity += 1;
            return block;
        }
    }

    fn copy_list(&mut self, list_index: usize) -> NonNull<BlockList<T>> {
        let mut new_block_list = allocate!(BlockList<T>);

        unsafe {
            new_block_list.as_mut().counter = 1;

            let mut block_list = self.lists[list_index].as_mut();
            block_list.counter -= 1;

            for count in 0..self.capacity - list_index * BLOCK_LIST_SIZE {
                if count == BLOCK_LIST_SIZE {
                    break;
                }
                block_list.blocks[count].as_mut().counter += 1;
            }

            let mut data = block_list.clone();
            data.counter = 1;
            write(new_block_list.as_ptr(), data);
        }

        self.lists[list_index] = new_block_list;
        return new_block_list;
    }

    fn single_reference_list(&mut self, list_index: usize) -> NonNull<BlockList<T>> {
        unsafe {
            let list = &self.lists[list_index];
            match list.as_ref().counter == 1 {
                true => return *list,
                false => return self.copy_list(list_index),
            }
        }
    }

    fn single_reference_block(&mut self, block_index: usize) -> NonNull<Block<T>> {
        unsafe {
            let mut list = self.single_reference_list(block_index / BLOCK_LIST_SIZE);
            let mut block = list.as_ref().blocks[block_index % BLOCK_LIST_SIZE];

            if block.as_ref().counter == 1 {
                return block;
            }

            let mut new_block = allocate!(Block<T>);
            new_block.as_mut().counter = 1;
            block.as_mut().counter -= 1;

            if self.length > block_index * BLOCK_SIZE {
                for index in 0..clamped!(self.length - block_index * BLOCK_SIZE, BLOCK_SIZE) {
                    let pointer = new_block.as_ref().data.as_ptr() as *mut T;
                    write(pointer.add(index), block.as_ref().data[index].clone());
                }
            }

            list.as_mut().blocks[block_index % BLOCK_LIST_SIZE] = new_block;
            return new_block;
        }
    }

    fn shift_right_from(&mut self, index: usize) {
        for current in (index..self.length).rev() {
            let item = self.read_raw(current);
            self.write_raw(current + 1, item);
        }
    }

    fn shift_left_from(&mut self, index: usize) {
        for current in index..self.length - 1 {
            let item = self.read_raw(current + 1);
            self.write_raw(current, item);
        }
    }

    fn write_raw(&mut self, index: usize, value: T) {
        let block = match index == self.capacity * BLOCK_SIZE {
            true => self.append_block(),
            false => self.single_reference_block(index / BLOCK_SIZE),
        };

        unsafe {
            let address = block.as_ref().data.as_ptr() as *mut T;
            write(address.add(index % BLOCK_SIZE), value);
        }
    }

    fn read_raw(&mut self, index: usize) -> T {
        unsafe {
            let block_pointer = self.single_reference_block(index / BLOCK_SIZE);
            let block = block_pointer.as_ref();
            let address = block.data.as_ptr() as *const T;

            return read(address.add(index % BLOCK_SIZE));
        }
    }

    pub fn push(&mut self, value: T) {
        self.write_raw(self.length, value);
        self.length += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.length == 0 {
            return None;
        }

        self.length -= 1;
        return Some(self.read_raw(self.length));
    }

    pub fn index(&self, index: usize) -> &T {
        if index >= self.length {
            panic!("vector index out of range");
        }
        let offset = index % BLOCK_SIZE;
        let block_number = index / BLOCK_SIZE;
        let block_index = block_number % BLOCK_LIST_SIZE;
        let list_index = block_number / BLOCK_LIST_SIZE;

        return unsafe { self.lists[list_index].as_ref().blocks[block_index].as_ref().data.index(offset) };
    }

    pub fn index_mut(&mut self, index: usize) -> &mut T {
        if index >= self.length {
            panic!("vector index mut out of range");
        }
        let offset = index % BLOCK_SIZE;
        let block_number = index / BLOCK_SIZE;
        let block_index = block_number % BLOCK_LIST_SIZE;
        let list_index = block_number / BLOCK_LIST_SIZE;
        self.single_reference_block(block_number);
        return unsafe { self.lists[list_index].as_mut().blocks[block_index].as_mut().data.index_mut(offset) };
    }

    pub fn serialize(&self) -> String {
        let mut string = String::new();
        let mut index = 0;

        for list in &self.lists {
            string.push_str(&format!("[ location: {:?}", list));
            let list = unsafe { list.as_ref() };
            string.push_str(&format!(", counter: {}", list.counter));
            string.push_str(", blocks:");

            while index < self.capacity && (index == 0 || index % BLOCK_LIST_SIZE != 0) {
                let block = &list.blocks[index];
                string.push_str(&format!(" [ location: {:?}", block));
                let block = unsafe { block.as_ref() };
                string.push_str(&format!(", counter: {} ]", block.counter));
                index += 1;
            }

            string.push_str(" ] ");
        }

        return string;
    }

    pub fn remove(&mut self, index: usize) -> T {
        if index >= self.length {
            panic!("vector remove out of range");
        }
        let item = self.read_raw(index);
        self.shift_left_from(index);
        self.length -= 1;
        return item;
    }

    pub fn insert(&mut self, index: usize, item: T) {
        if index > self.length {
            panic!("vector remove out of range");
        }
        if index == self.length {
            self.push(item);
        } else {
            self.shift_right_from(index);
            self.write_raw(index, item);
            self.length += 1;
        }
    }

    pub fn append(&mut self, source: &Vector<T>) {
        for item in source.iter() {
            self.push(item.clone());
        }
    }

    pub fn clear(&mut self) {
        self.length = 0;
        // deallocate unneeded lists
    }

    pub fn is_empty(&self) -> bool {
        return self.length == 0;
    }

    pub fn len(&self) -> usize {
        return self.length;
    }

    pub fn slice(&self, start: usize, end: usize) -> Self {
        let mut sliced = Self::new();
        if start > end {
            panic!("vector range too small");
        }
        if end >= self.length {
            panic!("vector slice out of range");
        }
        for index in start..=end {
            sliced.push(self[index].clone());
        }
        return sliced;
    }

    pub fn slice_end(&self, start: usize) -> Self {
        return self.slice(start, self.length - 1);
    }

    pub fn flip(&self) -> Self {
        return self.reverse_iter().cloned().collect();
    }

    pub fn retain<F>(&mut self, mut f: F) where F: FnMut(&T) -> bool {
        let mut new_self = Self::new();
        for item in self.iter() {
            if f(item) {
                new_self.push(item.clone());
            }
        }
        *self = new_self;
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
        self.length = 0;
        // dereeference all lists (?)
        return cloned;
    }
}

impl<T: Clone + PartialEq> Vector<T> {

    pub fn contains(&self, compare: &T) -> bool {
        return self.iter().find(|item| **item == *compare).is_some();
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

    pub fn compare_lists(&self, lists: &Vec<NonNull<BlockList<T>>>) -> bool {
        for index in 0..lists.len() {
            if self.length < index * LIST_SIZE {
                break;
            }

            if self.lists[index] == lists[index] {
                continue;
            }

            unsafe {
                let capacity = clamped!(self.capacity - index * BLOCK_LIST_SIZE, BLOCK_LIST_SIZE);
                let length = clamped!(self.length - index * LIST_SIZE, LIST_SIZE);
                if !self.lists[index].as_ref().compare_blocks(lists[index].as_ref(), capacity, length) {
                    return false;
                }
            }
        }
        return true;
    }
}

impl<T> Clone for Vector<T> {

    fn clone(&self) -> Self {
        for list in &self.lists {
            unsafe { (*list.as_ptr()).counter += 1; }
        }

        Self {
            lists:      self.lists.clone(),
            length:     self.length,
            capacity:   self.capacity,
        }
    }
}

impl<T: Clone> FromIterator<T> for Vector<T> {

    fn from_iter<I: IntoIterator<Item = T>>(iterator: I) -> Vector<T> {
        let mut vector = Vector::new();
        for item in iterator {
            vector.push(item);
        }
        return vector;
    }
}

impl<T: Clone + PartialEq> PartialEq for Vector<T> {

    fn eq(&self, other: &Self) -> bool {
        match self.length == other.len() {
            true => return other.compare_lists(&self.lists),
            false => return false,
        }
    }
}

impl<T: Clone + Eq> Eq for Vector<T> { }

impl<T: Clone> Index<usize> for Vector<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        return self.index(index);
    }
}

impl<T: Clone> IndexMut<usize> for Vector<T> {

    fn index_mut(&mut self, index: usize) -> &mut T {
        return self.index_mut(index);
    }
}

impl<T: Clone + Debug> Debug for Vector<T> {

    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let string = self.iter().map(|item| format!("{:?}", item)).collect::<Vec<String>>().join(", ");
        return write!(f, "[{}]", string);
    }
}

impl<T: Clone + Display> Display for Vector<T> {

    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let string = self.iter().map(|item| format!("{}", item)).collect::<Vec<String>>().join(", ");
        return write!(f, "[{}]", string);
    }
}

impl<T> Drop for Vector<T> {

    fn drop(&mut self) {
        for (list_index, list_pointer) in self.lists.iter_mut().enumerate() {
            let list = unsafe { list_pointer.as_mut() };
            list.counter -= 1;
            if list.counter == 0 {
                let capacity = clamped!(self.capacity - list_index * BLOCK_LIST_SIZE, BLOCK_LIST_SIZE);
                let length = match self.length > list_index * LIST_SIZE {
                    true => clamped!(self.length - list_index * LIST_SIZE, LIST_SIZE),
                    false => 0,
                };
                list.drop_blocks(capacity, length);
                unsafe { deallocate!(list_pointer, BlockList<T>) };
            }
        }
    }
}
