
TARGET = wasm32-unknown-unknown

all: src/lib.rs
	RUSTFLAGS='-C target-feature=-crt-static' cargo build --target $(TARGET)

inspect: src/lib.rs
	wasmer inspect target/wasm32-unknown-unknown/debug/example_tokio_wasm.wasm

run: src/lib.rs
	 wasmtime --invoke main target/$(TARGET)/debug/example_tokio_wasm.wasm
