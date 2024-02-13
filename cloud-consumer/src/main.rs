use std::path::PathBuf;

use anyhow::{anyhow, Result};
use reqwest::{Client, Method, Request, Response, Url};
use wasmtime::*;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};

#[derive(Clone)]
struct CloudProvider {
    module: PathBuf,
    token: String,
}

impl CloudProvider {
    fn new_aws() -> Self {
        CloudProvider {
            #[cfg(debug_assertions)]
            module: PathBuf::from("target/wasm32-wasi/debug/aws_cloud_provider.wasm"),
            #[cfg(not(debug_assertions))]
            module: PathBuf::from("target/wasm32-wasi/release/aws_cloud_provider.wasm"),
            token: "AWS_TOKEN".to_owned(),
        }
    }

    fn new_azure() -> Self {
        CloudProvider {
            #[cfg(debug_assertions)]
            module: PathBuf::from("target/wasm32-wasi/debug/azure_cloud_provider.wasm"),
            #[cfg(not(debug_assertions))]
            module: PathBuf::from("target/wasm32-wasi/release/azure_cloud_provider.wasm"),
            token: "AZURE_TOKEN".to_owned(),
        }
    }
}

struct State {
    wasi: WasiCtx,
    response: Option<Response>,
}

struct CloudInstance {
    store: Store<State>,
    memory: Memory,
    instance: Instance,
}

impl CloudInstance {
    async fn copy_memory(&mut self, bytes: &[u8]) -> Result<isize> {
        let alloc = self
            .instance
            .get_typed_func::<i32, i32>(&mut self.store, "alloc")?;
        let guest_ptr_offset = alloc
            .call_async(&mut self.store, bytes.len() as i32)
            .await?;

        self.memory
            .write(&mut self.store, guest_ptr_offset as usize, bytes)?;

        Ok(guest_ptr_offset as isize)
    }

    async fn deallocate_memory(&mut self, ptr: isize, len: usize) -> Result<()> {
        let dealloc = self
            .instance
            .get_typed_func::<(i32, i32), ()>(&mut self.store, "dealloc")?;
        dealloc
            .call_async(&mut self.store, (ptr as i32, len as i32))
            .await?;
        Ok(())
    }

    async fn publish(&mut self, payload: &str) -> Result<String> {
        let guest_ptr_offset = self.copy_memory(payload.as_bytes()).await?;

        let publish = self
            .instance
            .get_typed_func::<(i32, i32), i32>(&mut self.store, "publish")?;
        let ret = publish
            .call_async(
                &mut self.store,
                (guest_ptr_offset as i32, payload.len() as i32),
            )
            .await?;

        self.deallocate_memory(guest_ptr_offset as isize, payload.len())
            .await?;

        if ret != 0 {
            Err(anyhow!("failed to publish"))
        } else {
            let response = self
                .store
                .data_mut()
                .response
                .take()
                .unwrap()
                .text()
                .await?;
            Ok(response)
        }
    }
}

#[derive(Clone)]
struct Environment {
    engine: Engine,
    module: Module,
    linker: Linker<State>,
    cloud_provider: CloudProvider,
}

impl Environment {
    pub fn new(cloud_provider: CloudProvider) -> Result<Self, Error> {
        let mut config = Config::new();
        config.async_support(true);
        let engine = Engine::new(&config)?;
        let module = Module::from_file(&engine, &cloud_provider.module)?;

        let linker = Linker::new(&engine);

        Ok(Self {
            engine,
            module,
            linker,
            cloud_provider,
        })
    }
}

macro_rules! get_data {
    ($mem:ident, $caller:ident, $ptr:ident, $len:ident) => {{
        let data = $mem
            .data(&$caller)
            .get($ptr as u32 as usize..)
            .and_then(|arr| arr.get(..$len as u32 as usize));
        let string = match data {
            Some(data) => match std::str::from_utf8(data) {
                Ok(s) => s,
                Err(_) => return Err(anyhow!("invalid utf-8")),
            },
            None => return Err(anyhow!("pointer/length out of bounds")),
        };
        string
    }};
}

async fn fetch(
    mut caller: Caller<'_, State>,
    r#type: i32,
    url_ptr: i32,
    url_len: i32,
    body_ptr: i32,
    body_len: i32,
) -> Result<i32> {
    let mem = match caller.get_export("memory") {
        Some(Extern::Memory(mem)) => mem,
        _ => return Err(anyhow!("failed to find host memory")),
    };
    let url = get_data!(mem, caller, url_ptr, url_len);
    let method = match r#type {
        0 => Method::GET,
        1 => Method::POST,
        2 => Method::PUT,
        3 => Method::DELETE,
        _ => return Err(anyhow!("invalid type")),
    };
    let body = if Method::GET == method {
        None
    } else {
        Some(
            get_data!(mem, caller, body_ptr, body_len)
                .to_string()
                .into(),
        )
    };

    let url = Url::parse(url)?;

    let mut request = Request::new(method, url);
    *request.body_mut() = body;
    let response = Client::new().execute(request).await?;

    caller.data_mut().response = Some(response);

    Ok(0)
}

async fn create_instance(mut env: Environment) -> Result<CloudInstance> {
    let ctx = WasiCtxBuilder::new()
        .inherit_stdin()
        .inherit_stdout()
        .inherit_stderr()
        .env("TOKEN", &env.cloud_provider.token)?
        .build();
    let state = State {
        wasi: ctx,
        response: None,
    };
    let mut store = Store::new(&env.engine, state);
    wasmtime_wasi::add_to_linker(&mut env.linker, |s| &mut s.wasi)?;

    let fetch = Func::wrap5_async(
        &mut store,
        |caller: Caller<'_, State>,
         r#type: i32,
         url_ptr: i32,
         url_len: i32,
         body_ptr: i32,
         body_len: i32| {
            Box::new(
                async move { fetch(caller, r#type, url_ptr, url_len, body_ptr, body_len).await },
            )
        },
    );
    env.linker.define(&mut store, "env", "fetch", fetch)?;
    let instance = env
        .linker
        .instantiate_async(&mut store, &env.module)
        .await?;

    let memory = instance
        .get_memory(&mut store, "memory")
        .ok_or(anyhow!("failed to find `memory` export"))?;

    Ok(CloudInstance {
        store,
        memory,
        instance,
    })
}

async fn publish_wasm(env: Environment) -> Result<()> {
    let mut cloud_instance = create_instance(env).await?;
    let published = cloud_instance.publish("Publish 42").await?;

    println!("HOST: Published: {}", published);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let aws_env = Environment::new(CloudProvider::new_aws())?;
    let azure_env = Environment::new(CloudProvider::new_azure())?;
    println!("HOST: Publishing from AWS environment");
    publish_wasm(aws_env).await?;
    println!("HOST: Publishing from Azure environment");
    publish_wasm(azure_env).await?;
    Ok(())
}
