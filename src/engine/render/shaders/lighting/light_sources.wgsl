struct LightSource {
    pos: vec2<f32>,
    _pad0: vec2<f32>,
    colour: vec4<f32>,
}

@group(0) @binding(0)
var outputTex: texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(1)
var<storage, read> lights: array<LightSource>;

struct Meta {
    pos: vec2<f32>,
    light_count: u32,
    mid: vec2<i32>,
}

@group(0) @binding(2)
var<uniform> light_meta: Meta;

@compute @workgroup_size(64,1,1)
fn set_light_sources(
    @builtin(global_invocation_id) gid : vec3<u32>
) {
    let index = gid.x;
    if (index >= light_meta.light_count) {
        return;
    }
    let light = lights[index];

    let size = textureDimensions(outputTex);

    var px = i32(light.pos.x - light_meta.pos.x);
    var py = i32(light.pos.y - light_meta.pos.y);

    px += light_meta.mid.x;
    py += light_meta.mid.y;
    py = i32(size.y) - py;

    if (px < 0 || py < 0 || px >= i32(size.x) || py >= i32(size.y)) {
        return;
    }

    textureStore(
        outputTex,
        vec2<i32>(px, py),
        light.colour
    );
}