use super::*;
use std::{
    marker::PhantomData,
};

/// Iterator for initialising potentially uninitialised `HeapArray<T>`.
pub struct InitIter<'a, T>
{
    from: &'a mut HeapArray<T>,
    current_idex: usize,
}

/// A safe wrapper to initialise potentially uninitialised data.
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

    /// Consumes the instance, zeroing all remaining bytes in the iterator.
    pub fn uninit(self)
    {
	let len = self.from.len_bytes() - (self.current_idex * HeapArray::<T>::element_size());
	if len > 0 {
	    unsafe {
		ptr::memset(self.from.as_mut_ptr().offset(self.current_idex as isize) as *mut u8, 0, len);
	    }
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

pub trait InitIterExt<T>
{

    /// Fill the rest of the iterator with a `clone()`d value.
    fn fill(self, value: T) where T: Clone;

    /// Fill the rest of the iterator with output from a function.
    fn fill_with<F>(self, func: F) where F: FnMut() -> T;

    /// Fill the rest of the iterator with `default()`
    fn fill_default(self) where T: Default;
}

impl<'a, T,I> InitIterExt<T> for I
where I: Iterator<Item=Init<'a, T>>,
      T: 'a
{
    fn fill(self, value: T)
    where T:Clone
    {
	for mut x in self
	{
	    if !x.is_init() {
		x.put(value.clone());
	    }
	}
    }
    fn fill_with<F>(self, mut func: F)
    where F: FnMut() -> T
    {
	for mut x in self
	{
	    if !x.is_init() {
		x.put(func());
	    }
	}
    }
    fn fill_default(self)
    where T:Default
    {
	for mut x in self
	{
	    if !x.is_init() {
		x.put(Default::default());
	    }
	}
    }
}

impl<'a, T> Init<'a, T>
{

    /// Has the value been set with `put()` or `assume_init()` yet?
    pub fn is_init(&self) -> bool
    {
	self.init_ok
    }

    /// Assume the value has been initialised.
    pub unsafe fn assume_init(&mut self)
    {
	self.init_ok = true;
    }

    /// Initialise or reset the value and then return a mutable reference to it.
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

    /// Get a reference to the value if it has been initialised.
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

    /// Get a mutable reference to the value if it has been initialised.
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
