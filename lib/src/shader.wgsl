struct Args {

    screen_size: vec2<u32>,
    view_size: vec2<f32>,
    zoom: vec2<f32>,
    center_pos: vec2<f32>,
    max_iterations: u32,
    selected_fractal: u32,
    selected_color: u32,
    _padding: u32,
}

@group(0) @binding(0)
var<storage, read_write> v_indices: array<u32>; 

@group(0) @binding(1)
var<uniform> args: Args;

fn index_to_world_pos(index: u32) -> vec2<f32> {
    let screen_x = index % args.screen_size.x;
    let screen_y = (index - screen_x) / args.screen_size.x;
    
    let screen_pos_normalized = vec2(
        (f32(screen_x) / f32(args.screen_size.x)) - 0.5, 
        (f32(screen_y) / f32(args.screen_size.y)) - 0.5
    );

    return vec2(
        ((screen_pos_normalized.x * args.view_size.x) / args.zoom.x) + args.center_pos.x,
        ((screen_pos_normalized.y * args.view_size.y) / args.zoom.y) + args.center_pos.y,
    );
}

fn mandelbrot_escape_time(world_pos: vec2<f32>) -> u32 {
    // https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set#Optimized_escape_time_algorithms

    var n: u32 = 0u;

    var x: f32 = 0.0;
    var x2: f32 = 0.0;
    var y: f32 = 0.0;
    var y2: f32 = 0.0;

    let q = pow(world_pos.x - 0.25, 2.0) + pow(world_pos.y, 2.0);
    let is_in_main_bulb = q * (q + world_pos.x - 0.25) <= 0.25 * pow(world_pos.y, 2.0);

    if is_in_main_bulb {
        return args.max_iterations;
    } else {
        loop  {
            if !(x2 + y2 <= 4.0 && n < args.max_iterations) {
                break;
            }

            y = 2.0 * x * y + world_pos.y;
            x = x2 - y2 + world_pos.x;

            x2 = pow(x, 2.0);
            y2 = pow(y, 2.0);

            n += 1u;
        }
    }

    return n;
}

fn color_histogram(escape_time: u32) -> u32 {
    // https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set#Histogram_coloring

    return color(0u, 0u, u32((f32(escape_time) / f32(args.max_iterations)) * 255.0));
}

fn color_lch(escape_time: u32) -> u32 {
    // https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set#LCH_coloring

    let pi = 3.14159;

    let s = f32(escape_time) / f32(args.max_iterations);
    let v = pow(cos(1.0 - (pi * s)), 2.0);

    return color(
        u32(75.0 - (75.0 * v)),
        u32(28.0 + (75.0 - (75.0 * v))),
        u32(pow(360.0 * s, 1.5) % 360.0),
    );
}

fn color_olc(escape_time: u32) -> u32 {
    // https://github.com/OneLoneCoder/Javidx9/blob/54b26051d0fd1491c325ae09f50a7fc3f25030e8/PixelGameEngine/SmallerProjects/OneLoneCoder_PGE_Mandelbrot.cpp#L543C3-L543C3

    let a = 0.1; 
    let n = f32(escape_time);
    
    return color(
        u32(0.5 * (sin((a * n)) + 0.5) * 255.0),
        u32(0.5 * (sin((a * n + 2.094)) + 0.5) * 255.0),
        u32(0.5 * (sin((a * n + 4.188)) + 0.5) * 255.0),
    );
}

fn color(red: u32, green: u32, blue: u32) -> u32 {
    return blue | (green << 8u) | (red << 16u);
}

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = v_indices[global_id.x];
    let world_pos = index_to_world_pos(index);

    var escape_time: u32 = 0u;
    switch args.selected_fractal {
        case 0u: {
            escape_time = mandelbrot_escape_time(world_pos);
        }
        default: {
            escape_time = u32(-1);
        }
    }

    var color: u32 = 0u;
    switch args.selected_color {
        case 0u: {
            color = color_histogram(escape_time);
        }
        case 1u: {
            color = color_lch(escape_time);
        }
        case 2u: {
            color = color_olc(escape_time);
        }
        default: { 
            color = color(255u, 0u, 0u);
        }
    }
    
    v_indices[global_id.x] = color;
}
