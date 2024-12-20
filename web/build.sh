cd ../dsp
cargo build --target wasm32-unknown-unknown --release
wasm-gc target/wasm32-unknown-unknown/release/voxdsp.wasm ../web/dsp.wasm
