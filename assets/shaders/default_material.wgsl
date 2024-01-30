#import bevy_pbr::mesh_functions::{get_model_matrix, mesh_position_local_to_clip}


struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> @builtin(position) vec4<f32> {
    return mesh_position_local_to_clip(
        get_model_matrix(vertex.instance_index),
        vec4<f32>(vertex.position, 1.0),
    );
}


@group(1) @binding(0) var<uniform> color: vec4<f32>;

@fragment
fn fragment() -> @location(0) vec4<f32> {
    return color;
}
