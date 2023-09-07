# Rust function mangler for JNI
[![Crates.io](https://img.shields.io/crates/v/jni-mangle)](https://crates.io/crates/jni-mangle)
[![Docs.rs](https://docs.rs/jni-mangle/badge.svg)](https://docs.rs/jni-mangle)
[![Build](https://github.com/Ewpratten/jni-mangle/actions/workflows/build.yml/badge.svg)](https://github.com/Ewpratten/jni-mangle/actions/workflows/build.yml)
[![Clippy](https://github.com/Ewpratten/jni-mangle/actions/workflows/clippy.yml/badge.svg)](https://github.com/Ewpratten/jni-mangle/actions/workflows/clippy.yml)

The `jni-mangle` crate provides proc macros for working with Rust functions that are called from Java through JNI.

The main purpose of this crate is to turn rust functions that might look like this:

```rust
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_example_Example_addTwoNumbers(a: i32, b: i32) -> i32 {
   a + b    
}
```

Into something a little more readable:

```rust
use jni_mangle::mangle;

#[mangle(package="com.example", class="Example", method="addTwoNumbers")]
pub fn add_two_numbers(a: i32, b: i32) -> i32 {
   a + b    
}
```
