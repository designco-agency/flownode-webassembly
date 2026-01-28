// Brightness and Contrast Adjustment Compute Shader
// brightness: -1.0 to 1.0 (0.0 = no change)
// contrast: -1.0 to infinity (0.0 = no change, 1.0 = double contrast)

struct BrightnessContrastParams {
    brightness: f32,  // -1.0 to 1.0, additive offset
    contrast: f32,    // -1.0 to inf, multiplier around midpoint
    _padding: vec2<f32>,
}

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var output_texture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> params: BrightnessContrastParams;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let dims = textureDimensions(input_texture);
    
    // Bounds checking
    if (global_id.x >= dims.x || global_id.y >= dims.y) {
        return;
    }
    
    let coords = vec2<i32>(global_id.xy);
    var color = textureLoad(input_texture, coords, 0);
    
    // Apply brightness (additive)
    var rgb = color.rgb + vec3<f32>(params.brightness);
    
    // Apply contrast (multiply around 0.5 midpoint)
    // contrast_factor: 0 = no contrast (all gray), 1 = normal, >1 = enhanced
    let contrast_factor = params.contrast + 1.0;
    rgb = (rgb - vec3<f32>(0.5)) * contrast_factor + vec3<f32>(0.5);
    
    // Clamp to valid range
    rgb = clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0));
    
    let final_color = vec4<f32>(rgb, color.a);
    
    textureStore(output_texture, coords, final_color);
}
