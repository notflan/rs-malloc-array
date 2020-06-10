use super::*;
use std::{
    marker::PhantomData,
};

pub struct InitIter<'a, T>
{
    from: &'a mut HeapArray<T>,
    current_idex: usize,
}

pub struct Init<'a, T>
{
    ptr: *mut T,
    init_ok: bool,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> InitIter<'a, T>
{
    pub(crate) fn new(from: &'a mut HeapArray<T>, current_idex: usize) -> InitIter<'a, T>
    {
	InitIter {
	    from,
	    current_idex,
	}
    }
}

impl<'a, T> Iterator for InitIter<'a, T>
{
    type Item = Init<'a, T>;

    fn next(&mut self) -> Option<Self::Item>
    {
	if self.current_idex >= self.from.len() {
	    None
	} else {
	    self.current_idex+=1;
	    unsafe {
		Some(Init{
		    ptr: self.from.as_mut_ptr().offset((self.current_idex as isize)-1),
		    init_ok: false,
		    _marker: PhantomData,
		})
	    }
	}
    }
}

impl<'a, T> Init<'a, T>
{
    pub fn is_init(&self) -> bool
    {
	self.init_ok
    }
    pub unsafe fn assume_init(&mut self)
    {
	self.init_ok = true;
    }
    pub fn put(&mut self, value: T) -> &mut T
    {
	if self.init_ok {
	    unsafe {
		*self.ptr = value;	
		return &mut (*self.ptr);
	    }
	}
	self.init_ok = true;
	unsafe {
	    ptr::put(self.ptr, value);
	    &mut (*self.ptr)
	}
    }
    pub fn get(&self) -> Option<&T>
    {
	unsafe {
	    if self.init_ok {
		Some(& (*self.ptr))
	    } else {
		None
	    }
	}
    }
    pub fn get_mut(&mut self) -> Option<&mut T>
    {
	unsafe {
	    if self.init_ok {
		Some(&mut (*self.ptr))
	    } else {
		None
	    }
	}
    }
}
