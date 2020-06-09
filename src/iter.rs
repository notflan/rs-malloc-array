use super::*;

use std::{
    mem::{
	replace,
	MaybeUninit,
	forget,
    },
};
use ptr::{
    VoidPointer,
};

pub struct IntoIter<T>
{
    start: *mut T,
    current: *mut T,
    sz: usize,
}

impl<T> IntoIter<T>
{
    fn current_offset(&self) -> usize
    {
	(self.current as usize) - (self.start as usize)
    }
    fn free_if_needed(&mut self)
    {
	if self.start != ptr::null() && self.current_offset() >= self.sz {
	    unsafe {
		alloc::free(self.start as VoidPointer);
	    }
	    self.start = ptr::null();
	}
    }
}

impl<T> Iterator for IntoIter<T>
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item>
    {
	let output = if self.current_offset() >= self.sz {
	    None
	} else {
	    unsafe {
		let output = replace(&mut (*self.current), MaybeUninit::zeroed().assume_init());
		self.current = self.current.offset(1);
		Some(output)
	    }
	};
	self.free_if_needed();
	output
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
	    current: self.ptr,
	    sz: self.len(),
	};
	forget(self);
	output
    }
}
