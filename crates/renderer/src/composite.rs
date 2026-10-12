//! Composite background/foreground media under and over lyric frames.

/// Straight-alpha RGBA image buffer.
#[derive(Clone, Debug)]
pub struct RgbaImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl RgbaImage {
    pub fn new(width: u32, height: u32, rgba: Vec<u8>) -> Result<Self, String> {
        let expected = width as usize * height as usize * 4;
        if rgba.len() != expected {
            return Err("RGBA buffer size does not match dimensions".into());
        }
        Ok(Self {
            width,
            height,
            rgba,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FitMode {
    Cover,
    Contain,
    Fill,
}

impl FitMode {
    pub fn parse(s: &str) -> Self {
        match s {
            "contain" => Self::Contain,
            "fill" => Self::Fill,
            _ => Self::Cover,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LayerBlit {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub opacity: f32,
    pub fit: FitMode,
    pub image: RgbaImage,
}

/// Blend `src` over `dst` (both straight alpha) at pixel `(dx, dy)`.
fn blend_pixel(dst: &mut [u8], src: &[u8], opacity: f32) {
    let sa = (src[3] as f32 / 255.0) * opacity.clamp(0.0, 1.0);
    if sa <= 0.0 {
        return;
    }
    let da = dst[3] as f32 / 255.0;
    let out_a = sa + da * (1.0 - sa);
    if out_a <= 0.0 {
        dst[0] = 0;
        dst[1] = 0;
        dst[2] = 0;
        dst[3] = 0;
        return;
    }
    for i in 0..3 {
        let s = src[i] as f32 / 255.0;
        let d = dst[i] as f32 / 255.0;
        let out = (s * sa + d * da * (1.0 - sa)) / out_a;
        dst[i] = (out * 255.0).round().clamp(0.0, 255.0) as u8;
    }
    dst[3] = (out_a * 255.0).round().clamp(0.0, 255.0) as u8;
}

fn sample_nearest(img: &RgbaImage, u: f32, v: f32) -> [u8; 4] {
    if img.width == 0 || img.height == 0 {
        return [0, 0, 0, 0];
    }
    let x = (u * img.width as f32).floor().clamp(0.0, img.width as f32 - 1.0) as u32;
    let y = (v * img.height as f32)
        .floor()
        .clamp(0.0, img.height as f32 - 1.0) as u32;
    let i = ((y * img.width + x) * 4) as usize;
    [
        img.rgba[i],
        img.rgba[i + 1],
        img.rgba[i + 2],
        img.rgba[i + 3],
    ]
}

fn dest_uv(fit: FitMode, px: u32, py: u32, dw: u32, dh: u32, sw: u32, sh: u32) -> Option<(f32, f32)> {
    if dw == 0 || dh == 0 || sw == 0 || sh == 0 {
        return None;
    }
    match fit {
        FitMode::Fill => Some((px as f32 / dw as f32, py as f32 / dh as f32)),
        FitMode::Contain | FitMode::Cover => {
            let scale = if fit == FitMode::Cover {
                (dw as f32 / sw as f32).max(dh as f32 / sh as f32)
            } else {
                (dw as f32 / sw as f32).min(dh as f32 / sh as f32)
            };
            let rw = sw as f32 * scale;
            let rh = sh as f32 * scale;
            let ox = (dw as f32 - rw) * 0.5;
            let oy = (dh as f32 - rh) * 0.5;
            let sx = px as f32 - ox;
            let sy = py as f32 - oy;
            if sx < 0.0 || sy < 0.0 || sx >= rw || sy >= rh {
                return None;
            }
            Some((sx / rw, sy / rh))
        }
    }
}

/// Draw one media layer into a canvas-sized straight-alpha buffer.
pub fn blit_layer(dst: &mut [u8], canvas_w: u32, canvas_h: u32, layer: &LayerBlit) {
    let opacity = layer.opacity.clamp(0.0, 1.0);
    if opacity <= 0.0 || layer.width == 0 || layer.height == 0 {
        return;
    }
    let expected = canvas_w as usize * canvas_h as usize * 4;
    if dst.len() != expected {
        return;
    }
    for py in 0..layer.height {
        for px in 0..layer.width {
            let dx = layer.x + px as i32;
            let dy = layer.y + py as i32;
            if dx < 0 || dy < 0 || dx as u32 >= canvas_w || dy as u32 >= canvas_h {
                continue;
            }
            let Some((u, v)) = dest_uv(
                layer.fit,
                px,
                py,
                layer.width,
                layer.height,
                layer.image.width,
                layer.image.height,
            ) else {
                continue;
            };
            let src = sample_nearest(&layer.image, u, v);
            let di = ((dy as u32 * canvas_w + dx as u32) * 4) as usize;
            blend_pixel(&mut dst[di..di + 4], &src, opacity);
        }
    }
}

/// Compose `under` layers, then `lyrics`, then `over` layers into a new straight-alpha buffer.
pub fn compose_stack(
    canvas_w: u32,
    canvas_h: u32,
    under: &[LayerBlit],
    lyrics: &[u8],
    over: &[LayerBlit],
) -> Result<Vec<u8>, String> {
    let expected = canvas_w as usize * canvas_h as usize * 4;
    if lyrics.len() != expected {
        return Err("Lyric buffer size does not match canvas".into());
    }
    let mut out = vec![0u8; expected];
    for layer in under {
        blit_layer(&mut out, canvas_w, canvas_h, layer);
    }
    for i in (0..expected).step_by(4) {
        let src = &lyrics[i..i + 4];
        blend_pixel(&mut out[i..i + 4], src, 1.0);
    }
    for layer in over {
        blit_layer(&mut out, canvas_w, canvas_h, layer);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(w: u32, h: u32, rgba: [u8; 4]) -> RgbaImage {
        let mut buf = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..w * h {
            buf.extend_from_slice(&rgba);
        }
        RgbaImage::new(w, h, buf).unwrap()
    }

    #[test]
    fn background_stays_under_lyrics_and_foreground_over() {
        let canvas_w = 4u32;
        let canvas_h = 1u32;
        let lyrics = {
            let mut v = vec![0u8; 16];
            // Pixel 1: opaque white lyric
            v[4] = 255;
            v[5] = 255;
            v[6] = 255;
            v[7] = 255;
            v
        };
        let under = LayerBlit {
            x: 0,
            y: 0,
            width: 4,
            height: 1,
            opacity: 1.0,
            fit: FitMode::Fill,
            image: solid(4, 1, [255, 0, 0, 255]),
        };
        let over = LayerBlit {
            x: 2,
            y: 0,
            width: 1,
            height: 1,
            opacity: 1.0,
            fit: FitMode::Fill,
            image: solid(1, 1, [0, 0, 255, 255]),
        };
        let out = compose_stack(canvas_w, canvas_h, &[under], &lyrics, &[over]).unwrap();
        // Pixel 0: red background only
        assert_eq!(&out[0..4], &[255, 0, 0, 255]);
        // Pixel 1: white lyric over red
        assert_eq!(&out[4..8], &[255, 255, 255, 255]);
        // Pixel 2: blue foreground over red
        assert_eq!(&out[8..12], &[0, 0, 255, 255]);
    }

    #[test]
    fn empty_under_keeps_lyric_alpha() {
        let lyrics = vec![10, 20, 30, 128, 0, 0, 0, 0];
        let out = compose_stack(2, 1, &[], &lyrics, &[]).unwrap();
        assert_eq!(&out[0..4], &[10, 20, 30, 128]);
        assert_eq!(&out[4..8], &[0, 0, 0, 0]);
    }
}
