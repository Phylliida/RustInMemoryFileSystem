rustup target add wasm32-wasip1
 wasm-objdump target/wasm32-wasip1/debug/tokio-wasi.wasm -j import -x
rustup install nightly 
 rustup target add wasm32-wasip1 --toolchain nightly
