use std::{
    ffi::c_void,
    error,
    fmt,
};
use crate::{
    ptr::{self,VoidPointer,},
};

#[derive(Debug)]
pub struct Error;

impl error::Error for Error{}
impl fmt::Display for Error
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
	write!(f, "Allocation failed.")
    }
}

#[inline]
unsafe fn malloc_internal(sz: libc::size_t) -> *mut c_void
{
    #[cfg(feature="jemalloc")]
    return jemalloc_sys::malloc(sz);
    #[cfg(not(feature="jemalloc"))]
    return libc::malloc(sz);
}
#[inline]
unsafe fn calloc_internal(nm: libc::size_t, sz: libc::size_t) -> *mut c_void
{
    #[cfg(feature="jemalloc")]
    return jemalloc_sys::calloc(nm,sz);
    #[cfg(not(feature="jemalloc"))]
    return libc::calloc(nm,sz);
}
#[inline]
unsafe fn free_internal(ptr: *mut c_void)
{
    #[cfg(feature="jemalloc")]
    return jemalloc_sys::free(ptr);
    #[cfg(not(feature="jemalloc"))]
    return libc::free(ptr);
}
#[inline]
unsafe fn realloc_internal(ptr: *mut c_void, sz: libc::size_t) -> *mut c_void
{
    #[cfg(feature="jemalloc")]
    return jemalloc_sys::realloc(ptr,sz);
    #[cfg(not(feature="jemalloc"))]
    return libc::realloc(ptr,sz);
}

const NULL_PTR: *mut c_void = 0 as *mut c_void;

pub unsafe fn malloc(sz: usize) -> Result<VoidPointer,Error>
{
    #[cfg(feature="zst_noalloc")]
    if sz == 0 {
	return Ok(ptr::NULL_PTR);
    }
    
    match malloc_internal(sz as libc::size_t)
    {
	null if null == NULL_PTR => Err(Error),
	ptr => Ok(ptr as VoidPointer),
    }
}

pub unsafe fn calloc(nm: usize, sz: usize) -> Result<VoidPointer, Error>
{
    #[cfg(feature="zst_noalloc")]
    if (nm*sz) == 0 {
	return Ok(ptr::NULL_PTR);
    }
    
    match calloc_internal(nm as libc::size_t, sz as libc::size_t)
    {
	null if null == NULL_PTR => Err(Error),
	ptr => Ok(ptr as VoidPointer),
    }
}

pub unsafe fn free(ptr: VoidPointer)
{
    if ptr != crate::ptr::NULL_PTR {
	free_internal(ptr as *mut c_void);
    }
}

pub unsafe fn realloc(ptr: VoidPointer, sz: usize) -> Result<VoidPointer, Error>
{
    #[cfg(feature="zst_noalloc")]
    if sz == 0 {
	free(ptr);
	return Ok(crate::ptr::NULL_PTR);
    }

    if ptr == crate::ptr::NULL_PTR {
	return malloc(sz);
    }
    
    match realloc_internal(ptr as *mut c_void, sz as libc::size_t)
    {
	null if null == NULL_PTR => Err(Error),
	ptr => Ok(ptr as VoidPointer),
    }
}
