# MqAnim

This is a little animation library I am building on top of the create [Macroquad](https://github.com/not-fl3/macroquad).

This library is primarily used to make maths animations with Macroquad for my
blog [Roughly Understood](https://roughly-understood.com)

## Sharp rendering

The animation is rendered into an offscreen texture and then blitted to the
window. That texture is allocated at the resolution the animation is actually
displayed at (the on screen size multiplied by the display DPI scale) and text
drawn with `ui::draw_text_centered` is rasterized at exactly the size it is
displayed at. This means the output stays sharp even when the window is much
larger than the animation's world size (for example a full screen canvas on the
blog), instead of being upscaled from a fixed resolution texture.

If you are on a HiDPI/retina display, set `high_dpi: true` in the `Conf` you
pass to `#[macroquad::main]` so that Macroquad reports the physical resolution
of the framebuffer.

The offscreen texture is multisampled (4x) when the backend supports it, so
lines and curves come out anti-aliased even though they are drawn offscreen.
On those backends `Animation::enable_fxaa` is ignored, because the multisampled
texture is already anti-aliased and FXAA would only blur it (text in
particular). FXAA is still applied on backends without MSAA support
(WebGL1/GL2).
