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

`cargo run --example screenshot` renders a test scene and saves both the
animation's internal render target and the final window framebuffer to
`target/screenshots/` so the output can be inspected for blur.

## Drawing paths

`draw_line` draws each segment as an isolated quad with flat (butt) ends, so a
polyline built from `draw_line` calls leaves wedge shaped gaps at its corners.
`draw::draw_path` draws a whole polyline with round joins and round caps
instead:

```rust
mqanim::draw::draw_path(&points, 4., PURPLE);
```

A join polygon is only drawn once per vertex and the number of sides scales
with the line thickness, so thin paths stay cheap.
