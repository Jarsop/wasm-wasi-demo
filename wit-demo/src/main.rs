mod bindings;

use crate::bindings::test::wit_test::http::{request, Method};

fn main() {
    let response = request(Method::Post, "https://example.com", "Hello, world!");
    println!("{:?}", response);
}
