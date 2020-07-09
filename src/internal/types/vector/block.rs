use super::super::allocator::*;
use std::cmp::PartialEq;

macro_rules! clamped {
    ($value:expr, $max:expr) => ({
        match $value >= $max {
            true => $max,
            false => $value,
        }
    })
}

pub const BLOCK_SIZE: usize = 16;
pub const BLOCK_LIST_SIZE: usize = 8;
pub const LIST_SIZE: usize = BLOCK_SIZE * BLOCK_LIST_SIZE;

#[derive(Clone, Debug)]
pub struct Block<T> {
    pub data:       [T; BLOCK_SIZE],
    pub counter:    usize,
}

#[derive(Clone, Debug)]
pub struct BlockList<T> {
    pub blocks:     [NonNull<Block<T>>; BLOCK_LIST_SIZE],
    pub counter:    usize,
}

impl<T> Block<T> {

    pub fn drop_values(&mut self, length: usize) {
        unsafe {
            for index in 0..length {
                let address = self.data.as_ptr() as *mut T;
                let value = read(address.add(index));
                mem::drop(value);
            }
        }
    }
}

impl<T: PartialEq> Block<T> {

    pub fn compare_values(&self, other: &Block<T>, length: usize) -> bool {
        for index in 0..length {
            if self.data[index] != other.data[index] {
                return false;
            }
        }
        return true;
    }
}

impl<T> BlockList<T> {

    pub fn drop_blocks(&mut self, capacity: usize, length: usize) {
        for index in 0..capacity {
            let block = unsafe { self.blocks[index].as_mut() };
            block.counter -= 1;
            if block.counter == 0 {
                if length > index * BLOCK_SIZE {
                    block.drop_values(clamped!(length - index * BLOCK_SIZE, BLOCK_SIZE));
                }
                unsafe { deallocate!(self.blocks[index], Block<T>) };
            }
        }
    }
}

impl<T: PartialEq> BlockList<T> {

    pub fn compare_blocks(&self, other: &BlockList<T>, capacity: usize, length: usize) -> bool {
        for index in 0..capacity {
            if self.blocks[index] == other.blocks[index] {
                continue;
            }

            unsafe {
                let length = clamped!(length - index * BLOCK_SIZE, BLOCK_SIZE);
                if !self.blocks[index].as_ref().compare_values(other.blocks[index].as_ref(), length) {
                    return false;
                }
            }
        }
        return true;
    }
}
