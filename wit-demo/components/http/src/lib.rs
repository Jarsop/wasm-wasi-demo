mod bindings;

use crate::bindings::{
    exports::test::wit_test::http::{Guest, Method, StatusCode},
    test::wit_test::io::print,
};

struct Component;

impl Guest for Component {
    fn request(method: Method, url: String, body: String) -> (StatusCode, Option<String>) {
        print(&format!("{:?} {} {}", method, url, body));
        (200, Some("Hello, world!".to_string()))
    }
}
