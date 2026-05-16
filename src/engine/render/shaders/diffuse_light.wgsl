@group(0) @binding(0)
var inputTex: texture_2d<f32>;
@group(0) @binding(1)
var outputTex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2)
var tiles: texture_2d<u32>;

@compute @workgroup_size(16,16,1)
fn diffuse_light(@builtin(global_invocation_id) gid : vec3<u32>){
    let size = textureDimensions(inputTex);
    if (gid.x >= size.x || gid.y >= size.y) {
        return;
    }

    let tile = textureLoad(tiles,vec2<u32>(gid.x,size.y-gid.y),0).r;
    let current = textureLoad(inputTex, vec2<u32>(gid.x,gid.y), 0).rgb;

    if tile == 1{
        textureStore(outputTex, vec2<u32>(gid.x, gid.y), vec4<f32>(current,1.0));
        return;
    }

    let decay = 0.83;

    let left_pixel  = textureLoad(inputTex, vec2<u32>(gid.x + 1,gid.y), 0).rgb * decay;
    let right_pixel = textureLoad(inputTex, vec2<u32>(gid.x - 1,gid.y), 0).rgb * decay;
    let up_pixel    = textureLoad(inputTex, vec2<u32>(gid.x,gid.y + 1), 0).rgb * decay;
    let down_pixel  = textureLoad(inputTex, vec2<u32>(gid.x,gid.y - 1), 0).rgb * decay;

    let mx = max(
        max(left_pixel, right_pixel),
        max(up_pixel, down_pixel)
    );

    let avg =
    (left_pixel +
     right_pixel +
     up_pixel +
     down_pixel) * 0.25;

    let result = mix(max(current, mx), avg, 0.15);

    textureStore(
        outputTex,
        vec2<u32>(gid.xy),
        vec4<f32>(result, 1.0)
    );
}

@group(0) @binding(3)
var<uniform> sky_light: vec3<f32>;

@compute @workgroup_size(16,16)
fn set_lit_tiles(@builtin(global_invocation_id) gid : vec3<u32>){
    let size = textureDimensions(inputTex);
    let current = textureLoad(inputTex, vec2<u32>(gid.x,gid.y), 0);

    let tile = textureLoad(tiles,vec2<u32>((gid.x),(size.y-(gid.y) )),0).r;

    if tile == 1{
        return;
    }

    var colour = vec4<f32>(0.,0.,0.,1.0);

    if tile == 0{
        colour = vec4<f32>(sky_light,1.);
    }
    else if tile == 4{
        colour = vec4<f32>(1.0,0.7,0.6,1.);
    }
    else if tile == 6{
        colour =  vec4<f32>(0.2,0.6,1.0,1.);
    }
    else if tile == 9{
        colour = vec4<f32>(0.1,1.0,0.1,1.);
    }

    textureStore(outputTex, vec2<i32>(i32(gid.x), i32(gid.y)), max(colour,current));
}