use super::*;

use std::{
    mem::{
	forget,
    },
    marker::{
	Send,Sync,
    },
    ops::{
	Drop,
    },
};
use ptr::{
    VoidPointer,
};

/// An iterator that consumes `HeapArray<T>` instance and ensures all memory is appropriately freed when consumed or dropped.
pub struct IntoIter<T>
{
    start: *mut T,
    current_offset: usize,
    sz: usize,
}

unsafe impl<T: Send> Send for IntoIter<T>{}
unsafe impl<T: Sync> Sync for IntoIter<T>{} //this is probably fine right?

impl<T> IntoIter<T>
{
    fn current(&mut self) -> *mut T
    {
	unsafe {
	    self.start.offset(self.current_offset as isize)
	}
    }
    fn free_if_needed(&mut self)
    {
	if self.start != ptr::null() && self.current_offset >= self.sz {
	    unsafe {
		alloc::free(self.start as VoidPointer);
	    }
	    self.start = ptr::null();
	}
    }
    fn drain_if_needed(&mut self)
    {
	if self.start != ptr::null() {

	    unsafe {
		if self.current_offset<self.sz {
		    for i in self.current_offset..self.sz
		    {
			drop(ptr::take(self.start.offset(i as isize)));
		    }
		}

		alloc::free(self.start as VoidPointer);
	    }
	    self.start = ptr::null();
	}
    }
}

impl<T> Drop for IntoIter<T>
{
    fn drop(&mut self)
    {
	self.drain_if_needed();
    }
}

impl<T> Iterator for IntoIter<T>
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item>
    {
	let output = if self.current_offset >= self.sz || self.sz == 0 {
	    None
	} else {
	    unsafe {
		let output = crate::ptr::take(self.current());//replace(&mut (*self.current), MaybeUninit::zeroed().assume_init());
		self.current_offset+=1;

		Some(output)
	    }
	};
	self.free_if_needed();
	output
    }
}

impl<T> ExactSizeIterator for IntoIter<T>
{
    fn len(&self) -> usize
    {
	self.sz
    }
}

impl<T> IntoIterator for HeapArray<T>
{
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter
    {
	let output = Self::IntoIter {
	    start: self.ptr,
	    current_offset: 0,
	    sz: self.len(),
	};
	forget(self);
	output
    }
}
