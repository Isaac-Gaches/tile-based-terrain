@group(0) @binding(0)
var inputTex: texture_2d<f32>;
@group(0) @binding(1)
var outputTex: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(16,16)
fn smooth_horizontal(@builtin(global_invocation_id) gid : vec3<u32>){
    let size = textureDimensions(inputTex);
    if (gid.x >= size.x || gid.y >= size.y) {
        return;
    }
    var sum = vec4<f32>(0.0);

    for (var i = -1; i <= 1; i += 1) {
        var x = i32(gid.x) + i;
        x = clamp(x, 0, i32(size.x) - 1);

        var pixel = textureLoad(inputTex, vec2<i32>(x,i32(gid.y)), 0);

        sum += pixel;
    }

    sum /= 3.;
    let current_pixel = textureLoad(inputTex, vec2<i32>(i32(gid.x),i32(gid.y)), 0);
    let new_pixel = vec4<f32>(max(sum.r,current_pixel.r),max(sum.g,current_pixel.g),max(sum.b,current_pixel.b),1.);

    textureStore(outputTex, vec2<i32>(i32(gid.x), i32(gid.y)), new_pixel);
}

@compute @workgroup_size(16,16)
fn smooth_vertical(@builtin(global_invocation_id) gid : vec3<u32>){

    let size = textureDimensions(inputTex);
    if (gid.x >= size.x || gid.y >= size.y) {
        return;
    }

    var sum = vec4<f32>(0.0);

    for (var i = -1; i <= 1; i += 1) {
        var y = i32(gid.y) + i;
        y = clamp(y, 0, i32(size.y) - 1);

        var pixel = textureLoad(inputTex, vec2<i32>(i32(gid.x),y), 0);

        sum += pixel;
    }

    sum /= 3.;
    let current_pixel = textureLoad(inputTex, vec2<i32>(i32(gid.x),i32(gid.y)), 0);
    let new_pixel = vec4<f32>(max(sum.r,current_pixel.r),max(sum.g,current_pixel.g),max(sum.b,current_pixel.b),1.);

    textureStore(outputTex, vec2<i32>(i32(gid.x), i32(gid.y)), new_pixel);
}