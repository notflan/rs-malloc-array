#![allow(dead_code)]

extern crate libc;
#[cfg(feature="jemalloc")]
extern crate jemalloc_sys;


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_bytes()
    {
	unsafe {
	    let heap = HeapArray::<i32>::from_bytes(&[0xff,0xff,0xff,0xff,0,0,0,0,0xff,0xff,0xff,0xff]);
	    assert_eq!(heap[0], -1);
	    assert_eq!(heap[1], 0);
	    assert_eq!(heap[2], -1);
	}
    }
    
    #[test]
    fn copy() {
	let heap = heap![unsafe 1u16; 10];
	let mut heap2 = heap![unsafe 10u16; 20];

	unsafe {
	    assert_eq!(heap2.memory_from_raw(heap.as_ptr(), heap.len()), 10);
	}

	assert_eq!(heap2[0], 1);
	assert_eq!(heap2[10], 10);

	unsafe {
	    let heap3 = HeapArray::from_raw_copied(heap2.as_ptr(), 15);
	    assert_eq!(heap3.len(), 15);
	    assert_eq!(heap3[0], 1);
	    assert_eq!(heap3[10], 10);
	}
    }
    
    #[test]
    fn as_slice() {
	let heap = heap![unsafe 0, 1, 2, 3u8];

	assert_eq!(heap.as_slice(), [0,1,2,3u8]);
    }
    
    #[test]
    fn non_trivial_type() {
	let heap = heap!["test one".to_owned(), "test two".to_owned()];
	let refs = heap![unsafe "test three"; 2];
	
	assert_eq!(&refs[..], &["test three", "test three"]);
	assert_eq!(heap.as_slice(), ["test one", "test two"]);
    }

    struct Unit;

    #[test]
    fn reinterpret()
    {
	let heap = heap![0u8; 32];
	unsafe {
	    let heap = heap.reinterpret::<i32>();
	    assert_eq!(heap.len(), 8);
	    let heap = heap.reinterpret::<u8>();
	    assert_eq!(heap.len(), 32);
	}
    }
    
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

	let non = heap!["strings".to_owned(), "strings!!!".to_owned()];
	let iter = non.into_iter();
	drop(iter);
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

    #[test]
    fn init()
    {
	let mut array = heap![String; 32];
	for mut string in array.initialise()
	{
	    string.put("Hiya".to_owned());
	}
	assert_eq!(array.len(), 32);
	for x in array.into_iter()
	{
	    assert_eq!(x, "Hiya");
	}

	let mut array = heap![String; 10];
	array.initialise().fill("wowe".to_owned());
	for x in array.into_iter()
	{
	    assert_eq!(x, "wowe");
	}

	

	let mut array = heap![String; 10];
	array.initialise().fill_with(|| "wow".to_owned());
	for x in array.into_iter()
	{
	    assert_eq!(x, "wow");
	}

	

	let mut array = heap![String; 10];
	array.initialise().fill_default();
	for x in array.into_iter()
	{
	    assert_eq!(x, "");
	}
    }
}

mod ptr;
mod alloc;
mod reinterpret;
pub mod init;
pub use init::InitIterExt;
pub mod store;

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
	ConstVoidPointer,
    },
};

#[macro_export]
/// `vec![]`-like macro for creating `HeapArray<T>` instances.
///
/// Provides methods for creating safely accessable arrays using `malloc()` with a `Vec<T>` like interface.
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

	    if ha.len() == ha.len_bytes() && ha.len() > 0 {
		unsafe {
		    let mut vl = $value;
		    
		    ha.set_memory(*std::mem::transmute::<_, &mut u8>(&mut vl));
		}
	    } else {
		for x in 0..num {
		    ha.replace_and_forget(x, $value);
		}
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
		    ha.replace_and_forget(fp-1, $n);
		)*
	    }
	    ha
	}
    };
}

/// Array created by libc `malloc()` and dropped by libc `free()`.
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
    /// Size of memory of this instance in bytes.
    pub fn len_bytes(&self) -> usize
    {
	Self::element_size() * self.size
    }

    /// Number of elements in this instance.
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

    /// Create an iterator for safely setting potentially uninitialised values within the instance.
    pub fn initialise<'a>(&'a mut self) -> init::InitIter<'a, T>
    {
	init::InitIter::new(self, 0)
    }

    /// Set each byte to a value.
    pub unsafe fn set_memory(&mut self, value: u8)
    {
	ptr::memset(self.ptr as *mut u8, value, self.len_bytes());
    }

    /// Creates a new `HeapArray<T>` from zeroed memory.
    pub fn new(size: usize) -> Self
    {
	Self {
	    ptr: unsafe{alloc::calloc(size, Self::element_size()).expect("calloc()")} as *mut T,
	    size,
	    drop_check: true,
	}
    }

    /// Creates a new `HeapArray<T>` from uninitialised memory.
    pub fn new_uninit(size: usize) -> Self
    {
	Self {
	    ptr: unsafe{alloc::malloc(size * Self::element_size()).expect("malloc()")} as *mut T,
	    size,
	    drop_check: true,
	}
    }

    /// Consumes the instance, returning a new instance after calling `realloc()` on the underlying memory.
    pub fn resize(self, size: usize) -> Self
    {
	unsafe {
	    let ptr = alloc::realloc(self.ptr as VoidPointer, size).expect("realloc()") as *mut T;
	    
	    let output = Self {
		ptr,
		size,
		drop_check: self.drop_check
	    };
	    std::mem::forget(self);
	    output
	}
    }

    /// Creates a new `HeapArray<T>` from an initial element and a size.
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

    /// Creates a new `HeapArray<T>` from a range of elements and a size, repeating if needed.
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

    /// As an immutable slice of `T`.
    pub fn as_slice(&self) -> &[T]
    {
	unsafe{slice::from_raw_parts(self.ptr, self.size)}
    }

    /// As a mutable slice of `T`.
    pub fn as_slice_mut(&mut self) -> &mut [T]
    {
	unsafe{slice::from_raw_parts_mut(self.ptr, self.size)}
    }

    /// As immutable raw pointer.
    pub fn as_ptr(&self) -> *const T
    {
	self.ptr as *const T
    }

    /// As mutable raw pointer.
    pub fn as_ptr_mut(&mut self) -> *mut T
    {
	self.ptr
    }

    /// An immutable slice of the memory.
    pub fn memory(&self) -> &[u8]
    {
	unsafe {
	    slice::from_raw_parts(self.ptr as *const u8, self.len_bytes())
	}
    }

    /// A mutable slice of the memory.
    pub unsafe fn memory_mut(&mut self) -> &mut [u8]
    {
	slice::from_raw_parts_mut(self.ptr as *mut u8, self.len_bytes())
    }


    /// Consumes the instance. Returns a raw pointer and the number of elements.
    pub fn into_raw_parts(self) -> (*mut T, usize)
    {
	let op = (self.ptr, self.size);
	std::mem::forget(self);
	op
    }

    /// Create a `HeapArray<T>` from a raw pointer and a number of elements.
    pub unsafe fn from_raw_parts(ptr: *mut T, size: usize) -> Self
    {
	Self {
	    ptr,
	    size,
	    drop_check: true,
	}
    }

    /// Consumes the instance. Frees the memory without dropping the items.
    pub fn free(self)
    {
	if self.ptr != ptr::null() {
	    unsafe {
		alloc::free(self.ptr as VoidPointer);
	    }
	}
	std::mem::forget(self);
    }

    /// Consumes the instance, moving all elements into a slice.
    pub fn into_slice(self, slice: &mut [T])
    {
	let ptr = &mut slice[0] as *mut T;
	assert!(slice.len() >= self.len());
	unsafe{
	    ptr::memmove(ptr as ptr::VoidPointer, self.ptr as ptr::VoidPointer, self.len_bytes());
	}
	self.free();
    }

    /// Coerce or clone memory from a boxed slice.
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

    /// Coerce or clone memory into a boxed slice.
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

    /// Reinterpret the memory of this instance into an insteance of a different type
    /// # Panics
    /// If `U` cannot fit into `T`.  
    pub unsafe fn reinterpret<U>(self) -> HeapArray<U>
    {
	assert!(self.len_bytes() % std::mem::size_of::<U>() == 0);
	let output = HeapArray {
	    size: self.len_bytes() / std::mem::size_of::<U>(),
	    ptr: self.ptr as *mut U,
	    drop_check: self.drop_check,
	};
	std::mem::forget(self);
	output
    }

    /// Reinterpret the memory of this instance into an immutable slice of a different type.
    /// # Panics
    /// If `U` cannot fit into `T`.  
    pub fn reinterpret_ref<U>(&self) -> &[U]
    {
	assert!(self.len_bytes() % std::mem::size_of::<U>() == 0);
	unsafe {
	    slice::from_raw_parts(self.ptr as *const U, self.len_bytes() / std::mem::size_of::<U>())
	}
    }
    /// Reinterpret the memory of this instance into a mutable slice of a different type.
    /// # Panics
    /// If `U` cannot fit into `T`.  
    pub unsafe fn reinterpret_mut<U>(&mut self) -> &mut [U]
    {
	assert!(self.len_bytes() % std::mem::size_of::<U>() == 0);
	slice::from_raw_parts_mut(self.ptr as *mut U, self.len_bytes() / std::mem::size_of::<U>())
    }

    /// Immutable slice iterator for this instance
    pub fn iter<'a>(&'a self) -> slice::Iter<'a, T>
    {
	self.as_slice().iter()
    }

    /// Mutable slice iterator for this instance
    pub fn iter_mut<'a>(&'a mut self) -> slice::IterMut<'a, T>
    {
	self.as_slice_mut().iter_mut()
    }

    /// Replace the element at `index` with `value`, and `forget` the old one.
    /// Useful with `new_uninit()`.
    pub fn replace_and_forget(&mut self, index: usize, value: T)
    {
	assert!(index<self.len());
	unsafe {
	    ptr::put(self.as_ptr_mut().offset(index as isize), value);
	}
    }


    /// Clone the memory to a new instance.
    pub unsafe fn clone_mem(&self) -> Self
    {
	let mut output = Self::new_uninit(self.len());
	output.drop_check = self.drop_check;
	ptr::memcpy(output.ptr as VoidPointer, self.ptr as VoidPointer, self.len_bytes());

	output
    }

    /// Leak the memory to a static slice reference.
    pub fn leak(mut self) -> &'static mut [T]
    {
	unsafe {
	    let bx = Box::from_raw(self.as_slice_mut() as *mut [T]);
	    std::mem::forget(self);
	    Box::leak(bx)
	}
    }

    /// Copy memory in from a slice of bytes.
    pub unsafe fn memory_from_bytes<U: AsRef<[u8]>>(&mut self, from: U) -> usize
    {
	let from = from.as_ref();
	let size = std::cmp::min(from.len(), self.len_bytes());
	ptr::memcpy(self.ptr as VoidPointer, &from[0] as *const u8 as ConstVoidPointer, size);
	size
    }

    /// Copy memory in from a pointer to bytes.
    pub unsafe fn memory_from_raw_bytes(&mut self, from: *const u8, size: usize) -> usize
    {
	let size = std::cmp::min(size, self.len_bytes());
	ptr::memcpy(self.ptr as VoidPointer, from as *const u8 as ConstVoidPointer, size);
	size
    }

    /// Copy memory in from a raw pointer.
    pub unsafe fn memory_from_slice<U: AsRef<[T]>>(&mut self, from: U) -> usize
    {
	let from = from.as_ref();
	let size = std::cmp::min(from.len(), self.len());
	ptr::memcpy(self.ptr as VoidPointer, &from[0] as *const T as ConstVoidPointer, size * std::mem::size_of::<T>());
	size
    }
    
    /// Copy memory in from a raw pointer.
    pub unsafe fn memory_from_raw(&mut self, from: *const T, size: usize) -> usize
    {
	let size = std::cmp::min(size, self.len());
	ptr::memcpy(self.ptr as VoidPointer, from as *const T as ConstVoidPointer, size * std::mem::size_of::<T>());
	size
    }

    /// Create a new instance with memory copied from a raw pointer.
    pub unsafe fn from_raw_copied(from: *const T, size: usize) -> Self
    {
	let mut inp = Self::new_uninit(size);
	inp.memory_from_raw(from, size);
	inp
    }
    
    /// Create a new instance with memory copied from a slice.
    pub unsafe fn from_slice_copied<U: AsRef<[T]>>(from: U) -> Self
    where T: Copy
    {
	let from = from.as_ref();
	Self::from_raw_copied(&from[0] as *const T, from.len())
    }

    /// Create a new instance with memory bytes copied from a raw pointer.
    pub unsafe fn from_raw_bytes(from: *const u8, size: usize) -> Self
    {
	assert_eq!(size % Self::element_size(),0,"Cannot fit T into this size.");
	
	let mut inp = Self::new_uninit(size / Self::element_size());
	inp.memory_from_raw_bytes(from, size);
	inp
    }
    
    /// Create a new instance with memory bytes copied from a slice.
    pub unsafe fn from_bytes<U: AsRef<[u8]>>(from: U) -> Self
    {
	let from = from.as_ref();
	Self::from_raw_bytes(&from[0], from.len())
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

impl<T> From<Box<[T]>> for HeapArray<T>
{
    fn from(sl: Box<[T]>) -> Self
    {
	Self::from_boxed_slice(sl)
    }
}
impl<T> From<HeapArray<T>> for Box<[T]>
{
    fn from(ha: HeapArray<T>) -> Self
    {
	ha.into_boxed_slice()
    }
}

mod iter;
pub use iter::*;

impl<T> std::cmp::Eq for HeapArray<T>
where T: std::cmp::Eq {}
impl<T, U> std::cmp::PartialEq<U> for HeapArray<T>
where T: std::cmp::PartialEq,
      U: AsRef<[T]>
{
    fn eq(&self, other: &U) -> bool
    {
	let other = other.as_ref();
	self.len() == other.len() &&
	{
	    for (x, y) in self.iter().zip(0..other.len()) {
		if x != &other[y] {return false;}
	    }
	    true
	}
    }
}

impl<T> std::hash::Hash for HeapArray<T>
where T: std::hash::Hash
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H)
    {
	self.size.hash(state);
	self.as_slice().hash(state);
    }
}

impl<T> Clone for HeapArray<T>
where T: Clone
{
    fn clone(&self) -> Self
    where T: Clone
    {
	let mut output = Self::new_uninit(self.len());
	output.drop_check = self.drop_check;

	unsafe {
	    for (i,x) in (0..self.len()).zip(self.iter())
	    {   
		ptr::put(output.as_ptr_mut().offset(i as isize), x.clone());
	    }
	}
	output
    }
}

use std::fmt;
impl<T> fmt::Debug for HeapArray<T>
where T: fmt::Debug
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
	write!(f, "{}: (", std::any::type_name::<Self>())?;
	let len = self.len();
	for (x,i) in self.iter().zip(0..len)
	{
	    write!(f, "{:?}", x)?;
	    if i < len-1 {
		write!(f, " ")?;
	    }
	}
	write!(f, ")")
    }
}
