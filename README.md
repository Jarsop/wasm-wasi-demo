# WASM Rust/C demo

## Build

Chill:

```bash
make
# Debug mode
DEBUG=1 make
```

**NOTE**: This install the WASI SDK automatically

## Run

One terminal run the server:

```bash
./target/release/echo-server
```

Another run the cloud-consumer:

```bash
./target/release/cloud-consumer
```

**NOTE**: If you compiled in `DEBUG` mode replace `release` by `debug`

**That's all**

## Clean

```bash
make clean
# Clean SDK also
make clean-all
```
