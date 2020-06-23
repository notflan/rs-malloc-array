use crate::*;

/// Statically typed pointer store. `free()`s and drops on drop.
#[derive(Debug)]
pub struct Store<T>
{
    pointers: Vec<*mut T>,
}

#[cfg(test)]
mod tests
{
    use super::*;
    #[test]
    fn store()
    {
	let mut store = Store::new();
	unsafe {
	    for _a in 0..10 {
		let _ptr = store.ptr(alloc::malloc(100).unwrap());
	    }
	}
    }
    #[test]
    fn dyn_store()
    {
	let mut store = DynStore::new();
	unsafe {
	    for _a in 0..10 {
		let ptr = store.ptr(alloc::malloc(4).unwrap() as *mut u32);
		*ptr = 100u32;
	    }
	}
	
    }
    #[test]
    fn into_ha()
    {
	let mut store = Store::new();
	unsafe {
	    for a in 0..10 {
		*store.ptr(alloc::malloc(4).unwrap() as *mut u32) = a;
	    }
	}
	let ha = store.into_heap_array();
	assert_eq!(ha.len(), 10);
	assert_eq!(&ha[..], &[0,1,2,3,4,5,6,7,8,9]);
    }
}

impl<T> Store<T>
{
    /// Create a new pointer store.
    pub fn new() -> Self
    {
	Self{pointers:Vec::new()}
    }

    /// Add a pointer to the store.
    pub fn ptr(&mut self, ptr: *mut T) -> *mut T
    {
	self.pointers.push(ptr);
	ptr
    }

    /// Remove a pointer from the store.
    pub fn remove(&mut self, ptr: *mut T)
    {
	while let Some(i) = self.pointers.iter().position(|x| *x == ptr)
	{
	    self.pointers.remove(i);
	}
    }

    /// Consumes the instance and returns the pointers without freeing them.
    pub fn into_raw_parts(mut self) -> Vec<*mut T>
    {
	std::mem::replace(&mut self.pointers, Vec::new())
    }

    /// Consume a vector of pointers and return a new `Store<T>`.
    pub fn from_raw_parts(pointers: Vec<*mut T>) -> Self
    {
	Self {
	    pointers,
	}
    }
    
    /// Free all the pointers in the store without calling their destructors (if the have any).
    pub fn free(mut self)
    {
	for &mut x in self.pointers.iter_mut()
	{
	    unsafe {
		alloc::free(x as VoidPointer);
	    }
	}
	self.pointers.clear()
    }

    /// Move all data from all pointers into a new `HeapArray<T>` instance and free the old pointers.
    pub fn into_heap_array(mut self) -> HeapArray<T>
    {
	let mut output = heap![T; self.pointers.len()];
	for (mut init, old) in output.initialise().zip(std::mem::replace(&mut self.pointers, Vec::new()).into_iter())
	{
	    unsafe {
		init.put(ptr::take(old));
		alloc::free(old as *mut ());
	    }
	}
	output
    }
}

impl<T> std::ops::Drop for Store<T>
{
    fn drop(&mut self)
    {
	for &mut ptr in self.pointers.iter_mut()
	{
	    unsafe {
		drop(ptr::take(ptr));
		alloc::free(ptr as VoidPointer);
	    }
	}
	self.pointers.clear();
    }
}

/// Dynamically typed pointer store. Frees on drop.
#[derive(Debug)]
pub struct DynStore
{
    pointers: Vec<VoidPointer>,
}

impl DynStore
{
    /// Create a new pointer store.
    pub fn new() -> Self
    {
	Self{pointers:Vec::new()}
    }

    /// Add a pointer to the store.
    pub fn ptr<T>(&mut self, ptr: *mut T) -> *mut T
    {
	self.pointers.push(ptr as VoidPointer);
	ptr
    }

    /// Remove a pointer from the store.
    pub fn remove<T>(&mut self, ptr: *mut T)
    {
	while let Some(i) = self.pointers.iter().position(|x| *x == ptr as VoidPointer)
	{
	    self.pointers.remove(i);
	}
    }

    /// Consumes the instance and returns the pointers without freeing them.
    pub fn into_raw_parts(mut self) -> Vec<*mut ()>
    {
	std::mem::replace(&mut self.pointers, Vec::new())
    }
    
    /// Consume a vector of pointers and return a new `Store<T>`.
    pub fn from_raw_parts(pointers: Vec<*mut ()>) -> Self
    {
	Self {
	    pointers,
	}
    }

    
    /// Free all the pointers in the store without calling their destructors (if the have any).
    pub fn free(mut self)
    {
	for &mut x in self.pointers.iter_mut()
	{
	    unsafe {
		alloc::free(x);
	    }
	}
	self.pointers.clear()
    }
    
}

impl std::ops::Drop for DynStore
{
    fn drop(&mut self)
    {
	for &mut ptr in self.pointers.iter_mut()
	{
	    unsafe {
		drop(ptr::take(ptr));
		alloc::free(ptr);
	    }
	}
	self.pointers.clear();
    }
}

impl<T> IntoIterator for Store<T>
{
    type Item = *mut T;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(mut self) -> Self::IntoIter
    {
	std::mem::replace(&mut self.pointers, Vec::new()).into_iter()
    }
}

impl IntoIterator for DynStore
{
    type Item = *mut ();
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(mut self) -> Self::IntoIter
    {
	std::mem::replace(&mut self.pointers, Vec::new()).into_iter()
    }
}

use std::iter::FromIterator;
impl<T> FromIterator<*mut T> for Store<T>
{
    fn from_iter<I: IntoIterator<Item=*mut T>>(iter: I) -> Self
    {
	Self {
	    pointers: Vec::from_iter(iter)
	}
    }
}

impl<T> FromIterator<*mut T> for DynStore
{
    fn from_iter<I: IntoIterator<Item=*mut T>>(iter: I) -> Self
    {
	Self {
	    pointers: Vec::from_iter(iter.into_iter().map(|x| x as *mut ()))
	}
    }
}
