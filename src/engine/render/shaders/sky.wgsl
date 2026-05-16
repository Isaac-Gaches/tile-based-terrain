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

fn hash2(p: vec2<f32>) -> vec2<f32> {
    let x = dot(p, vec2<f32>(127.1, 311.7));
    let y = dot(p, vec2<f32>(269.5, 183.3));

    return normalize(
        fract(sin(vec2<f32>(x, y)) * 43758.5453) * 2.0 - 1.0
    );
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);

    let u = f * f * (3.0 - 2.0 * f);

    let ga = hash2(i + vec2<f32>(0.0, 0.0));
    let gb = hash2(i + vec2<f32>(1.0, 0.0));
    let gc = hash2(i + vec2<f32>(0.0, 1.0));
    let gd = hash2(i + vec2<f32>(1.0, 1.0));

    let va = dot(ga, f - vec2<f32>(0.0, 0.0));
    let vb = dot(gb, f - vec2<f32>(1.0, 0.0));
    let vc = dot(gc, f - vec2<f32>(0.0, 1.0));
    let vd = dot(gd, f - vec2<f32>(1.0, 1.0));

    return mix(
        mix(va, vb, u.x),
        mix(vc, vd, u.x),
        u.y
    ) * 0.5 + 0.5;
}

fn fbm(p0: vec2<f32>) -> f32 {
    var p = p0;
    var value = 0.0;
    var amplitude = 0.5;

    for (var i = 0; i < 3; i++) {
        value += noise(p) * amplitude;
        p *= 2.0;
        amplitude *= 0.5;
    }

    return value;
}

@fragment
fn nebular_fs(in: VertexOutput) -> @location(0) vec4<f32> {
    var p = in.uv - vec2<f32>(0.,1.0);

    let t = sky.time * 5.;

    let warp = vec2<f32>(
        fbm(p * 1.2 + vec2<f32>(t, 0.0)),
        fbm(p * 1.2 - vec2<f32>(0.0, t))
    );

    let q = p +(warp) * 3.0;

    var n = fbm(q * 2.0);

    let detail = fbm(q * 6.0 + 10.0) * 0.25;
        n += detail;

    n = smoothstep(0.4, 0.7, n);

    let color = mix(sky.cloud_edge, sky.cloud_main, pow(n, 3.0));

    let vignette = 1.0 - smoothstep(0.8, 1.2, length(p));

    let alpha = n * vignette * 0.4 /** visibility(sky.time)*/;

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
