
TARGET = wasm32-unknown-unknown

all: src/lib.rs
	RUSTFLAGS='-C target-feature=-crt-static' cargo build --target $(TARGET) --release
	cp target/wasm32-unknown-unknown/release/in_memory_wasm_filesystem.wasm .

inspect: src/lib.rs
	wasmer inspect target/wasm32-unknown-unknown/release/in_memory_wasm_filesystem.wasm

run: src/lib.rs
	 wasmtime --invoke main target/$(TARGET)/release/in_memory_wasm_filesystem.wasm
