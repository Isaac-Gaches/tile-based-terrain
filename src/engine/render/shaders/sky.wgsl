struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Colours{
    top_colour: vec3<f32>,
    bottom_colour: vec3<f32>,
}
@group(0) @binding(0)
var<uniform> colours: Colours;

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var out:VertexOutput;
    out.position = vec4<f32>(in.position,0.99,1.0);
    out.uv = in.position;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let colour = mix(colours.bottom_colour,colours.top_colour,(in.uv.y+1.0)/2.);
    return vec4<f32>(colour,1.0);
}