struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct InstanceInput {
    @location(1) position: vec3<f32>,
    @location(2) rotation: f32,
    @location(3) scale: f32,
    @location(4) tex_index: u32,
    @location(5) colour: vec4<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) sprite_uv: vec2<f32>,
};

@group(0) @binding(0)
var sprite_texture: texture_2d<f32>;
@group(0) @binding(1)
var sprite_sampler: sampler;

struct AtlasFrame{
    min_uv: vec2<f32>,
    max_uv: vec2<f32>,
}

@group(0) @binding(2)
var<storage,read> atlas: array<AtlasFrame>;

struct Camera{
    aspect: f32,
}

@group(0) @binding(3)
var<uniform> camera: Camera;

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput
) -> VertexOutput {
    var out: VertexOutput;

    let local_pos = vertex.position.xy * instance.scale;

    let c = cos(instance.rotation);
    let s = sin(instance.rotation);

    let rotated_pos = vec2<f32>(
        local_pos.x * c - local_pos.y * s,
        local_pos.x * s + local_pos.y * c
    );
    
    var world_pos = instance.position.xy + rotated_pos;
    world_pos.y = 1.0 + (world_pos.y - 1.0) * camera.aspect;
    out.clip_position = vec4<f32>(vec3<f32>(world_pos, instance.position.z), 1.0);

    let frame = atlas[instance.tex_index];
    let quad_uv = vertex.position.xy * vec2<f32>(1.0, -1.0) + vec2<f32>(0.5, 0.5);
    out.sprite_uv = frame.min_uv + quad_uv * (frame.max_uv - frame.min_uv);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex = textureSample(sprite_texture, sprite_sampler, in.sprite_uv);
    if tex.a== 0.{discard;}
    return tex;
}