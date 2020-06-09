#![allow(dead_code)]

extern crate libc;
#[cfg(feature="jemalloc")]
extern crate jemalloc_sys;


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn as_slice() {
	let heap = heap![unsafe 0, 1, 2, 3u8];

	assert_eq!(heap.as_slice(), [0,1,2,3u8]);
    }
    
    #[test]
    fn non_trivial_type() {
	let heap = heap!["test one".to_owned(), "test two".to_owned()];

	assert_eq!(heap.as_slice(), ["test one", "test two"]);
    }

    struct Unit;
    
    #[test]
    fn zero_size() {
	let heap: HeapArray<u8> = heap![];
	let heap_zst: HeapArray<()> = heap![(); 3];

	assert_eq!(&heap.as_slice(), &[]);
	assert_eq!(&heap_zst.as_slice(), &[(),(),()]);

	let heap = heap![Unit; 32];
	let mut i=0;
	for _x in heap.into_iter()
	{
	    i+=1;
	}
	assert_eq!(i, 32);
    }

    #[test]
    fn into_iter() {
	let primitive = heap![1,3,5,7,9u32];

	let iter = primitive.into_iter();
	assert_eq!(iter.len(), 5);
	for x in iter
	{
	    assert_eq!(x % 2, 1);
	}

	let non = heap!["string one".to_owned(), "string two!".to_owned(), "string".to_owned()];

	let iter = non.into_iter();
	assert_eq!(iter.len(), 3);
	for x in iter
	{
	    assert_eq!(&x[..6], "string");	    
	}
    }
    
    #[test]
    fn vec()
    {
	let heap = heap![0,1,2,3,4u8];
	let vec = vec![0,1,2,3,4u8];

	assert_eq!(&vec[..], &heap[..]);

	let heap = Vec::from(heap);

	assert_eq!(vec,heap);

	let heap = HeapArray::from(heap);

	assert_eq!(&vec[..], &heap[..]);
    }
    
    #[test]
    fn boxed_slices() {
	let array = [0,1,2,3,4];
	let vector = vec![0,1,2,3,4];
	assert_eq!(&vector[..], &array[..]);
	let slice = vector.into_boxed_slice();
	assert_eq!(&slice[..], &array[..]);
	let heap = HeapArray::from_boxed_slice(slice);
	assert_eq!(&heap[..], &array[..]);
	let slice = heap.into_boxed_slice();
	assert_eq!(&slice[..], &array[..]);
	let vector = Vec::from(slice);
	assert_eq!(&vector[..], &array[..]);
    }
}

mod ptr;
mod alloc;
mod reinterpret;

use std::{
    ops::{
	Drop,
	Index,IndexMut,
	Deref,DerefMut,
    },
    borrow::{
	Borrow,BorrowMut,
    },
    slice::{
	self,
	SliceIndex,
    },
    marker::{
	Send,
	Sync,
    },
};
use crate::{
    ptr::{
	VoidPointer,
    },
};

#[macro_export]
/// `vec![]`-like macro for creating `HeapArray<T>` instances.
///
/// Provices methods for creating safly accessable arrays using `malloc()` with a `Vec<T>` like interface.
/// Also provides methods of optimising deallocations.
///
/// # Usage
///
/// Works like array definitions `[type; size]`, and like the `vec![]` macro `[value; size]`. Prepend the statement with `unsafe` (`[unsafe type|value; size]`) to prevent potentially redundant `drop()` calls.
///
/// # Examples
///
/// ```rust
///  use malloc_array::{heap, HeapArray};
///  let ints = heap![unsafe 4u32; 32]; // Creates a 32 element `u32` array with each element set to `4`.
///  let ints = heap![unsafe u32; 32]; // Creates an uninitialised 32 element `u32` array.
///  let ints = heap![u32; 32]; // Same as above, except when `ints` is dropped, each element will be also dropped redundantly.
///  let strings = heap!["string one".to_owned(), "string two".to_owned()]; //Creates a 2 element string array.
///  let strings = heap![unsafe "memory".to_owned(), "leak".to_owned()]; //Same as above, except `drop()` will not be called on the 2 strings, potentially causing a memory leak.
///  let strings: HeapArray<u8> = heap![]; //Creates an empty `u8` array.
/// ```
macro_rules! heap {
    () => {
	$crate::HeapArray::new_uninit(0)
    };
    (@) => (0usize);
    (@ $x:tt $($xs:tt)* ) => (1usize + $crate::heap!(@ $($xs)*));

    (unsafe $($xs:tt)*) => {
	{
	    #[allow(unused_unsafe)]
	    unsafe {
		let mut output = $crate::heap!($($xs)*);
		output.drop_check = false;
		output
	    }
	}
    };
    
    ($type:ty; $number:expr) => {
	{
	    $crate::HeapArray::<$type>::new($number)
	}
    };
    ($value:expr; $number:expr) => {
	{
	    let num = $number;
	    let mut ha = $crate::HeapArray::new_uninit(num);
	    
	    for x in 0..num {
		ha[x] = $value;
	    }
	    
	    ha
	}
    };
    ($($n:expr),*) => {
	{
	    let mut ha = $crate::HeapArray::new_uninit($crate::heap!(@ $($n)*));
	    {
		let fp = 0;
		$(
		    let fp = fp + 1; 
		    ha[fp-1] = $n;
		)*
	    }
	    ha
	}
    };
}

pub struct HeapArray<T> {
    ptr: *mut T,
    size: usize,

    /// Call `drop()` on sub-elements when `drop`ping the array. This is not needed for types that implement `Copy`.
    pub drop_check: bool,
}

unsafe impl<T> Sync for HeapArray<T>
where T: Sync{}
unsafe impl<T> Send for HeapArray<T>
where T: Send{}

impl<T> HeapArray<T>
{
    pub fn len_bytes(&self) -> usize
    {
	Self::element_size() * self.size
    }
    pub fn len(&self) -> usize
    {
	self.size
    }
    
    const fn element_size() -> usize
    {
	std::mem::size_of::<T>()
    }
    const fn is_single() -> bool
    {
	std::mem::size_of::<T>() == 1
    }
    pub unsafe fn from_raw_parts(ptr: *mut T, size: usize) -> Self
    {
	Self{
	    ptr,
	    size,
	    drop_check: true,
	}
    }
    pub fn new(size: usize) -> Self
    {
	Self {
	    ptr: unsafe{alloc::calloc(size, Self::element_size()).expect("calloc()")} as *mut T,
	    size,
	    drop_check: true,
	}
    }
    pub fn new_uninit(size: usize) -> Self
    {
	Self {
	    ptr: unsafe{alloc::malloc(size * Self::element_size()).expect("malloc()")} as *mut T,
	    size,
	    drop_check: true,
	}
    }
    pub fn new_repeat(initial: T, size: usize) -> Self
    where T: Copy
    {
	let this = Self::new_uninit(size);
	if size > 0 {
	    if Self::is_single() {
		unsafe {
		    ptr::memset(this.ptr as *mut u8, reinterpret::bytes(initial), this.len_bytes());
		}
	    } else {
		unsafe {
		    for x in 0..size {
			*this.ptr.offset(x as isize) = initial;
		    }
		}
	    }
	}
	this
    }
    pub fn new_range<U>(initial: U, size: usize) -> Self
    where T: Copy,
	  U: AsRef<[T]>
    {
	let initial = initial.as_ref();
	if size > 0 {
	    if initial.len() == 1 {
		Self::new_repeat(initial[0], size)
	    } else {
		let this = Self::new_uninit(size);
		unsafe {
		    for x in 0..size {
			*this.ptr.offset(x as isize) = initial[x % initial.len()];
		    }
		    this
		}
	    }
	} else {
	    Self::new_uninit(size)
	}
    }

    pub fn as_slice(&self) -> &[T]
    {
	unsafe{slice::from_raw_parts(self.ptr, self.size)}
    }
    pub fn as_slice_mut(&mut self) -> &mut [T]
    {
	unsafe{slice::from_raw_parts_mut(self.ptr, self.size)}
    }
    pub fn as_ptr(&self) -> *const T
    {
	self.ptr as *const T
    }
    pub fn as_ptr_mut(&mut self) -> *mut T
    {
	self.ptr
    }
    pub fn memory(&self) -> &[u8]
    {
	unsafe {
	    slice::from_raw_parts(self.ptr as *const u8, self.len_bytes())
	}
    }
    pub fn memory_mut(&mut self) -> &mut [u8]
    {
	unsafe {
	    slice::from_raw_parts_mut(self.ptr as *mut u8, self.len_bytes())
	}
    }

    pub fn into_raw(self) -> (*mut T, usize)
    {
	let op = (self.ptr, self.size);
	std::mem::forget(self);
	op
    }

    pub fn free(self)
    {
	if self.ptr != ptr::null() {
	    unsafe {
		alloc::free(self.ptr as VoidPointer);
	    }
	}
	std::mem::forget(self);
    }

    pub fn into_slice(self, slice: &mut [T])
    {
	let ptr = &mut slice[0] as *mut T;
	assert!(slice.len() >= self.len());
	unsafe{
	    ptr::memmove(ptr as ptr::VoidPointer, self.ptr as ptr::VoidPointer, self.len_bytes());
	}
	self.free();
    }

    pub fn from_boxed_slice(bx: Box<[T]>) -> Self
    {
	#[cfg(feature="assume_libc")]
	unsafe {
	    let len = bx.len();
	    Self::from_raw_parts(Box::<[T]>::into_raw(bx) as *mut T, len)
	}
	#[cfg(not(feature="assume_libc"))]
	{
	    let len = bx.len();
	    let out = Self::from(Vec::from(bx));
	    assert_eq!(len, out.len());
	    out
	}
    }

    #[allow(unused_mut)]
    pub fn into_boxed_slice(mut self) -> Box<[T]>
    {
	#[cfg(feature="assume_libc")]
	unsafe {
	    let bx = Box::from_raw(self.as_slice_mut() as *mut [T]);
	    std::mem::forget(self);
	    bx
	}
	#[cfg(not(feature="assume_libc"))]
	{
	    let len = self.len();
	    let vec = Vec::from(self);
	    assert_eq!(vec.len(), len);
	    vec.into_boxed_slice()
	}
    }
}

impl<T, I> Index<I> for HeapArray<T>
where I: SliceIndex<[T]>
{
    type Output = <I as SliceIndex<[T]>>::Output;
    fn index(&self, index: I) -> &Self::Output
    {
	&self.as_slice()[index]
    }
}


impl<T, I> IndexMut<I> for HeapArray<T>
where I: SliceIndex<[T]>
{
    fn index_mut(&mut self, index: I) -> &mut <Self as Index<I>>::Output
    {
	&mut self.as_slice_mut()[index]
    }
}

impl<T> Drop for HeapArray<T>
{
    fn drop(&mut self)
    {
	if self.ptr != ptr::null::<T>() {
	    if self.drop_check {
		for i in 0..self.size
		{
		    unsafe {
			drop(ptr::take(self.ptr.offset(i as isize)));
		    }
		}
	    }
	    unsafe{alloc::free(self.ptr as VoidPointer)};
	    self.ptr = ptr::null::<T>();
	}
    }
}

impl<T> AsMut<[T]> for HeapArray<T>
{
    fn as_mut(&mut self) -> &mut [T]
    {
	self.as_slice_mut()
    }
}
impl<T> AsRef<[T]> for HeapArray<T>
{
    fn as_ref(&self) -> &[T]
    {
	self.as_slice()
    }
}

impl<T> Deref for HeapArray<T>
{
    type Target = [T];
    fn deref(&self) -> &Self::Target
    {
	self.as_slice()
    }
}
impl<T> DerefMut for HeapArray<T>
{
    fn deref_mut(&mut self) -> &mut <Self as Deref>::Target
    {
	self.as_slice_mut()
    }
}

impl<T> Borrow<[T]> for HeapArray<T>
{
    fn borrow(&self) -> &[T]
    {
	self.as_slice()
    }
}
impl<T> BorrowMut<[T]> for HeapArray<T>
{
    fn borrow_mut(&mut self) -> &mut [T]
    {
	self.as_slice_mut()
    }
}

// `From`s

impl<T> From<HeapArray<T>> for Vec<T>
{
    fn from(ha: HeapArray<T>) -> Self
    {
	let mut output = Vec::with_capacity(ha.len());
	unsafe {
	    ptr::memmove(output.as_mut_ptr() as ptr::VoidPointer, ha.ptr as ptr::VoidPointer, ha.len_bytes());
	    output.set_len(ha.len());
	}
	ha.free();
	output
    }
}
impl<T> From<Vec<T>> for HeapArray<T>
{
    fn from(vec: Vec<T>) -> Self
    {
	let mut output = HeapArray::new_uninit(vec.len());
	for (i,x) in (0..vec.len()).zip(vec.into_iter())
	{
	    output[i] = x;
	}
	output
    }
}

mod iter;
pub use iter::*;
