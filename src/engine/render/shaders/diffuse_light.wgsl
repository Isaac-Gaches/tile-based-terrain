@group(0) @binding(0)
var inputTex: texture_2d<f32>;
@group(0) @binding(1)
var outputTex: texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(2)
var tiles: texture_2d<u32>;

@compute @workgroup_size(16,16)
fn set_lit_tiles(@builtin(global_invocation_id) gid : vec3<u32>){
    let size = textureDimensions(inputTex);

    let tile = textureLoad(tiles,vec2<u32>((gid.x),(size.y-(gid.y) )),0).r;

    if tile == 0{
        textureStore(outputTex, vec2<i32>(i32(gid.x), i32(gid.y)), vec4<f32>(1.0,1.0,1.0,1.));
        return;
    }
    else if tile == 4{
        textureStore(outputTex, vec2<i32>(i32(gid.x), i32(gid.y)), vec4<f32>(1.,0.6,0.5,1.));
        return;
    }
    else if tile == 6{
        textureStore(outputTex, vec2<i32>(i32(gid.x), i32(gid.y)), vec4<f32>(0.2,0.6,1.0,1.));
        return;
    }
    else if tile == 9{
        textureStore(outputTex, vec2<i32>(i32(gid.x), i32(gid.y)), vec4<f32>(0.1,0.7,0.1,1.));
        return;
    }
}

@compute @workgroup_size(16,16)
fn diffuse_horizontal(@builtin(global_invocation_id) gid : vec3<u32>){
    let size = textureDimensions(inputTex);
    if (gid.x >= size.x || gid.y >= size.y) {
        return;
    }

    let tile = textureLoad(tiles,vec2<u32>(gid.x,size.y-gid.y),0).r;
    let current = textureLoad(inputTex, vec2<u32>(gid.x,gid.y), 0);

    if tile == 1{
        textureStore(outputTex, vec2<u32>(gid.x, gid.y), current);
        return;
    }

    var mx = vec3<f32>(0.);
    let decay = 0.82;

    let left_pixel = textureLoad(inputTex, vec2<u32>(clamp(gid.x- 1,0,size.x- 1),gid.y), 0).rgb * decay;
    mx = max(left_pixel,mx);

    let right_pixel = textureLoad(inputTex, vec2<u32>(clamp(gid.x+1,0,size.x- 1),gid.y), 0).rgb * decay;
    mx = max(right_pixel,mx);

    let avg = ((left_pixel+right_pixel) * 0.5);
    let result = mix(vec3<f32>(max(mx.r,current.r),max(mx.g,current.g),max(mx.b,current.b)),avg,0.05);

    textureStore(outputTex, vec2<u32>(gid.x, gid.y), vec4<f32>(result,1.0));
}
@compute @workgroup_size(16,16)
fn diffuse_vertical(@builtin(global_invocation_id) gid : vec3<u32>){
    let size = textureDimensions(inputTex);
    if (gid.x >= size.x || gid.y >= size.y) {
        return;
    }

    let tile = textureLoad(tiles,vec2<u32>(gid.x,size.y-gid.y),0).r;
    let current = textureLoad(inputTex, vec2<u32>(gid.x,gid.y), 0);

    if tile == 1{
        textureStore(outputTex, vec2<u32>(gid.x, gid.y), current);
        return;
    }

    let decay = 0.82;
    var mx = vec3<f32>(0.);

    let up_pixel = textureLoad(inputTex, vec2<u32>(gid.x,clamp(gid.y+ 1,0,size.y- 1)), 0).rgb * decay;
    mx = max(up_pixel,mx);

    let down_pixel = textureLoad(inputTex, vec2<u32>(gid.x,clamp(gid.y- 1,0,size.y- 1)), 0).rgb * decay;
    mx = max(down_pixel,mx);

    let avg = (up_pixel+down_pixel * 0.5);
    let result = mix(vec3<f32>(max(mx.r,current.r),max(mx.g,current.g),max(mx.b,current.b)),avg,0.05);

    textureStore(outputTex, vec2<u32>(gid.x, gid.y), vec4<f32>(result,1.0));
}

