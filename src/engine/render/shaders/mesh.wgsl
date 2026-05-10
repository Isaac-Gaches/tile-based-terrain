struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tile_tex_coord: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) light_tex_coord: vec2<f32>,
    @location(1) tile_tex_coord: vec2<f32>,
};

struct Camera{
    position: vec2<f32>,
    zoom: f32,
    ratio: f32,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(0) @binding(1)
var tile_texture: texture_2d<f32>;
@group(0) @binding(2)
var tile_sampler: sampler;

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

@group(0) @binding(6)
var occlusion_texture: texture_2d<f32>;

@vertex
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.light_tex_coord = vec2<f32>(
        (vertex.position.x + 0.5 + light_meta.render_distance - light_meta.position.x)/(light_meta.render_distance*2.+light_meta.chunk_size),
        1.0-(vertex.position.y - 0.5 + light_meta.render_distance - light_meta.position.y)/(light_meta.render_distance*2.+light_meta.chunk_size)
    );
    out.tile_tex_coord = vertex.tile_tex_coord;
    out.clip_position = vec4<f32>((vertex.position- vec3<f32>(camera.position,0.)) * vec3<f32>(camera.zoom,camera.zoom,1.0) * vec3<f32>(1.0,camera.ratio,1.0),1.0);
    return out;
}

@fragment
fn fs_fg_tiles(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex = textureSample(tile_texture, tile_sampler, in.tile_tex_coord);
    if tex.a== 0.{discard;}
    let light = textureSample(light_texture, light_sampler, in.light_tex_coord) ;
    return tex * light;
}

@fragment
fn fs_bg_tiles(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex = textureSample(tile_texture, tile_sampler, in.tile_tex_coord);
    if tex.a== 0.{discard;}
    let light = textureSample(light_texture, light_sampler, in.light_tex_coord) * 0.3 * max(textureSample(occlusion_texture, light_sampler, in.light_tex_coord).r,0.5);
    return tex * light;
}