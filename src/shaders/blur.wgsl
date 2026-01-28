// Gaussian Blur Compute Shader (Separable, Two-Pass)
// Pass 1: Horizontal blur, Pass 2: Vertical blur
// Set params.direction = vec2(1.0, 0.0) for horizontal, vec2(0.0, 1.0) for vertical

struct BlurParams {
    direction: vec2<f32>,  // (1,0) for horizontal, (0,1) for vertical
    radius: f32,           // blur radius in pixels
    sigma: f32,            // gaussian sigma (typically radius / 3)
}

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var output_texture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> params: BlurParams;

// Gaussian weight function
fn gaussian(x: f32, sigma: f32) -> f32 {
    let sigma2 = sigma * sigma;
    return exp(-(x * x) / (2.0 * sigma2)) / (2.506628274631 * sigma); // sqrt(2*PI) ≈ 2.506628
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let dims = textureDimensions(input_texture);
    
    // Bounds checking
    if (global_id.x >= dims.x || global_id.y >= dims.y) {
        return;
    }
    
    let coords = vec2<i32>(global_id.xy);
    let direction = vec2<i32>(params.direction);
    let radius = i32(params.radius);
    let sigma = max(params.sigma, 0.001); // Prevent division by zero
    
    var color_sum = vec4<f32>(0.0);
    var weight_sum: f32 = 0.0;
    
    // Sample along the blur direction
    for (var i: i32 = -radius; i <= radius; i++) {
        let sample_coords = coords + direction * i;
        
        // Clamp to texture bounds
        let clamped_coords = clamp(
            sample_coords,
            vec2<i32>(0),
            vec2<i32>(dims) - vec2<i32>(1)
        );
        
        let weight = gaussian(f32(i), sigma);
        let sample_color = textureLoad(input_texture, clamped_coords, 0);
        
        color_sum += sample_color * weight;
        weight_sum += weight;
    }
    
    // Normalize by total weight
    let final_color = color_sum / weight_sum;
    
    textureStore(output_texture, vec2<i32>(global_id.xy), final_color);
}
