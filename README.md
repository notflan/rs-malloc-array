# malloc-array - `Vec<T>` like `malloc()` wrapper

This crate provides a `vec!`-like macro, `heap!` for creating arrays managed with `malloc()` and `free()`. It also provides the container type `HeapArray<T>` as a safe wrapper around these.

See [documentation] for more details.

[documentation]: https://docs.rs/malloc-array

## Macro usage

### Creating zero-initialised arrays.
These are created with `calloc()`.
``` rust
heap![Type, size];
```
Note that if `Type` does not support zero-initialisation it is undefined behaviour to drop or access any element of the returned array. To assign without dropping see the associated function [replace_and_forget]:

``` rust
let mut array = heap![String; 3];
array.replace_and_forget(0, format!("snibbedy"));
array.replace_and_forget(1, format!("snab"));
array.replace_and_forget(2, format!(":D"));

drop(array); // This is now safe.
```

 [replace_and_forget]: https://docs.rs/malloc-array/1.0.0/malloc_array/struct.HeapArray.html#method.replace_and_forget
 
#### Alternatively initialising with iterator
The library also provides the `InitIter` type, which is a mutable iterator for `HeapArray<T>` that allows you to safely initialise porentially uninitialised elements.

``` rust
let mut array = heap![String; 10];
for mut init in array.initialise()
{
	init.set(format!("string!"));
	// Also see docs for `init::Init` type.
}
drop(array); // This is now safe.
```
##### Filling the iterator
The iterator also provides methods to fill itself of uninitialised values.

###### Fill with `Clone`
``` rust
array.initialise().fill("value".to_owned());
```
###### Fill with lambda
``` rust
array.initialise().fill_with(|| "value".to_owned());
```
###### Fill with `Default`
``` rust
array.initialise().fill_default();
```
###### Uninitialise the memory
Since it is unknown if the type `T` supports zero-initialisation, zeroing the memory is counted as making it uninitialised.
``` rust
array.initialise().uninit(); //Sets all the rest of the iterator bytes to 0.
```

### Creating initialised arrays.
These are created with `malloc()` and set with `replace_and_forget` (or, for the special case of `u8` sized types, `memset`).
``` rust
heap![expression; size];
```

### Creating n-element arrays.
These are created with `malloc()` and set with `replace_and_forget`.
``` rust
heap![expression_one, expression_two];
```

### Creating empty arrays.
These are created with either `malloc(0)`, or if the `zst_noalloc` feature is enabled they do not allocate.
``` rust
heap![];
```
`zst_noalloc` is enabled by default and causes instances with `len_bytes() == 0` to have `NULL` internal pointers instead of dangling ones returned by `malloc(0)`.
This behaviour may not be desireable and if it is not, disable the default featues.

### Dropping on free
Arrays created this way are dropped in a way that ensures each element is also dropped. For anything implementing the `Copy` trait, this is redundant.
To avoid this, pass the keyword `unsafe` to any of the above macro definitions:
``` rust
let bytes = heap![unsafe u8; 32]; //`u8` does not need to be dropped. 
let references = heap![unsafe ":D"; 10]; //Neither does `&'static str`.
```
Note that if the type does implement the `Drop` trait, then unless the elements are dropped manually (see [into_iter]) dropping the array can cause a resource leak.

 [into_iter]: https://docs.rs/malloc-array/1.0.0/malloc_array/struct.IntoIter.html

# License
GPL'd with love <3
