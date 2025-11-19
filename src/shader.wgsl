struct VertexInput {
    @location(0) pos: vec3f,
    @location(1) color: vec3f
};

@vertex
fn vs_main(@builtin(vertex_index) vtx_idx: u32) -> @builtin(position) vec4f {
    const vertex = array(
        vec2(-0.8, 0.8),
        vec2(0.8, 0.6),
        vec2(0.0, -0.8),
    );

    return vec4f(vertex[vtx_idx], 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4f {
    return vec4f(0.1, 0.1, 0.5, 1.0);
}