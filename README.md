# couple-crows
Simulating the boids behaviour of a couple crows in the browser using WebAssembly, webGPU


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
```
