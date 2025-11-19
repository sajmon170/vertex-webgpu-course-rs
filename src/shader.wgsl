struct VertexInput {
    @location(0) pos: vec3f,
    @location(1) color: vec3f
};

struct VertexOutput {
    @builtin(position) pos: vec4f,
    @location(0) color: vec3f
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let ratio = 640.0/480.0;
    return VertexOutput(vec4f(in.pos.x/ratio, in.pos.y, in.pos.z, 1.0), in.color);
}

@fragment
fn fs_main() -> @location(0) vec4f {
    return vec4f(0.1, 0.1, 0.5, 1.0);
}