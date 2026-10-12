//! CPU compositing of block transitions on premultiplied RGBA rasters.
use selah_domain::TransitionSpec;

/// Premultiplied RGBA pixels.
#[derive(Clone)]
pub struct Raster {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}
impl Raster {
    pub fn from_straight(width: u32, height: u32, rgba: &[u8]) -> Self {
        let mut data = rgba.to_vec();
        for p in data.chunks_exact_mut(4) {
            let a = p[3] as u32;
            if a < 255 {
                for c in &mut p[..3] {
                    *c = ((*c as u32 * a + 127) / 255) as u8;
                }
            }
        }
        Self {
            width,
            height,
            data,
        }
    }
    /// Straight alpha, which is what NDI and Syphon expect.
    pub fn to_straight(&self) -> Vec<u8> {
        let mut out = self.data.clone();
        for p in out.chunks_exact_mut(4) {
            let a = p[3] as u32;
            if a > 0 && a < 255 {
                for c in &mut p[..3] {
                    *c = ((*c as u32 * 255 + a / 2) / a).min(255) as u8;
                }
            }
        }
        out
    }
    fn at(&self, x: i32, y: i32) -> [f32; 4] {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return [0.0; 4];
        }
        let i = (y as usize * self.width as usize + x as usize) * 4;
        [
            self.data[i] as f32,
            self.data[i + 1] as f32,
            self.data[i + 2] as f32,
            self.data[i + 3] as f32,
        ]
    }
}

/// Shared with the web implementation (`src/motion.ts`); both are checked against the same table.
pub fn ease(kind: &str, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match kind {
        "ease-in" => t * t * t,
        "ease-out" => 1.0 - (1.0 - t).powi(3),
        "ease-in-out" => {
            if t < 0.5 {
                4.0 * t * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
            }
        }
        "spring" => {
            let (c1, c3) = (1.70158f32, 2.70158f32);
            1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
        }
        _ => t,
    }
}

fn direction(spec: &TransitionSpec) -> (f32, f32) {
    match spec.direction.as_str() {
        "right" => (1.0, 0.0),
        "up" => (0.0, -1.0),
        "down" => (0.0, 1.0),
        _ => (-1.0, 0.0),
    }
}
fn add(a: [f32; 4], wa: f32, b: [f32; 4], wb: f32) -> [f32; 4] {
    [0, 1, 2, 3].map(|i| (a[i] * wa + b[i] * wb).min(255.0))
}
fn over(top: [f32; 4], bottom: [f32; 4]) -> [f32; 4] {
    add(top, 1.0, bottom, 1.0 - top[3] / 255.0)
}
fn build(w: u32, h: u32, mut f: impl FnMut(i32, i32) -> [f32; 4]) -> Raster {
    let mut data = Vec::with_capacity(w as usize * h as usize * 4);
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            data.extend(f(x, y).map(|v| v.round() as u8));
        }
    }
    Raster {
        width: w,
        height: h,
        data,
    }
}
fn box_blur(src: &Raster, radius: usize) -> Raster {
    if radius == 0 {
        return src.clone();
    }
    let (w, h) = (src.width as usize, src.height as usize);
    let window = (radius * 2 + 1) as u32;
    let mut tmp = vec![0u8; src.data.len()];
    for y in 0..h {
        for c in 0..4 {
            let mut sum: u32 = 0;
            for x in 0..=radius.min(w - 1) {
                sum += src.data[(y * w + x) * 4 + c] as u32;
            }
            for x in 0..w {
                tmp[(y * w + x) * 4 + c] = (sum / window) as u8;
                let add = x + radius + 1;
                if add < w {
                    sum += src.data[(y * w + add) * 4 + c] as u32;
                }
                if x >= radius {
                    sum -= src.data[(y * w + x - radius) * 4 + c] as u32;
                }
            }
        }
    }
    let mut out = vec![0u8; src.data.len()];
    for x in 0..w {
        for c in 0..4 {
            let mut sum: u32 = 0;
            for y in 0..=radius.min(h - 1) {
                sum += tmp[(y * w + x) * 4 + c] as u32;
            }
            for y in 0..h {
                out[(y * w + x) * 4 + c] = (sum / window) as u8;
                let add = y + radius + 1;
                if add < h {
                    sum += tmp[(add * w + x) * 4 + c] as u32;
                }
                if y >= radius {
                    sum -= tmp[((y - radius) * w + x) * 4 + c] as u32;
                }
            }
        }
    }
    Raster {
        width: src.width,
        height: src.height,
        data: out,
    }
}

/// Large radii blur a quarter-size copy and scale back up, which keeps 1080p inside a 30 fps frame budget.
fn fast_blur(src: &Raster, radius: f32) -> Raster {
    const F: usize = 4;
    if radius < F as f32 * 2.0 {
        return box_blur(src, radius as usize);
    }
    let (w, h) = (src.width as usize, src.height as usize);
    let (sw, sh) = (w.div_ceil(F), h.div_ceil(F));
    let mut small = vec![0u8; sw * sh * 4];
    for y in 0..sh {
        for x in 0..sw {
            let mut acc = [0u32; 4];
            let mut n = 0;
            for yy in (y * F)..((y + 1) * F).min(h) {
                for xx in (x * F)..((x + 1) * F).min(w) {
                    for (c, a) in acc.iter_mut().enumerate() {
                        *a += src.data[(yy * w + xx) * 4 + c] as u32;
                    }
                    n += 1;
                }
            }
            for (c, a) in acc.iter().enumerate() {
                small[(y * sw + x) * 4 + c] = (a / n) as u8;
            }
        }
    }
    let blurred = box_blur(
        &Raster {
            width: sw as u32,
            height: sh as u32,
            data: small,
        },
        (radius / F as f32) as usize,
    );
    let mut data = Vec::with_capacity(src.data.len());
    for y in 0..h {
        for x in 0..w {
            let i = ((y / F) * sw + x / F) * 4;
            data.extend_from_slice(&blurred.data[i..i + 4]);
        }
    }
    Raster {
        width: src.width,
        height: src.height,
        data,
    }
}

/// `t` is already eased progress. Both rasters must share dimensions.
pub fn compose(from: &Raster, to: &Raster, spec: &TransitionSpec, t: f32) -> Raster {
    let t = t.clamp(0.0, 1.0);
    let (w, h) = (to.width, to.height);
    let (wf, hf) = (w as f32, h as f32);
    let (dx, dy) = direction(spec);
    match spec.kind.as_str() {
        "fade" => build(w, h, |x, y| add(from.at(x, y), 1.0 - t, to.at(x, y), t)),
        "dip" => build(w, h, |x, y| {
            if t < 0.5 {
                add(from.at(x, y), 1.0 - 2.0 * t, [0.0; 4], 0.0)
            } else {
                add(to.at(x, y), 2.0 * t - 1.0, [0.0; 4], 0.0)
            }
        }),
        "slide" => {
            let (ox, oy) = (-dx * wf * (1.0 - t), -dy * hf * (1.0 - t));
            build(w, h, |x, y| {
                over(
                    to.at(x - ox.round() as i32, y - oy.round() as i32),
                    from.at(x, y),
                )
            })
        }
        "push" => {
            let (fx, fy) = (dx * wf * t, dy * hf * t);
            let (tx, ty) = (fx - dx * wf, fy - dy * hf);
            build(w, h, |x, y| {
                add(
                    from.at(x - fx.round() as i32, y - fy.round() as i32),
                    1.0,
                    to.at(x - tx.round() as i32, y - ty.round() as i32),
                    1.0,
                )
            })
        }
        "reveal" => {
            let (fx, fy) = (dx * wf * t, dy * hf * t);
            build(w, h, |x, y| {
                over(
                    from.at(x - fx.round() as i32, y - fy.round() as i32),
                    to.at(x, y),
                )
            })
        }
        "wipe" => {
            let f = spec.feather.max(0.0001);
            let edge = t * (1.0 + f);
            build(w, h, |x, y| {
                let u = match spec.direction.as_str() {
                    "right" => x as f32 / wf,
                    "up" => (hf - y as f32) / hf,
                    "down" => y as f32 / hf,
                    _ => (wf - x as f32) / wf,
                };
                let m = ((edge - u) / f).clamp(0.0, 1.0);
                add(from.at(x, y), 1.0 - m, to.at(x, y), m)
            })
        }
        "zoom" => {
            let (s1, s2) = (1.0 + 0.3 * t, 0.8 + 0.2 * t);
            let (cx, cy) = (wf / 2.0, hf / 2.0);
            build(w, h, |x, y| {
                let sample = |r: &Raster, s: f32| {
                    r.at(
                        (cx + (x as f32 - cx) / s).round() as i32,
                        (cy + (y as f32 - cy) / s).round() as i32,
                    )
                };
                add(sample(from, s1), 1.0 - t, sample(to, s2), t)
            })
        }
        "blur" => {
            let max = (w.max(h) as f32 / 40.0).max(1.0);
            let a = fast_blur(from, t * max);
            let b = fast_blur(to, (1.0 - t) * max);
            build(w, h, |x, y| add(a.at(x, y), 1.0 - t, b.at(x, y), t))
        }
        _ => to.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn solid(w: u32, h: u32, px: [u8; 4]) -> Raster {
        Raster {
            width: w,
            height: h,
            data: px.repeat((w * h) as usize),
        }
    }
    fn spec(kind: &str, direction: &str) -> TransitionSpec {
        TransitionSpec {
            kind: kind.into(),
            direction: direction.into(),
            ..Default::default()
        }
    }
    #[test]
    fn easing_table_matches_web_implementation() {
        // Same literals are asserted in tests/motion.test.ts.
        let table = [
            ("linear", 0.25, 0.25),
            ("ease-in", 0.5, 0.125),
            ("ease-out", 0.5, 0.875),
            ("ease-in-out", 0.25, 0.0625),
            ("ease-in-out", 0.75, 0.9375),
        ];
        for (kind, t, expected) in table {
            assert!((ease(kind, t) - expected).abs() < 1e-5, "{kind} {t}");
        }
        assert_eq!(ease("spring", 1.0), 1.0);
    }
    #[test]
    fn every_transition_starts_on_from_and_ends_on_to() {
        let (from, to) = (
            solid(16, 16, [200, 0, 0, 255]),
            solid(16, 16, [0, 0, 200, 255]),
        );
        for kind in ["fade", "slide", "push", "wipe", "reveal", "zoom", "blur"] {
            for direction in ["left", "right", "up", "down"] {
                let s = spec(kind, direction);
                assert_eq!(
                    compose(&from, &to, &s, 0.0).data,
                    from.data,
                    "{kind} {direction} start"
                );
                assert_eq!(
                    compose(&from, &to, &s, 1.0).data,
                    to.data,
                    "{kind} {direction} end"
                );
            }
        }
    }
    #[test]
    fn dip_passes_through_transparent_and_fade_midpoint_blends() {
        let (from, to) = (solid(4, 4, [200, 0, 0, 255]), solid(4, 4, [0, 0, 200, 255]));
        assert_eq!(compose(&from, &to, &spec("dip", "left"), 0.5).data[3], 0);
        let mid = compose(&from, &to, &spec("fade", "left"), 0.5).data;
        assert_eq!(&mid[..4], &[100, 0, 100, 255]);
    }
    #[test]
    fn straight_alpha_round_trip_keeps_opaque_pixels() {
        let r = Raster::from_straight(1, 1, &[10, 20, 30, 255]);
        assert_eq!(r.to_straight(), vec![10, 20, 30, 255]);
    }
}

#[cfg(test)]
mod budget {
    use super::*;
    #[test]
    #[ignore = "timing report: cargo test -- --ignored --nocapture"]
    fn frame_cost_1080p() {
        let px = |c: u8| Raster {
            width: 1920,
            height: 1080,
            data: [c, 0, 0, 255].repeat(1920 * 1080),
        };
        let (a, b) = (px(10), px(200));
        for kind in [
            "fade", "dip", "slide", "push", "wipe", "reveal", "zoom", "blur",
        ] {
            let spec = TransitionSpec {
                kind: kind.into(),
                ..Default::default()
            };
            let start = std::time::Instant::now();
            let out = compose(&a, &b, &spec, 0.5);
            let _ = out.to_straight();
            println!("{kind}: {:?}", start.elapsed());
        }
    }
}
