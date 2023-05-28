// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) texture: vec2<f32>,
    @location(4) overlay: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) texture: vec2<f32>,
    @location(3) block_light: vec2<f32>,
    @location(4) ao: f32,
};

struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

fn ao_color(ao: u32) -> f32 {
    return f32(ao) * 0.3333;
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.color = in.color;
    out.normal = in.normal;
    out.texture = in.texture;
    out.block_light = vec2<f32>(f32(in.overlay & 15u), f32((in.overlay >> 8u) & 15u));
    out.ao = ao_color((in.overlay >> 4u) & 15u);
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    return out;
}

// Fragment shader

@group(1) @binding(0)
var t_atlas: texture_2d<f32>;
@group(1) @binding(1)
var s_atlas: sampler;

@group(2) @binding(0)
var t_lightmap: texture_2d<f32>;
@group(2) @binding(1)
var s_lightmap: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var sampleT = textureSample(t_atlas, s_atlas, in.texture);
    var light = textureSample(t_lightmap, s_lightmap, in.block_light * (1.0 / 15.0));
    if (sampleT.w < 0.1) {
        discard;
    }
    let ao_modifier = pow(mix(0.15, 1.0, in.ao), 0.5);
    let ao = vec3<f32>(ao_modifier);
    
    var color = vec4<f32>(in.color * ao, 1.0);
    return sampleT * color * light;
}
