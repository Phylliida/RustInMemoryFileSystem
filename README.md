
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

In progress