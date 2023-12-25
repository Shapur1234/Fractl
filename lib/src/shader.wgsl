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
    var n: u32 = 0u;

    var x: f32 = 0.0;
    var x2: f32 = 0.0;
    var y: f32 = 0.0;
    var y2: f32 = 0.0;

    loop  {
        if !(x2 + y2 <= 4.0 && n < args.max_iterations) {
            break;
        }

        y = 2.0 * x * y + world_pos.y;
        x = x2 - y2 + world_pos.x;

        x2 = pow(x, 2.0);
        y2 = pow(y, 2.0);

        n = n + 1u;
    }

    return n;
}

fn color_histogram(escape_time: u32) -> u32 {
    return color(0u, 0u, u32(f32(escape_time) / f32(args.max_iterations)) * 255u);
}

fn color(red: u32, green: u32, blue: u32) -> u32 {
    return blue | (green << 8u) | (red << 16u);
}

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // TODO: Different colors and fractals
    v_indices[global_id.x] = color_histogram(mandelbrot_escape_time(index_to_world_pos(v_indices[global_id.x])));
}
