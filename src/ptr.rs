use std::{
    ffi::c_void,
    mem::{
	self,
	MaybeUninit,
    },
};
use libc::{
    size_t,
    c_int,
};

pub type VoidPointer = *mut ();
pub type ConstVoidPointer = *const ();

pub const NULL_PTR: VoidPointer = 0 as VoidPointer;

pub fn null<T>() -> *mut T
{
    NULL_PTR as *mut T
}

pub unsafe fn memset(ptr: *mut u8, value: u8, length: usize)
{
    libc::memset(ptr as *mut c_void, value as c_int, length as size_t);
}

pub unsafe fn replace<T>(ptr: *mut T, value: T) -> T
{
    mem::replace(&mut *ptr, value)
}
pub unsafe fn take<T>(ptr: *mut T) -> T
{
    mem::replace(&mut *ptr, MaybeUninit::zeroed().assume_init())
}

pub unsafe fn memcpy(dst: VoidPointer, src: ConstVoidPointer, size: usize) -> VoidPointer
{
    libc::memcpy(dst as *mut c_void, src as *const c_void, size as size_t) as VoidPointer
}
pub unsafe fn memmove(dst: VoidPointer, src: ConstVoidPointer, size: usize) -> VoidPointer
{
    libc::memmove(dst as *mut c_void, src as *const c_void, size as size_t) as VoidPointer
}
