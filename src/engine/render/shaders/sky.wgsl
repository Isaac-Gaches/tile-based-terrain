struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Sky{
    time: f32,
    top_colour: vec3<f32>,
    bottom_colour: vec3<f32>,
    cloud_main: vec3<f32>,
    cloud_edge: vec3<f32>,
}
@group(0) @binding(0)
var<uniform> sky: Sky;

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
    let colour = mix(sky.bottom_colour,sky.top_colour,(in.uv.y+1.0)/2.);
    return vec4<f32>(colour,1.0);
}

fn hash(p: vec2<f32>) -> vec2<f32> {
    // cheaper hash
    let p3 = fract(vec3<f32>(p.xyx) * 0.1031);
    let q = p3 + dot(p3, p3.yzx + 33.33);

    return fract((q.xx + q.yz) * q.zy) * 2.0 - 1.0;
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);

    // smooth interpolation
    let u = f * f * (3.0 - 2.0 * f);

    let va = dot(hash(i), f);
    let vb = dot(hash(i + vec2<f32>(1.0, 0.0)), f - vec2<f32>(1.0, 0.0));
    let vc = dot(hash(i + vec2<f32>(0.0, 1.0)), f - vec2<f32>(0.0, 1.0));
    let vd = dot(hash(i + vec2<f32>(1.0, 1.0)), f - vec2<f32>(1.0, 1.0));

    return mix(
        mix(va, vb, u.x),
        mix(vc, vd, u.x),
        u.y
    ) * 0.5 + 0.5;
}

fn fbm(p0: vec2<f32>) -> f32 {
    var p = p0;

    // manually unrolled = faster on many laptop GPUs
    var v = 0.0;

    v += noise(p) * 0.5;
    p *= 2.0;

    v += noise(p) * 0.25;
    p *= 2.0;

    v += noise(p) * 0.125;

    return v;
}

@fragment
fn nebular_fs(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = in.uv - vec2<f32>(0.0, 1.0);
    let t = sky.time * 5.0;

    // lower warp frequency slightly
    let warp = vec2<f32>(
        fbm(p + vec2<f32>(t, 0.0)),
        fbm(p - vec2<f32>(0.0, t))
    );

    let q = p + warp * 2.5;

    var n = fbm(q * 2.0);

    // cheaper detail layer
    n += noise(q * 5.0 + 10.0) * 0.18;

    n = smoothstep(0.42, 0.7, n);

    let density = n * n * n;

    let color = mix(
        sky.cloud_edge,
        sky.cloud_main,
        density
    );

    // avoid expensive length()
    let vignette =
        1.0 - smoothstep(0.64, 1.44, dot(p, p));

    let alpha = n * vignette * 0.4;

    return vec4<f32>(color * alpha, alpha);
}

/*
fn visibility(t: f32) -> f32 {
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
}*/
