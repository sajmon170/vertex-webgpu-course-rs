# Vertex WebGPU course - lesson #2
In the previous lesson we've initialized our environment and rendered a simple
triangle. However, that triangle was hardcoded inside the vertex shader. This
isn't feasible for larger projects. Luckily, graphics APIs provide a simple
solution to this problem - vertex and index buffers.

## Vertex Buffers
Vertex buffers are buffers that:
- can be written to by the CPU
- can be accessed by the GPU
- store vertex data.

Note that the vertex data is defined by the user. It isn't restricted to only
vertex positions - we can also pass in colors, UV coordinates etc. In this
lesson we're going to pass vertex positions and colors.

We need to define the vertex buffer structure in three places
1. The Rust codebase (to use them in our CPU-side code)
2. The `PipelineDescriptor` (to describe the low-level representation of our
struct so that we can send it to the GPU)
2. The shader code (to use them in our GPU-side code)

## The indexed drawing mode
We know how to render a single triangle - we provide a list of every vertex that
defines it. This is fine if it's the only thing we're rendering. However, this
quickly becomes extremely inefficient when we render complex meshes simply
because **the triangles are connected and therefore share vertices**.

In order to fix this, we'll use index buffers. They define triplets of indices
to the vertex buffer. This bypasses the need to clone vertices in order to
define new triangles - we just index the necessary positions instead.
