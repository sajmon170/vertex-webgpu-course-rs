struct VertexInput {
    @location(0) pos: vec3f,
    @location(1) color: vec3f
};

@vertex
fn vs_main(in: VertexInput) -> @builtin(position) vec4f {
    return vec4f(in.pos, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4f {
    return vec4f(0.1, 0.1, 0.5, 1.0);
}