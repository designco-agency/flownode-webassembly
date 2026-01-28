// Hue, Saturation, and Lightness Adjustment Compute Shader
// hue_shift: -1.0 to 1.0 (full rotation, 0.0 = no change)
// saturation: -1.0 to infinity (-1.0 = grayscale, 0.0 = no change, 1.0 = double saturation)
// lightness: -1.0 to 1.0 (additive, 0.0 = no change)

struct HSLParams {
    hue_shift: f32,    // -1.0 to 1.0 (maps to -360° to +360°)
    saturation: f32,   // -1.0 to inf, multiplier
    lightness: f32,    // -1.0 to 1.0, additive
    _padding: f32,
}

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var output_texture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> params: HSLParams;

// Convert RGB to HSL
fn rgb_to_hsl(rgb: vec3<f32>) -> vec3<f32> {
    let r = rgb.r;
    let g = rgb.g;
    let b = rgb.b;
    
    let max_c = max(max(r, g), b);
    let min_c = min(min(r, g), b);
    let delta = max_c - min_c;
    
    // Lightness
    let l = (max_c + min_c) * 0.5;
    
    // Saturation
    var s: f32 = 0.0;
    if (delta > 0.0) {
        if (l < 0.5) {
            s = delta / (max_c + min_c);
        } else {
            s = delta / (2.0 - max_c - min_c);
        }
    }
    
    // Hue
    var h: f32 = 0.0;
    if (delta > 0.0) {
        if (max_c == r) {
            h = (g - b) / delta;
            if (g < b) {
                h += 6.0;
            }
        } else if (max_c == g) {
            h = 2.0 + (b - r) / delta;
        } else {
            h = 4.0 + (r - g) / delta;
        }
        h /= 6.0;
    }
    
    return vec3<f32>(h, s, l);
}

// Helper function for HSL to RGB conversion
fn hue_to_rgb(p: f32, q: f32, t: f32) -> f32 {
    var t_mod = t;
    if (t_mod < 0.0) {
        t_mod += 1.0;
    }
    if (t_mod > 1.0) {
        t_mod -= 1.0;
    }
    
    if (t_mod < 1.0 / 6.0) {
        return p + (q - p) * 6.0 * t_mod;
    }
    if (t_mod < 0.5) {
        return q;
    }
    if (t_mod < 2.0 / 3.0) {
        return p + (q - p) * (2.0 / 3.0 - t_mod) * 6.0;
    }
    return p;
}

// Convert HSL to RGB
fn hsl_to_rgb(hsl: vec3<f32>) -> vec3<f32> {
    let h = hsl.x;
    let s = hsl.y;
    let l = hsl.z;
    
    if (s == 0.0) {
        // Achromatic (gray)
        return vec3<f32>(l, l, l);
    }
    
    var q: f32;
    if (l < 0.5) {
        q = l * (1.0 + s);
    } else {
        q = l + s - l * s;
    }
    let p = 2.0 * l - q;
    
    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);
    
    return vec3<f32>(r, g, b);
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let dims = textureDimensions(input_texture);
    
    // Bounds checking
    if (global_id.x >= dims.x || global_id.y >= dims.y) {
        return;
    }
    
    let coords = vec2<i32>(global_id.xy);
    let color = textureLoad(input_texture, coords, 0);
    
    // Convert to HSL
    var hsl = rgb_to_hsl(color.rgb);
    
    // Apply hue shift (wrap around)
    hsl.x = fract(hsl.x + params.hue_shift);
    
    // Apply saturation (multiplicative, clamped)
    hsl.y = clamp(hsl.y * (1.0 + params.saturation), 0.0, 1.0);
    
    // Apply lightness (additive, clamped)
    hsl.z = clamp(hsl.z + params.lightness, 0.0, 1.0);
    
    // Convert back to RGB
    let rgb = hsl_to_rgb(hsl);
    
    let final_color = vec4<f32>(rgb, color.a);
    
    textureStore(output_texture, coords, final_color);
}
