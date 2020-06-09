use std::{
    mem::size_of,
};

#[inline]
pub unsafe fn bytes<T,U>(input: T) -> U
where T: Copy,
      U: Copy
{
    //let _array: [(); size_of::<T>() - size_of::<U>()]; // rust is silly....
    if size_of::<U>() < size_of::<T>() {
	panic!("reinterpret: Expected at least {} bytes, got {}.", size_of::<T>(), size_of::<U>());
    }
    return *((&input as *const T) as *const U)
}
