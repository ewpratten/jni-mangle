use jni_mangle::mangle;

/// This function automatically generates the mangled name. 
/// 
/// It is also exposed to *both* Java and Rust
#[mangle(package = "com.example", class = "Example")]
pub fn my_rust_function(context: &str) {
    println!("my_rust_function ({})", context);
}

/// This function is only exposed to Java (although determined programmers can call it directly)
#[mangle(package = "com.example", class = "Example", alias=false)]
pub fn function_for_java() {
    println!("Called function_for_java");
}

/// This function adheres to both Rust and Java naming styles
#[mangle(package = "com.example", class = "Example", method = "addTwoNumbers")]
pub fn add_two_numbers(a: i32, b: i32) -> i32 {
    a + b
}

pub fn main() {

    // my_rust_function is available as both a rust function and a java one
    Java_com_example_Example_my_1rust_1function("Called using mangled name");
    my_rust_function("Called using rust name");

    // While `function_for_java` is only available as a java function, 
    // it is technically still possible to call from Rust too
    Java_com_example_Example_function_1for_1java();

    // Again, `add_two_numbers` may be called both ways (since aliases are enabled by default)
    let nums = Java_com_example_Example_addTwoNumbers(1, 2);
    println!("add_two_numbers (Java name) = {}", nums);
    let nums = add_two_numbers(1, 2);
    println!("add_two_numbers (Rust name) = {}", nums);

}
