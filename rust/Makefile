
TARGET = wasm32-unknown-unknown

all: src/lib.rs
	RUSTFLAGS='-C target-feature=-crt-static' cargo build --target $(TARGET)

seeimport: src/lib.rs
	wasm-objdump target/$(TARGET)/debug/example_tokio_wasm.wasm -j import -x

run: src/lib.rs
	 wasmtime --invoke main target/$(TARGET)/debug/example_tokio_wasm.wasm