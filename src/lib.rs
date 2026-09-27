// use std::cell::OnceCell;

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::OnceLock;

use macroquad::prelude::*;
pub mod plot;
pub mod ui;

static DEFAULT_FONT: OnceLock<Font> = OnceLock::new();

/// Scale factor (world units -> logical screen pixels) that text helpers must
/// use so that glyphs are rasterized at the resolution they are displayed at.
/// Updated by [`Animation::set_camera`].
static TEXT_RENDER_SCALE: AtomicU32 = AtomicU32::new(1.0f32.to_bits());

pub(crate) fn text_render_scale() -> f32 {
    f32::from_bits(TEXT_RENDER_SCALE.load(Ordering::Relaxed))
}

fn set_text_render_scale(scale: f32) {
    TEXT_RENDER_SCALE.store(scale.to_bits(), Ordering::Relaxed);
}

const RESIZE_HYSTERESIS: f32 = 0.5; // 50% window growth or shrink will cause resize

enum RenderState {
    CameraRendering,
    ScreenRendering,
}

pub struct Animation {
    render_target: RenderTarget,
    camera: Camera2D,
    bg_color: Color,
    render_state: RenderState,
    draw_size: Vec2,
    filter_mode: FilterMode,
    fxaa_material: Option<Material>,
    blit_material: Option<Material>,
    width: f32,
    height: f32,
    scale: f32,
    auto_resize: bool,
}

impl Animation {
    pub fn new(start_width: f32, start_height: f32, bg_color: Option<Color>) -> Self {
        // Screen dimensions will be:
        //     x: -width/2 -> width/2 (left -> right)
        //     y: -height/2 -> height/2 (bottom -> top)
        //
        // The render target is allocated at the resolution the animation is
        // displayed at (native resolution, including the DPI scale) so that
        // content never has to be upscaled.
        let scale = Self::compute_scale(start_width, start_height);
        set_text_render_scale(scale);
        let render_target = create_render_target(
            start_width * scale,
            start_height * scale,
            FilterMode::Linear,
        );

        let mut camera = Camera2D::from_display_rect(Rect::new(0., 0., start_width, start_height));

        camera.render_target = Some(render_target.clone());
        camera.target = vec2(0., 0.);

        let bg_color = if let Some(bg_color) = bg_color {
            bg_color
        } else {
            Color {
                r: 43. / 255.,
                g: 44. / 255.,
                b: 47. / 255.,
                a: 1.,
            }
        };
        let font = load_ttf_font_from_bytes(include_bytes!("./font/Droid Sans Mono.ttf"))
            .expect("The Font load failed for Droid Sans Mono ttf.");

        DEFAULT_FONT
            .set(font)
            .expect("Failed to set the Default font to Droid Sans Mono");

        // Copies the render target to the window while forcing the output
        // alpha to 1. Without this the alpha channel of the render target
        // (which is left partially transparent by macroquad's straight alpha
        // blending) would be applied a second time, darkening and washing out
        // every anti-aliased pixel (text especially).
        //
        // GLSL shaders are not available on the Metal backend, in which case
        // fall back to the default blit instead of failing.
        let blit_material = match load_material(
            ShaderSource::Glsl {
                vertex: BLIT_VERTEX_SHADER,
                fragment: BLIT_FRAGMENT_SHADER,
            },
            MaterialParams::default(),
        ) {
            Ok(material) => Some(material),
            Err(err) => {
                macroquad::logging::warn!(
                    "mqanim: failed to load the opaque blit shader ({err}), falling back to \
                     macroquad's default blit; anti-aliased edges may look darker"
                );
                None
            }
        };

        Self {
            render_target,
            camera,
            bg_color,
            filter_mode: FilterMode::Linear,
            render_state: RenderState::ScreenRendering,
            draw_size: vec2(start_width * scale, start_height * scale),
            fxaa_material: None,
            blit_material,
            width: start_width,
            height: start_height,
            scale,
            auto_resize: false,
        }
    }

    pub fn enable_auto_resize(&mut self) {
        self.auto_resize = true;
    }
    pub fn filter_mode(&mut self, filter_mode: FilterMode) {
        self.filter_mode = filter_mode;
        self.render_target.texture.set_filter(filter_mode);
    }

    pub fn get_world_mouse(&self) -> Vec2 {
        let mouse: Vec2 = mouse_position().into();
        self.screen_to_world(mouse)
    }

    pub fn screen_to_world(&self, point: Vec2) -> Vec2 {
        // Mouse position in the virtual screen
        let offset = self.display_offset();
        Vec2 {
            x: ((point.x - offset.x) / self.scale) - 0.5 * self.width,
            y: 0.5 * self.height - (point.y - offset.y) / self.scale,
        }
    }

    /// Enable FXAA on the final blit.
    ///
    /// When the render target is multisampled (which it is on every backend
    /// that supports it) this does nothing: the render target is already
    /// anti-aliased, and running FXAA on top of it only blurs the image, text
    /// especially. FXAA is still used as a fallback on backends without MSAA
    /// support (WebGL1/GL2).
    pub fn enable_fxaa(&mut self) {
        if supports_msaa_render_targets() {
            macroquad::logging::info!(
                "mqanim: the render target is multisampled, ignoring enable_fxaa (FXAA \
                 would only blur the image)"
            );
            self.fxaa_material = None;
            return;
        }

        let uniforms = vec![UniformDesc::new("texture_size", UniformType::Float2)];
        let material = match load_material(
            ShaderSource::Glsl {
                vertex: FXAA_VERTEX_SHADER,
                fragment: FXAA_FRAGMENT_SHADER,
            },
            MaterialParams {
                uniforms,
                ..Default::default()
            },
        ) {
            Ok(material) => material,
            Err(err) => {
                macroquad::logging::warn!(
                    "mqanim: failed to load the FXAA shader ({err}); continuing without FXAA"
                );
                return;
            }
        };

        self.fxaa_material = Some(material);
    }

    pub fn set_camera(&mut self) {
        self.scale = Self::compute_scale(self.width, self.height);
        set_text_render_scale(self.scale);

        if self.auto_resize {
            if self.width > self.draw_size.x * (1. + RESIZE_HYSTERESIS)
                || self.height > self.draw_size.y * (1. + RESIZE_HYSTERESIS)
            {
                self.resize_render_target(
                    self.width * RESIZE_HYSTERESIS,
                    self.height * RESIZE_HYSTERESIS,
                );
            } else if self.width <= self.draw_size.x * (1. - RESIZE_HYSTERESIS)
                || self.height <= self.draw_size.y * (1. - RESIZE_HYSTERESIS)
            {
                self.resize_render_target(
                    self.width * 1. / RESIZE_HYSTERESIS,
                    self.height * 1. / RESIZE_HYSTERESIS,
                );
            }
        }

        self.resize_to_native_resolution();

        set_camera(&self.camera);
        clear_background(self.bg_color);
        self.render_state = RenderState::CameraRendering;
    }

    pub fn set_default_camera(&mut self) {
        self.render_state = RenderState::ScreenRendering;
        set_default_camera();
    }

    pub fn draw_frame(&mut self) {
        if matches!(self.render_state, RenderState::CameraRendering) {
            panic!("Animation::set_default_camera must be called before you can draw the frame to the screen");
        }

        if let Some(material) = &self.fxaa_material {
            material.set_uniform(
                "texture_size",
                vec2(
                    self.render_target.texture.width(),
                    self.render_target.texture.height(),
                ),
            );
            gl_use_material(material);
        } else if let Some(material) = &self.blit_material {
            gl_use_material(material);
        } else {
            gl_use_default_material();
        }

        clear_background(self.bg_color);

        self.scale = Self::compute_scale(self.width, self.height);
        set_text_render_scale(self.scale);

        self.draw_size = self.display_size();
        let offset = self.display_offset();
        // Draw 'render_target' to window screen, properly scaled and letterboxed
        draw_texture_ex(
            &self.render_target.texture,
            offset.x,
            offset.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(self.draw_size),
                ..Default::default()
            },
        );

        gl_use_default_material();
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }

    pub fn size(&self) -> Vec2 {
        vec2(self.width, self.height)
    }

    /// The texture this animation is rendered into before it is scaled to the
    /// window/letterboxed by [`Animation::draw_frame`].
    ///
    /// The texture is allocated at the resolution the animation is displayed
    /// at (i.e. the world size scaled by [`Animation::scale`] and the display
    /// DPI scale), so it can be used for screenshots without any upscaling.
    pub fn render_target_texture(&self) -> &Texture2D {
        &self.render_target.texture
    }

    fn compute_scale(width: f32, height: f32) -> f32 {
        f32::min(screen_width() / width, screen_height() / height)
    }

    /// The animation's on screen size in logical pixels, snapped to whole
    /// physical pixels so that the final blit stays pixel perfect.
    fn display_size(&self) -> Vec2 {
        let dpi_scale = miniquad::window::dpi_scale();
        vec2(
            snap_to_physical_pixel(self.width * self.scale, dpi_scale),
            snap_to_physical_pixel(self.height * self.scale, dpi_scale),
        )
    }

    /// Where the animation is drawn on screen, snapped to whole physical
    /// pixels so that the final blit stays pixel perfect.
    fn display_offset(&self) -> Vec2 {
        let dpi_scale = miniquad::window::dpi_scale();
        let size = self.display_size();
        vec2(
            snap_to_physical_pixel((screen_width() - size.x) * 0.5, dpi_scale),
            snap_to_physical_pixel((screen_height() - size.y) * 0.5, dpi_scale),
        )
    }

    /// Make sure the render target has the resolution the animation is
    /// currently displayed at.
    fn resize_to_native_resolution(&mut self) {
        let dpi_scale = miniquad::window::dpi_scale();
        let size = self.display_size();
        let (target_width, target_height) = render_target_size(size.x, size.y, dpi_scale);
        let texture_size = self.render_target.texture.size();
        if texture_size.x as u32 != target_width || texture_size.y as u32 != target_height {
            self.rebuild_render_target();
        }
    }

    fn resize_render_target(&mut self, new_width: f32, new_height: f32) {
        self.width = new_width;
        self.height = new_height;
        self.scale = Self::compute_scale(new_width, new_height);
        set_text_render_scale(self.scale);
        self.rebuild_render_target();
    }

    fn rebuild_render_target(&mut self) {
        let size = self.display_size();
        let render_target = create_render_target(size.x, size.y, self.filter_mode);

        let mut camera = Camera2D::from_display_rect(Rect::new(0., 0., self.width, self.height));

        camera.render_target = Some(render_target.clone());
        camera.target = vec2(0., 0.);

        self.camera = camera;
        self.render_target = render_target;
    }
}

/// Round `value` (in logical pixels) to the nearest whole physical pixel.
fn snap_to_physical_pixel(value: f32, dpi_scale: f32) -> f32 {
    (value * dpi_scale).round() / dpi_scale
}

/// Samples used for the multisampled render target. 4x is the safest value
/// that is supported on virtually every GL/GLES backend.
const MSAA_SAMPLE_COUNT: i32 = 4;

/// Create a render target covering `width` x `height` logical pixels at the
/// current DPI scale. Anti-aliased with MSAA when the backend supports it.
fn create_render_target(width: f32, height: f32, filter_mode: FilterMode) -> RenderTarget {
    let (target_width, target_height) =
        render_target_size(width, height, miniquad::window::dpi_scale());
    let render_target = if supports_msaa_render_targets() {
        render_target_ex(
            target_width,
            target_height,
            RenderTargetParams {
                sample_count: MSAA_SAMPLE_COUNT,
                depth: false,
            },
        )
    } else {
        render_target(target_width, target_height)
    };
    render_target.texture.set_filter(filter_mode);
    render_target
}

/// Whether the backend supports resolving multisampled render targets (false
/// on WebGL1 and GL2).
fn supports_msaa_render_targets() -> bool {
    unsafe { macroquad::window::get_internal_gl().quad_context.info() }
        .features
        .resolve_attachments
}

/// Size in physical pixels of a render target that displays `width` x `height`
/// logical pixels at the given DPI scale.
fn render_target_size(width: f32, height: f32, dpi_scale: f32) -> (u32, u32) {
    (
        (width * dpi_scale).round().max(1.) as u32,
        (height * dpi_scale).round().max(1.) as u32,
    )
}

pub fn map(val: f32, min1: f32, max1: f32, min2: f32, max2: f32) -> f32 {
    ((val - min1) / (max1 - min1)) * (max2 - min2) + min2
}

const BLIT_VERTEX_SHADER: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;

varying mediump vec2 uv;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
}"#;

const BLIT_FRAGMENT_SHADER: &str = r#"#version 100
precision mediump float;

varying vec2 uv;

uniform sampler2D Texture;

void main() {
    // Force alpha to 1: the color in the render target is already
    // composited over the opaque background, so blending it again with its
    // own (partial) alpha would darken every anti-aliased pixel.
    gl_FragColor = vec4(texture2D(Texture, uv).rgb, 1.0);
}"#;

const FXAA_VERTEX_SHADER: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;

varying lowp vec2 uv;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
}"#;
const FXAA_FRAGMENT_SHADER: &str = r#"
#version 100
precision lowp float;

// THIS CODE IS THANKS TO: 
// https://blog.simonrodriguez.fr/articles/2016/07/implementing_fxaa.html

varying vec2 uv;

// UNIFORMS
uniform sampler2D Texture;
uniform vec2 texture_size;

// CONSTANTS
const float EDGE_THRESHOLD_MIN = 0.0312;
const float EDGE_THRESHOLD_MAX = 0.125;
const int ITERATIONS = 12;
const float SUBPIXEL_QUALITY = 0.75;

float rgb2luma(vec3 rgb){
    return sqrt(dot(rgb, vec3(0.299, 0.587, 0.114)));
}

float get_quality(int i) {
    float quality = 0.0;
    if(i < 6) {
       quality = 1.0; 
    } else if(i == 6) {
       quality = 1.5; 
    } else if(i > 6 && i < 10) {
       quality = 2.0; 
    } else if(i == 10) {
       quality = 4.0; 
    } else {
        quality = 8.0;
    }

    return quality;
}

void main() {
    vec2 inverseScreenSize = vec2(1.0/texture_size.x, 1.0/texture_size.y);
    vec3 colorCenter = texture2D(Texture,uv).rgb;

    // Luma at the current fragment
    float lumaCenter = rgb2luma(colorCenter);

    // Directions
    vec2 up = vec2(0., -inverseScreenSize.y);
    vec2 down = vec2(0., inverseScreenSize.y);
    vec2 left = vec2(-inverseScreenSize.x, 0.);
    vec2 right = vec2(inverseScreenSize.x, 0.);

    // Luma at the four direct neighbours of the current fragment.
    float lumaDown = rgb2luma(texture2D(Texture, uv + down).rgb);
    float lumaUp = rgb2luma(texture2D(Texture,uv + up).rgb);
    float lumaLeft = rgb2luma(texture2D(Texture,uv + left).rgb);
    float lumaRight = rgb2luma(texture2D(Texture,uv + right).rgb);

    // Find the maximum and minimum luma around the current fragment.
    float lumaMin = min(lumaCenter,min(min(lumaDown,lumaUp),min(lumaLeft,lumaRight)));
    float lumaMax = max(lumaCenter,max(max(lumaDown,lumaUp),max(lumaLeft,lumaRight)));

    // Compute the delta.
    float lumaRange = lumaMax - lumaMin;

    // If the luma variation is lower that a threshold (or if we are in a really dark area), we are not on an edge, don't perform any AA.
    if(lumaRange < max(EDGE_THRESHOLD_MIN,lumaMax*EDGE_THRESHOLD_MAX)){
        gl_FragColor = vec4(colorCenter, 1.0);
        return;
    }

    // Query the 4 remaining corners lumas.
    float lumaDownLeft = rgb2luma(texture2D(Texture,uv + down + left).rgb);
    float lumaUpRight = rgb2luma(texture2D(Texture,uv + up + right).rgb);
    float lumaUpLeft = rgb2luma(texture2D(Texture,uv + up + left).rgb);
    float lumaDownRight = rgb2luma(texture2D(Texture,uv + down + right).rgb);

    // Combine the four edges lumas (using intermediary variables for future computations with the same values).
    float lumaDownUp = lumaDown + lumaUp;
    float lumaLeftRight = lumaLeft + lumaRight;

    // Same for corners
    float lumaLeftCorners = lumaDownLeft + lumaUpLeft;
    float lumaDownCorners = lumaDownLeft + lumaDownRight;
    float lumaRightCorners = lumaDownRight + lumaUpRight;
    float lumaUpCorners = lumaUpRight + lumaUpLeft;

    // Compute an estimation of the gradient along the horizontal and vertical axis.
    float edgeHorizontal =  abs(-2.0 * lumaLeft + lumaLeftCorners)  + abs(-2.0 * lumaCenter + lumaDownUp ) * 2.0    + abs(-2.0 * lumaRight + lumaRightCorners);
    float edgeVertical =    abs(-2.0 * lumaUp + lumaUpCorners)      + abs(-2.0 * lumaCenter + lumaLeftRight) * 2.0  + abs(-2.0 * lumaDown + lumaDownCorners);

    // Is the local edge horizontal or vertical ?
    bool isHorizontal = (edgeHorizontal >= edgeVertical);

    // Select the two neighboring texels lumas in the opposite direction to the local edge.
    float luma1 = isHorizontal ? lumaDown : lumaLeft;
    float luma2 = isHorizontal ? lumaUp : lumaRight;
    // Compute gradients in this direction.
    float gradient1 = luma1 - lumaCenter;
    float gradient2 = luma2 - lumaCenter;

    // Which direction is the steepest ?
    bool is1Steepest = abs(gradient1) >= abs(gradient2);

    // Gradient in the corresponding direction, normalized.
    float gradientScaled = 0.25*max(abs(gradient1),abs(gradient2));

    // Choose the step size (one pixel) according to the edge direction.
    float stepLength = isHorizontal ? inverseScreenSize.y : inverseScreenSize.x;

    // Average luma in the correct direction.
    float lumaLocalAverage = 0.0;

    if(is1Steepest){
        // Switch the direction
        stepLength = - stepLength;
        lumaLocalAverage = 0.5*(luma1 + lumaCenter);
    } else {
        lumaLocalAverage = 0.5*(luma2 + lumaCenter);
    }

    // Shift UV in the correct direction by half a pixel.
    vec2 currentUv = uv;
    if(isHorizontal){
        currentUv.y += stepLength * 0.5;
    } else {
        currentUv.x += stepLength * 0.5;
    }

    // Compute offset (for each iteration step) in the right direction.
    vec2 offset = isHorizontal ? vec2(inverseScreenSize.x,0.0) : vec2(0.0,inverseScreenSize.y);
    // Compute UVs to explore on each side of the edge, orthogonally. The QUALITY allows us to step faster.
    vec2 uv1 = currentUv - offset;
    vec2 uv2 = currentUv + offset;

    // Read the lumas at both current extremities of the exploration segment, and compute the delta wrt to the local average luma.
    float lumaEnd1 = rgb2luma(texture2D(Texture,uv1).rgb);
    float lumaEnd2 = rgb2luma(texture2D(Texture,uv2).rgb);
    lumaEnd1 -= lumaLocalAverage;
    lumaEnd2 -= lumaLocalAverage;

    // If the luma deltas at the current extremities are larger than the local gradient, we have reached the side of the edge.
    bool reached1 = abs(lumaEnd1) >= gradientScaled;
    bool reached2 = abs(lumaEnd2) >= gradientScaled;
    bool reachedBoth = reached1 && reached2;

    // If the side is not reached, we continue to explore in this direction.
    if(!reached1){
        uv1 -= offset;
    }
    if(!reached2){
        uv2 += offset;
    }

    // If both sides have not been reached, continue to explore.
    if(!reachedBoth){

        for(int i = 2; i < ITERATIONS; i++){
            // If needed, read luma in 1st direction, compute delta.
            if(!reached1){
                lumaEnd1 = rgb2luma(texture2D(Texture, uv1).rgb);
                lumaEnd1 = lumaEnd1 - lumaLocalAverage;
            }
            // If needed, read luma in opposite direction, compute delta.
            if(!reached2){
                lumaEnd2 = rgb2luma(texture2D(Texture, uv2).rgb);
                lumaEnd2 = lumaEnd2 - lumaLocalAverage;
            }
            // If the luma deltas at the current extremities is larger than the local gradient, we have reached the side of the edge.
            reached1 = abs(lumaEnd1) >= gradientScaled;
            reached2 = abs(lumaEnd2) >= gradientScaled;
            reachedBoth = reached1 && reached2;

            // If the side is not reached, we continue to explore in this direction, with a variable quality.
            if(!reached1){
                uv1 -= offset * get_quality(i);
            }
            if(!reached2){
                uv2 += offset * get_quality(i);
            }

            // If both sides have been reached, stop the exploration.
            if(reachedBoth){ break;}
        }
    }

    // Compute the distances to each extremity of the edge.
    float distance1 = isHorizontal ? (uv.x - uv1.x) : (uv.y - uv1.y);
    float distance2 = isHorizontal ? (uv2.x - uv.x) : (uv2.y - uv.y);

    // which direction is the extremity of the edge closer ?
    bool isDirection1 = distance1 < distance2;
    float distanceFinal = min(distance1, distance2);

    // Length of the edge.
    float edgeThickness = (distance1 + distance2);

    // UV offset: read in the direction of the closest side of the edge.
    float pixelOffset = - distanceFinal / edgeThickness + 0.5;

    // Is the luma at center smaller than the local average ?
    bool isLumaCenterSmaller = lumaCenter < lumaLocalAverage;

    // If the luma at center is smaller than at its neighbour, the delta luma at each end should be positive (same variation).
    // (in the direction of the closer side of the edge.)
    bool correctVariation = ((isDirection1 ? lumaEnd1 : lumaEnd2) < 0.0) != isLumaCenterSmaller;

    // If the luma variation is incorrect, do not offset.
    float finalOffset = correctVariation ? pixelOffset : 0.0;

    // Sub-pixel shifting
    // Full weighted average of the luma over the 3x3 neighborhood.
    float lumaAverage = (1.0/12.0) * (2.0 * (lumaDownUp + lumaLeftRight) + lumaLeftCorners + lumaRightCorners);
    // Ratio of the delta between the global average and the center luma, over the luma range in the 3x3 neighborhood.
    float subPixelOffset1 = clamp(abs(lumaAverage - lumaCenter)/lumaRange,0.0,1.0);
    float subPixelOffset2 = (-2.0 * subPixelOffset1 + 3.0) * subPixelOffset1 * subPixelOffset1;
    // Compute a sub-pixel offset based on this delta.
    float subPixelOffsetFinal = subPixelOffset2 * subPixelOffset2 * SUBPIXEL_QUALITY;

    // Pick the biggest of the two offsets.
    finalOffset = max(finalOffset,subPixelOffsetFinal);

    // Compute the final UV coordinates.
    vec2 finalUv = uv;
    if(isHorizontal){
        finalUv.y += finalOffset * stepLength;
    } else {
        finalUv.x += finalOffset * stepLength;
    }

    // Read the color at the new UV coordinates, and use it.
    vec3 finalColor = texture2D(Texture,finalUv).rgb;

    gl_FragColor = vec4(finalColor, 1.0);
}"#;
