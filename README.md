
UNDER CONSTRUCTION

# What is it?

This is a ~200KB "polyfill" that hosts an file system in memory, in pure wasm.

It is developed around a rust port I made of the [v86 js emulator](https://github.com/copy/v86/blob/master/lib/filesystem.js)'s file system.

All wasm file system WASI commands can be forwarded to this file system,
allowing you to run web assembly programs that use the file system without giving
it access to the file system.

Symlinks are supported!

Solutions like this exist for javascript, but this is pure rust -> pure wasm,
which is more suitable for use cases outside the browser.

# What WASI file system commands are supported?

Every `fd_` and `path_`. See [The wasi reference](https://wasix.org/docs/api-reference)
or [the code](https://github.com/Phylliida/v86FileSystemPort/blob/main/rust/src/wasi.rs) for more information

# To Compile

```
rustup install nightly 
rustup target add wasm32-unknown-unknown --toolchain nightly
cd rust
make
```

# What about stdin/stdout/stderr?

In progressff

# How do I use this in my rust code?

Just use regular file system commands like you would in any other rust program, and then target wasm32-wasip1. That'll export any WASI file system commands you need, and you can connect them up to this wasm.

# Alternatives

I wasn't able to find any suitable alternatives online.

- Emscripten's WASMFS seemed a sensible choice, so I [previously wrote a port for it](https://github.com/Phylliida/EmscriptenWasmFSWrapper). However I couldn't get it working well, and it's missing quite a few features
- [https://github.com/WebAssembly/wasi-filesystem](https://github.com/WebAssembly/wasi-filesystem) is read-only
- [https://github.com/wasm-forge/ic-wasi-polyfill](https://github.com/wasm-forge/ic-wasi-polyfill) is a wrapper for the internet computer (some crypto aws alternative), and isn't a standalone wasm

The following are rust, in memory file system alternatives that may be worth writing WASI wrappers for (but don't currently have WASI wrappers for them)
- [https://github.com/gz/btfs](https://github.com/gz/btfs)
- [https://github.com/GodTamIt/btrfs-diskformat](https://github.com/GodTamIt/btrfs-diskformat)
- [https://github.com/twmb/rsfs](https://github.com/twmb/rsfs)
- [https://github.com/andrewhalle/memfs](https://github.com/andrewhalle/memfs)

# Further reading

Other in memory file system implementations

- [https://crates.io/crates/nine-memfs](https://crates.io/crates/nine-memfs) says is buggy :/
- [https://github.com/golemfactory/sp-wasm/tree/master/sp-wasm-memfs](https://github.com/golemfactory/sp-wasm/tree/master/sp-wasm-memfs)
- [https://github.com/phR0ze/rivia-vfs](https://github.com/phR0ze/rivia-vfs) has a memory file system
- [https://github.com/manuel-woelker/rust-vfs](https://github.com/manuel-woelker/rust-vfs), though it says MemoryFS is intended mainly for unit tests
- [https://github.com/cloudflare/workers-wasi](https://github.com/cloudflare/workers-wasi) soft and hard links not yet supported