# Rust Discord Minesweeper

## Dependencies

You need to have `wasmtime` installed on your system
*and rust obviously*

## Run

Run with `cargo run --release`

*You need to have a `token.txt` and a `application_id.txt` in order to compile the program*

## For python script

Download https://github.com/singlestore-labs/python-wasi/releases
And put in a "python" folder :
- python3.10.wasm
- lib/ (with the python3.10 folder)

# Compiling to wasm

## Rust

Just add the target with `rustup target add wasm32/wasi`
And then you can compile with `cargo build --target=wasm32/wasi`
