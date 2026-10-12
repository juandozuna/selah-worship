pub mod composite;
pub mod media;
pub mod transition;
pub use composite::{blit_layer, compose_stack, FitMode, LayerBlit, RgbaImage};
use selah_domain::{LiveSnapshot, TextAnimSpec, TextRegion};
use std::sync::Arc;
use transition::{ease, Raster};

pub struct Renderer {
    fontdb: Arc<resvg::usvg::fontdb::Database>,
    device: wgpu::Device,
    queue: wgpu::Queue,
}
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub png: Vec<u8>,
    pub texture: wgpu::Texture,
}
fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
impl Renderer {
    pub fn new() -> Result<Self, String> {
        let mut fontdb = resvg::usvg::fontdb::Database::new();
        fontdb.load_system_fonts();
        let instance = wgpu::Instance::default();
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .ok_or("No compatible graphics adapter found")?;
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Selah renderer"),
                required_features: wgpu::Features::empty(),
                required_limits:
                    wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
            },
            None,
        ))
        .map_err(|e| e.to_string())?;
        Ok(Self {
            fontdb: Arc::new(fontdb),
            device,
            queue,
        })
    }
    #[allow(clippy::too_many_arguments)]
    fn animated_lines(
        &self,
        svg: &mut String,
        region: usize,
        r: &TextRegion,
        lines: &[String],
        start: f32,
        line_height: f32,
        spec: &TextAnimSpec,
        ms: f32,
    ) {
        // (text, left x, baseline y, index within the stagger sequence)
        let mut units: Vec<(String, f32, f32, usize)> = Vec::new();
        let mut counter = 0usize;
        for (n, line) in lines.iter().enumerate() {
            let y = start + n as f32 * line_height + r.font_size * 0.35;
            let lw = self.text_width(line, r);
            let left = match r.align.as_str() {
                "left" => r.x + 4.0,
                "right" => r.x + r.width - 4.0 - lw,
                _ => r.x + r.width / 2.0 - lw / 2.0,
            };
            match spec.unit.as_str() {
                "line" => {
                    units.push((line.clone(), left, y, counter));
                    counter += 1;
                }
                "word" | "letter" => {
                    let letters = spec.unit == "letter";
                    let mut chars = line.char_indices().peekable();
                    while let Some((b, c)) = chars.next() {
                        if c.is_whitespace() {
                            continue;
                        }
                        let mut end = b + c.len_utf8();
                        if !letters {
                            while let Some(&(nb, nc)) = chars.peek() {
                                if nc.is_whitespace() {
                                    break;
                                }
                                end = nb + nc.len_utf8();
                                chars.next();
                            }
                        }
                        let x = left + self.text_width(&line[..b], r);
                        units.push((line[b..end].to_string(), x, y, counter));
                        counter += 1;
                    }
                }
                _ => units.push((line.clone(), left, y, 0)),
            }
        }
        let count = if spec.unit == "block" {
            1
        } else {
            counter.max(1)
        };
        let ranks = unit_ranks(count, &spec.order);
        let dist = r.font_size * 0.6;
        let font = escape(&r.font_family);
        for (k, (text, x, y, index)) in units.iter().enumerate() {
            let delay = ranks[*index] as f32 * spec.stagger_ms as f32;
            let p = ((ms - delay) / spec.duration_ms as f32).clamp(0.0, 1.0);
            let e = ease(&spec.easing, p);
            let (mut opacity, mut dx, mut dy, mut blur) = (e.clamp(0.0, 1.0), 0.0, 0.0, 0.0);
            match spec.kind.as_str() {
                "typewriter" => opacity = if ms >= delay { 1.0 } else { 0.0 },
                "blur" => blur = (1.0 - e).max(0.0) * r.font_size * 0.25,
                "fade" => {}
                _ => match spec.direction.as_str() {
                    "left" => dx = (1.0 - e) * dist,
                    "right" => dx = -(1.0 - e) * dist,
                    "down" => dy = -(1.0 - e) * dist,
                    _ => dy = (1.0 - e) * dist,
                },
            }
            if opacity <= 0.001 {
                continue;
            }
            let filter = if blur > 0.5 {
                svg.push_str(&format!("<defs><filter id='b{region}_{k}' x='-30%' y='-60%' width='160%' height='220%'><feGaussianBlur stdDeviation='{blur}'/></filter></defs>"));
                format!(" filter='url(#b{region}_{k})'")
            } else {
                String::new()
            };
            let text = escape(text);
            svg.push_str(&format!(
                "<g opacity='{opacity}' transform='translate({dx} {dy})'{filter}>"
            ));
            if r.shadow {
                svg.push_str(&format!("<text x='{x}' y='{}' font-family='{font}' font-size='{}' font-weight='600' fill='#000000' fill-opacity='.6'>{text}</text>", y + 3.0, r.font_size));
            }
            svg.push_str(&format!("<text x='{x}' y='{y}' font-family='{font}' font-size='{}' font-weight='600' fill='{}' stroke='#000000' stroke-width='{}' stroke-linejoin='round' paint-order='stroke'>{text}</text></g>", r.font_size, r.color, r.outline * 2.0));
        }
    }
    fn text_width(&self, text: &str, r: &selah_domain::TextRegion) -> f32 {
        use resvg::usvg::fontdb::{Family, Query, Weight};
        let families = [Family::Name(&r.font_family), Family::SansSerif];
        let Some(id) = self.fontdb.query(&Query {
            families: &families,
            weight: Weight(600),
            ..Default::default()
        }) else {
            return text.chars().count() as f32 * r.font_size * 0.55;
        };
        self.fontdb
            .with_face_data(id, |data, index| {
                let Some(face) = rustybuzz::Face::from_slice(data, index) else {
                    return 0.0;
                };
                let mut buffer = rustybuzz::UnicodeBuffer::new();
                buffer.push_str(text);
                buffer.guess_segment_properties();
                let glyphs = rustybuzz::shape(&face, &[], buffer);
                let advance: i32 = glyphs.glyph_positions().iter().map(|p| p.x_advance).sum();
                advance as f32 / face.units_per_em() as f32 * r.font_size
            })
            .unwrap_or(0.0)
    }
    /// Final (settled) frame with PNG and GPU texture.
    pub fn render(&self, live: &LiveSnapshot) -> Result<Frame, String> {
        let pixmap = self.rasterize(live, None)?;
        self.finish(live, pixmap)
    }
    /// Premultiplied pixels at `anim_ms` into the text animation (`None` = settled).
    pub fn render_raster(
        &self,
        live: &LiveSnapshot,
        anim_ms: Option<f32>,
    ) -> Result<Raster, String> {
        let pixmap = self.rasterize(live, anim_ms)?;
        Ok(Raster {
            width: pixmap.width(),
            height: pixmap.height(),
            data: pixmap.data().to_vec(),
        })
    }
    /// Longest text animation among the regions of this snapshot, in milliseconds.
    pub fn text_anim_total_ms(live: &LiveSnapshot) -> f32 {
        let Some(block) = &live.block else { return 0.0 };
        live.layout
            .regions
            .iter()
            .filter_map(|r| {
                let spec = effective_anim(live, r)?;
                let text = region_text(block, r);
                let n = unit_count(text, &spec) as f32;
                Some((n - 1.0).max(0.0) * spec.stagger_ms as f32 + spec.duration_ms as f32)
            })
            .fold(0.0, f32::max)
    }
    fn rasterize(
        &self,
        live: &LiveSnapshot,
        anim_ms: Option<f32>,
    ) -> Result<resvg::tiny_skia::Pixmap, String> {
        live.layout.validate()?;
        let l = &live.layout;
        if l.width > self.device.limits().max_texture_dimension_2d
            || l.height > self.device.limits().max_texture_dimension_2d
        {
            return Err("Canvas exceeds this graphics adapter's texture limit".into());
        }
        let mut svg = format!(
            "<svg xmlns='http://www.w3.org/2000/svg' width='{}' height='{}'>",
            l.width, l.height
        );
        if live.test_pattern {
            for (i, color) in [
                "#ffffff", "#e5ce57", "#57cbce", "#57ce70", "#cd57ce", "#ce5757", "#5768ce",
            ]
            .iter()
            .enumerate()
            {
                svg.push_str(&format!(
                    "<rect x='{}' y='0' width='{}' height='{}' fill='{color}'/>",
                    i as f32 * l.width as f32 / 7.0,
                    l.width as f32 / 7.0 + 1.0,
                    l.height
                ));
            }
        } else if let Some(block) = &live.block {
            for (i, r) in l.regions.iter().enumerate() {
                let text = region_text(block, r);
                let lines = wrap(text, r.width - r.outline * 2.0 - 8.0, |line| {
                    self.text_width(line, r)
                });
                let height = r.font_size * 1.25;
                let start =
                    r.y + (r.height - (lines.len().saturating_sub(1)) as f32 * height) / 2.0;
                let (anchor, x) = match r.align.as_str() {
                    "left" => ("start", r.x + 4.0),
                    "right" => ("end", r.x + r.width - 4.0),
                    _ => ("middle", r.x + r.width / 2.0),
                };
                svg.push_str(&format!("<defs><clipPath id='clip{i}'><rect x='{}' y='{}' width='{}' height='{}'/></clipPath></defs><g clip-path='url(#clip{i})'>",r.x,r.y,r.width,r.height));
                let anim = effective_anim(live, r).zip(anim_ms);
                if let Some((spec, ms)) = anim.filter(|(spec, ms)| *ms < anim_total(spec, text)) {
                    self.animated_lines(&mut svg, i, r, &lines, start, height, &spec, ms);
                } else {
                    for (n, line) in lines.iter().enumerate() {
                        let y = start + n as f32 * height + r.font_size * 0.35;
                        let font = escape(&r.font_family);
                        let line = escape(line);
                        if r.shadow {
                            svg.push_str(&format!("<text x='{x}' y='{}' text-anchor='{anchor}' font-family='{font}' font-size='{}' font-weight='600' fill='#000000' fill-opacity='.6'>{line}</text>",y+3.0,r.font_size));
                        }
                        svg.push_str(&format!("<text x='{x}' y='{y}' text-anchor='{anchor}' font-family='{font}' font-size='{}' font-weight='600' fill='{}' stroke='#000000' stroke-width='{}' stroke-linejoin='round' paint-order='stroke'>{line}</text>",r.font_size,r.color,r.outline*2.0));
                    }
                }
                svg.push_str("</g>");
            }
        }
        svg.push_str("</svg>");
        let options = resvg::usvg::Options {
            fontdb: self.fontdb.clone(),
            ..Default::default()
        };
        let tree = resvg::usvg::Tree::from_str(&svg, &options).map_err(|e| e.to_string())?;
        let mut pixmap =
            resvg::tiny_skia::Pixmap::new(l.width, l.height).ok_or("Could not allocate frame")?;
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::identity(),
            &mut pixmap.as_mut(),
        );
        Ok(pixmap)
    }
    fn finish(
        &self,
        live: &LiveSnapshot,
        pixmap: resvg::tiny_skia::Pixmap,
    ) -> Result<Frame, String> {
        let l = &live.layout;
        let png = pixmap.encode_png().map_err(|e| e.to_string())?;
        let mut rgba = pixmap.data().to_vec(); // tiny-skia is premultiplied; NDI and the public frame contract use straight alpha.
        for p in rgba.chunks_exact_mut(4) {
            let a = p[3] as u32;
            if a > 0 && a < 255 {
                for channel in &mut p[..3] {
                    *channel = ((*channel as u32 * 255 + a / 2) / a).min(255) as u8;
                }
            }
        }
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Selah canvas"),
            size: wgpu::Extent3d {
                width: l.width,
                height: l.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(l.width * 4),
                rows_per_image: Some(l.height),
            },
            wgpu::Extent3d {
                width: l.width,
                height: l.height,
                depth_or_array_layers: 1,
            },
        );
        Ok(Frame {
            width: l.width,
            height: l.height,
            rgba,
            png,
            texture,
        })
    }
}

fn region_text<'a>(block: &'a selah_domain::PresentationBlock, r: &TextRegion) -> &'a str {
    match r.binding.as_str() {
        "secondary" => block.secondary.as_deref().unwrap_or(""),
        "reference" => block.reference.as_deref().unwrap_or(""),
        _ => &block.primary,
    }
}
/// Region override (or resolved default) with preset kinds expanded to a unit and direction.
fn effective_anim(live: &LiveSnapshot, r: &TextRegion) -> Option<TextAnimSpec> {
    let mut spec = live.motion.as_ref()?.text.clone();
    if let Some(kind) = &r.anim {
        spec.kind = kind.clone();
    }
    match spec.kind.as_str() {
        "none" => return None,
        "typewriter" => spec.unit = "letter".into(),
        "word-rise" => {
            spec.unit = "word".into();
            spec.direction = "up".into();
        }
        "line-cascade" => {
            spec.unit = "line".into();
            spec.direction = "up".into();
        }
        _ => {}
    }
    Some(spec)
}
fn unit_count(text: &str, spec: &TextAnimSpec) -> usize {
    match spec.unit.as_str() {
        "line" => text.lines().count().max(1),
        "word" => text.split_whitespace().count().max(1),
        "letter" => text.chars().filter(|c| !c.is_whitespace()).count().max(1),
        _ => 1,
    }
}
fn anim_total(spec: &TextAnimSpec, text: &str) -> f32 {
    (unit_count(text, spec) as f32 - 1.0).max(0.0) * spec.stagger_ms as f32
        + spec.duration_ms as f32
}
/// Position of each unit in the stagger sequence; mirrored by `unitRanks` in `src/motion.ts`.
fn unit_ranks(n: usize, order: &str) -> Vec<usize> {
    let key = |i: usize| -> u64 {
        match order {
            "backward" => (n - 1 - i) as u64,
            "center" => ((i as f32 - (n as f32 - 1.0) / 2.0).abs() * 2.0) as u64 * 1000 + i as u64,
            "random" => (i as u64).wrapping_mul(2654435761) % 1_000_003 * 1000 + i as u64,
            _ => i as u64,
        }
    };
    let mut idx: Vec<usize> = (0..n).collect();
    idx.sort_by_key(|&i| key(i));
    let mut rank = vec![0; n];
    for (r, i) in idx.into_iter().enumerate() {
        rank[i] = r;
    }
    rank
}
fn wrap(text: &str, width: f32, measure: impl Fn(&str) -> f32) -> Vec<String> {
    let mut result = Vec::new();
    for paragraph in text.lines() {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            let candidate = if line.is_empty() {
                word.to_string()
            } else {
                format!("{line} {word}")
            };
            if !line.is_empty() && measure(&candidate) > width {
                result.push(std::mem::take(&mut line));
                line = word.into();
            } else {
                line = candidate;
            }
        }
        result.push(line);
    }
    result
}

#[cfg(test)]
mod motion_tests {
    use super::*;
    #[test]
    fn ranks_follow_order() {
        assert_eq!(unit_ranks(4, "forward"), vec![0, 1, 2, 3]);
        assert_eq!(unit_ranks(4, "backward"), vec![3, 2, 1, 0]);
        let center = unit_ranks(5, "center");
        assert_eq!(center[2], 0);
        let mut random = unit_ranks(6, "random");
        random.sort();
        assert_eq!(random, vec![0, 1, 2, 3, 4, 5]);
    }
}

#[cfg(test)]
mod render_tests {
    use super::*;
    use selah_domain::{CanvasLayout, MotionSettings, PresentationBlock, TextAnimSpec};
    fn live(kind: &str) -> LiveSnapshot {
        let region = TextRegion {
            id: "p".into(),
            name: "L".into(),
            x: 0.0,
            y: 0.0,
            width: 640.0,
            height: 200.0,
            font_size: 60.0,
            font_family: "Arial".into(),
            color: "#ffffff".into(),
            align: "center".into(),
            binding: "primary".into(),
            outline: 0.0,
            shadow: false,
            anim: None,
        };
        LiveSnapshot {
            song_id: None,
            block_index: None,
            test_pattern: false,
            ai_translation: false,
            block: Some(PresentationBlock {
                id: "b".into(),
                section: "V".into(),
                primary: "Amazing grace\nhow sweet".into(),
                secondary: None,
                reference: None,
                motion: None,
            }),
            layout: CanvasLayout {
                width: 640,
                height: 200,
                regions: vec![region],
                media_layers: vec![],
            },
            motion: Some(MotionSettings {
                text: TextAnimSpec {
                    kind: kind.into(),
                    unit: "word".into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
            media: vec![],
            stream_composite: false,
        }
    }
    fn ink(r: &Raster) -> u64 {
        r.data.chunks_exact(4).map(|p| p[3] as u64).sum()
    }
    #[test]
    fn animated_text_grows_into_the_settled_frame() {
        let Ok(renderer) = Renderer::new() else {
            return;
        }; // no graphics adapter on this machine
        for kind in [
            "fade",
            "slide",
            "blur",
            "typewriter",
            "word-rise",
            "line-cascade",
        ] {
            let live = live(kind);
            let total = Renderer::text_anim_total_ms(&live);
            assert!(total > 0.0, "{kind}");
            let start = ink(&renderer.render_raster(&live, Some(0.0)).unwrap());
            let mid = ink(&renderer.render_raster(&live, Some(total * 0.5)).unwrap());
            let end = ink(&renderer.render_raster(&live, Some(total)).unwrap());
            let settled = ink(&renderer.render_raster(&live, None).unwrap());
            assert!(start < mid && mid <= end, "{kind}: {start} {mid} {end}");
            assert_eq!(end, settled, "{kind} ends on the settled frame");
        }
    }
}
