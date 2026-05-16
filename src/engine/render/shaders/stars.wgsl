struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct Star{
    @location(1) position: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) brightness: f32,
    @location(1) uv: vec2<f32>,
};

struct Time{
    time: f32,
}
@group(0) @binding(0)
var<uniform> time: Time;

fn hash(p: vec2<f32>) -> f32 {
    return fract(
        sin(dot(p, vec2<f32>(127.1, 311.7)))
        * 43758.5453
    );
}

@vertex
fn vs_main(
    in: VertexInput,
    star: Star,
) -> VertexOutput {

    var out: VertexOutput;

    out.position = vec4<f32>(
        in.position * 0.0015 + star.position,
        0.999,
        1.0
    );

    out.uv = in.position;

    let phase = hash(star.position) * 6.28318;

    let twinkle =
        0.6 +
        0.4 * sin(time.time * 6.28318 * 500.0 + phase);

    let visibility = star_visibility(time.time);

    out.brightness = twinkle * visibility * (star.position.y + 1.0) * 0.5;

    return out;
}

fn star_visibility(t: f32) -> f32 {
    if (t >= 0.8) {
        return smoothstep(0.8, 0.9, t);
    }

    if (t <= 0.10) {
        return 1.0;
    }

    if (t <= 0.2) {
        return 1.0 - smoothstep(0.1, 0.2, t);
    }

    return 0.0;
}

@fragment
fn fs_main(in: VertexOutput)
    -> @location(0) vec4<f32>
{
    return vec4<f32>(1.0,1.0,1.0, (1.0 - length(in.uv)) * in.brightness);
}