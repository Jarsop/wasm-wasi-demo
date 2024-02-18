use anyhow::Context;
use std::{fs, path::Path};
use wit_component;

use wasmtime::{
    component::{bindgen, Component, Linker},
    Config, Engine, Result, Store,
};

// Generate bindings of the guest and host components.
bindgen!("publish" in "wit/publish.wit");

struct HostComponent;

impl host::Host for HostComponent {
    fn fetch(
        &mut self,
        method: host::Method,
        url: String,
        payload: Option<String>,
    ) -> Result<(host::StatusCode, Option<String>)> {
        let body = format!("{:?} {} {:?}", method, url, payload);
        Ok((200, Some(body)))
    }
}

struct MyState {
    host: HostComponent,
}

fn convert_to_component(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    let bytes = &fs::read(&path).context("failed to read input file")?;
    wit_component::ComponentEncoder::default()
        .module(&bytes)?
        .encode()
}

fn main() -> Result<()> {
    // Create an engine with the component model enabled (disabled by default).
    let engine = Engine::new(Config::new().wasm_component_model(true))?;

    // NOTE: The wasm32-unknown-unknown target is used here for simplicity, real world use cases
    // should probably use the wasm32-wasi target, and enable wasi preview2 within the component
    // model.
    let component =
        convert_to_component("target/wasm32-unknown-unknown/debug/azure_publisher.wasm")?;

    // Create our component and call our generated host function.
    let component = Component::from_binary(&engine, &component)?;
    let mut store = Store::new(
        &engine,
        MyState {
            host: HostComponent {},
        },
    );
    let mut linker = Linker::new(&engine);
    host::add_to_linker(&mut linker, |state: &mut MyState| &mut state.host)?;
    let (convert, _instance) = Publish::instantiate(&mut store, &component, &linker)?;
    let result = convert.call_publish(&mut store, "Hello world")?;
    println!("Result from WASM: {:?}", result);
    Ok(())
}
