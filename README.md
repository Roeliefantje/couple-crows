# couple-crows
Simulating the boids behaviour of a "couple" (100k+) crows in the browser using WebAssembly, webGPU.

## How to run natively
- Ensure rust is installed
- Clone the repository
- Run the repository
```
cargo run
```

# How to run on the web:
- Ensure rust is installed
- Ensure the rust installation has WASM support:
```
rustup target install wasm32-unknown-unknown
```
- Install wasm server runner
```
cargo install wasm-server-runner
```
- Clone the repository
- Compile and run the repository:
```
cargo run --target wasm32-unknown-unknown
```


## Notes:
The web application has the tendency to randomly crash when the program is starting to execute.
Still am not entirely sure why but refreshing the page multiple times until it works works for me.
If there are a lot of wgpu related compile issues the env variable might not be setup correctly:
```
RUSTFLAGS=--cfg=web_sys_unstable_apis
```

<!-- 
How to set up:
https://bevyengine.org/learn/book/getting-started/setup/#enable-fast-compiles-optional


Windows installation:

- Install the 64-bit.exe: https://www.rust-lang.org/learn/get-started
- Reload terminal
- Windows: Ensure you have the latest cargo-binutils as this lets commands like `cargo run` use the LLD linker automatically:
```
cargo install -f cargo-binutils
rustup component add llvm-tools-preview
```
- Enable Bevy's Dynamic Linking Feature with the flag` --features bevy/dynamic_linking`

To run the [main.rs](src/main.rs) use: 
```
cargo run --features bevy/dynamic_linking
``` -->
