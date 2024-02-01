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

The amount of crows part of the application can be changed in shared.rs

In assets/instancing.wgsl is a commented line (67) which allows you to change the color of the crows based on the velocity of the crow.

If you want to limit the framerate based on the execution time of the boids algorithm, you can use line 366 instead of 364.
(This will not compile to web)

