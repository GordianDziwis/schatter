[[stage(fragment)]]
fn main([[location(0)]] tex_coords: vec2<f32>) -> FragmentOutput {
    var out_color: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    let offsets: array<vec2<f32>, 9> = array<vec2<f32>, 9>(
        vec2<f32>(-1.0, -1.0), vec2<f32>(0.0, -1.0), vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0, 0.0), vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0),
        vec2<f32>(-1.0, 1.0), vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 1.0)
    );
    for (var i = 0u; i < 9u; i = i + 1u) {
        out_color = out_color + textureSample(tex, tex_sampler, tex_coords + offsets[i] / 9.0);
    }
    out_color = out_color / 9.0;
    return FragmentOutput(out_color);
}
