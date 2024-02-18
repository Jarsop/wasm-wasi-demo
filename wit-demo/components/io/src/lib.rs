mod bindings;

use crate::bindings::exports::test::wit_test::io::Guest;

struct Component;

impl Guest for Component {
    fn print(msg: String) {
        println!("{}", msg);
    }
}
