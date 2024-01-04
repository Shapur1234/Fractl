# Fractl

- Fractal renderer written in rust
- Singlethreaded, multithreading (using [rayon](https://github.com/rayon-rs/rayon)) and gpu compute (using [wgpu](https://github.com/gfx-rs/wgpu) - [WebGpu](https://developer.mozilla.org/en-US/docs/Web/API/WebGPU_API) implementation) versions
- Supports both native targets and [wasm](https://en.wikipedia.org/wiki/WebAssembly)
- Currently renders the [Mandelbrot set](https://en.wikipedia.org/wiki/Mandelbrot_set) and the [Multibrot set](https://en.wikipedia.org/wiki/Multibrot_set)

## Screenshots

![Mandelbrot](/screenshot/mandelbrot.png)
![Multibrot](/screenshot/multibrot.png)

## How to

- Download a binary from releases (TODO)
- Install using cargo (TODO)

### Building using nix

- (This should be the most reliable way)
- Have [nix](https://nixos.org/) [installed](https://nix.dev/install-nix.html) on your system
- Pick a package

| Package             | Description                               | Pararelism         | Floatin point percision |
| ------------------- | ----------------------------------------- | ------------------ | ----------------------- |
| gui                 | Native gui                                | No                 | Double (f64)            |
| gui-multithread     | Native gui                                | CPU multithreading | Double (f64)            |
| gui-gpu             | Native gui                                | GPU compute shader | Single (f32)            |
| gui-wasm            | Web gui                                   | No                 | Double (f64)            |
| gui-win             | Native gui crosscompiled to Winows x86_64 | No                 | Double (f64)            |
| gui-win-multithread | Native gui crosscompiled to Winows x86_64 | CPU multithreading | Double (f64)            |
| gui-win-gpu         | Native gui crosscompiled to Winows x86_64 | GPU compute shader | Single (f32)            |

To build: `nix build .#Package`

To run locally: `nix build .#Package`

### Building using cargo

### Controls

| Key             | Action                                  |
| --------------- | --------------------------------------- |
| LeftMouseButton | Center view on cursor                   |
| WSAD            | Move view                               |
| R               | Reset view                              |
| ScrollWheel     | Increase / Decrease zoom                |
| O               | Increase zoom                           |
| P               | Decrease zoom                           |
| ArrowKey        | Increase / Decrease zoom in a direction |
| T               | Reset zoom                              |
| K               | Increase percision (max iterations)     |
| L               | Decrease percision (max iterations)     |
| M               | Next fractal                            |
| N               | Previous fractal                        |
| B               | Next fractal coloring                   |
| V               | Previous fractal coloring               |
| U               | Toggle UI                               |
| Y               | Toggle crosshair                        |
| C               | Increase exponent (for multi-fractals)  |
| X               | Decrease exponent (for multi-fractals)  |
| F11             | Toggle Fullscreen                       |
| Escape          | Exit                                    |

## TODO

- Finish the readme :)
- Add Julia set, more fractals
- Wasm WebGpu with compute shader
- Redox portk
- Publish to crates.io
