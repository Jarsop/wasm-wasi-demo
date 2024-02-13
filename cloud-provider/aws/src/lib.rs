extern "C" {
    pub fn fetch(
        r#type: Method,
        url_ptr: *mut u8,
        url_len: i32,
        body_ptr: *mut u8,
        body_len: i32,
    ) -> i32;
}

#[repr(C)]
pub enum Method {
    GET = 0,
    POST = 1,
    PUT = 2,
    DELETE = 3,
}

#[no_mangle]
pub fn alloc(len: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(len);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    return ptr;
}

#[no_mangle]
pub unsafe fn dealloc(ptr: *mut u8, size: usize) {
    let data = Vec::from_raw_parts(ptr, size, size);
    std::mem::drop(data);
}

#[no_mangle]
pub unsafe fn publish(payload: *mut u8, len: usize) -> i32 {
    let method = Method::POST;

    let data = Vec::from_raw_parts(payload, len, len);
    let mut body = String::from_utf8(data).unwrap();
    let payload = body.clone();
    body.push_str(" from Rust WASM [AWS]");

    let mut url = "http://127.0.0.1:3000/echo".to_string();

    let body_ptr = body.as_mut_ptr();
    let body_len = body.len() as i32;

    let url_ptr = url.as_mut_ptr();
    let url_len = url.len() as i32;

    println!(
        "WASM: fetch: {}",
        fetch(method, url_ptr, url_len, body_ptr, body_len)
    );
    println!("WASM: payload: {}", payload);
    println!("WASM: TOKEN: {}", std::env::var("TOKEN").unwrap());

    0
}
