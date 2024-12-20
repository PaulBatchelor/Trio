# Trio
A singing synthesizer musical instrument with algorithmic
3-part harmony. Written using WebAudio, Canvas, WebAssembly,
and Rust.

A demo can be found [here](https://pbat.ch/recurse/demos/trio).

## Building

Clone this repo, and then the [VoxBox](https://github.com/PaulBatchelor/VoxBox)
repo right next to it. Go into this folder and run `sh build.sh`.
Make sure you have the `wasm32-unknown-unkown` target installed,
as well as `wasm-gc`.

If all goes well, a file called "dsp.wasm" should appear,
and you should be able to serve a local instance
of the instrument by running `python3 -m http.server` and
opening `localhost:8000` in a browser.
