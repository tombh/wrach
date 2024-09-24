// Just draws particles as simple pixels

#import types::WorldSettings;

@group(0) @binding(0) var<uniform> settings: WorldSettings;
@group(0) @binding(1) var<storage, read_write> positions: array<vec2<f32>>;

struct VertexInput {
    @builtin(vertex_index) index: u32,
    @builtin(instance_index) instance: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vertex(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    var local_position: vec2<f32>;
    var pixel_size = 0.4;
    // The GPU view target is in the range: `[-1.0, -1.0, 0.0, 0.0]`. So here we scale the viewport
    // coordinates to that.
    var factor: vec2<f32> = 1.0 / (settings.view_dimensions / 2.0);

    let index = square_indices[input.index];
    local_position = square_vertices[index] * factor * pixel_size;
    let particle_position = (positions[input.instance] * factor) - 1.0;
    let view_position = vec4<f32>(particle_position + local_position, 0.0, 1.0);

    out.position = view_position;
    out.color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    return out;
}

@fragment
fn fragment(@location(0) color: vec4<f32>) -> @location(0) vec4<f32> {
    return color;
}

var<private> square_vertices: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
    vec2<f32>(-1, -1),
    vec2<f32>(1, -1),
    vec2<f32>(-1, 1),
    vec2<f32>(1, 1),
);

var<private> square_indices: array<u32, 6> = array<u32, 6>(
    0, 1, 2,
    1, 3, 2
);

