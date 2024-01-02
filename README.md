# Fractl

- Fractal renderer written in rust
- Singlethreaded, multithreading (using [rayon](https://github.com/rayon-rs/rayon)) and gpu compute (using [wgpu](https://github.com/gfx-rs/wgpu) - [WebGpu](https://developer.mozilla.org/en-US/docs/Web/API/WebGPU_API) implementation) versions
- Supports both native targets and [wasm](https://en.wikipedia.org/wiki/WebAssembly)
- Currently renders the [Mandelbrot set](https://en.wikipedia.org/wiki/Mandelbrot_set) and the [Multibrot set](https://en.wikipedia.org/wiki/Multibrot_set) 

## Screenshots

![Mandelbrot](/screenshot/mandelbrot.png)
![Multibrot](/screenshot/multibrot.png)

## How to

### Installation

### Controls

## TODO

- Add Julia set, more fractals
- Wasm WebGpu with compute shader
- Redox port
- Publish to crates.io
