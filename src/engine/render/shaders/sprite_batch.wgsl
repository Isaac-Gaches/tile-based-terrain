struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coord: vec2<f32>,
};

struct InstanceInput {
    @location(2) position: vec2<f32>,
    @location(3) rotation: f32,
    @location(4) scale: f32,
    @location(5) tex_index: u32,
    @location(6) colour: vec4<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) light_tex_coord: vec2<f32>,
    @location(1) colour: vec4<f32>,
    @location(2) atlas_tex_coord: vec2<f32>,
};

struct Camera{
    position: vec2<f32>,
    zoom: f32,
    ratio: f32,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(0) @binding(1)
var atlas_texture: texture_2d<f32>;
@group(0) @binding(2)
var atlas_sampler: sampler;

struct LightMeta{
    position: vec2<f32>,
    render_distance: f32,
    chunk_size: f32,
}

@group(0) @binding(3)
var light_texture: texture_2d<f32>;
@group(0) @binding(4)
var light_sampler: sampler;
@group(0) @binding(5)
var<uniform> light_meta: LightMeta;

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput
) -> VertexOutput {
   var out: VertexOutput;
   out.light_tex_coord = vec2<f32>(
       (vertex.position.x + 0.5 +instance.position.x + light_meta.render_distance - light_meta.position.x)/(light_meta.render_distance*2.+light_meta.chunk_size),
       1.0-(vertex.position.y * instance.scale +instance.position.y- 0.5 + light_meta.render_distance - light_meta.position.y)/(light_meta.render_distance*2.+light_meta.chunk_size)
   );
   out.atlas_tex_coord = vertex.tex_coord;
   out.clip_position = vec4<f32>((vertex.position * instance.scale + instance.position - camera.position) * vec2<f32>(camera.zoom) * vec2<f32>(camera.ratio,1.0),0.0,1.0);
   out.colour = instance.colour;
   return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex = textureSample(atlas_texture, atlas_sampler, in.atlas_tex_coord);
    if tex.a== 0.{discard;}
    return vec4<f32>(in.colour) * textureSample(light_texture, light_sampler, in.light_tex_coord) * tex;
}
