use crate::TimerTray;
use ab_glyph::{Font, Point, PxScale, ScaleFont};

const FONT_DATA: &[u8] = include_bytes!("../assets/font.ttf");

pub struct Tray {
    pub ui: TimerTray,
    base_icon: slint::Image,
    minutes: u32,
}

impl Tray {
    pub fn new() -> Result<Self, slint::PlatformError> {
        let ui = TimerTray::new()?;
        let base_icon = ui.get_tray_image();
        Ok(Self {
            ui,
            base_icon,
            minutes: 0,
        })
    }

    pub fn set_running(&mut self, running: bool) {
        self.ui.set_timer_running(running);
    }

    pub fn set_custom_minutes(&self, minutes: u32) {
        self.ui.set_custom_minutes(minutes as i32);
    }

    pub fn set_tooltip(&mut self, text: &str) {
        self.ui.set_status(text.into());
    }

    pub fn notify(&self, message: &str, warning: bool) {
        crate::desktop::notify(message, warning);
    }

    pub fn repaint(&mut self, seconds: u32) {
        let minutes = seconds.div_ceil(60);
        if self.minutes != minutes {
            self.ui
                .set_tray_image(numbered_icon(&self.base_icon, minutes));
            self.minutes = minutes;
        }
    }
}

fn numbered_icon(base: &slint::Image, minutes: u32) -> slint::Image {
    if minutes == 0 {
        return base.clone();
    }
    let Some(mut buffer) = base.to_rgba8() else {
        return base.clone();
    };
    let Ok(font) = ab_glyph::FontRef::try_from_slice(FONT_DATA) else {
        return base.clone();
    };

    let width = buffer.width() as f32;
    let height = buffer.height() as f32;
    let label = minutes.to_string();

    let font_size = if label.len() <= 1 {
        height * 0.52
    } else if label.len() == 2 {
        height * 0.44
    } else {
        height * 0.85 / label.len() as f32
    };

    let scale = PxScale::from(font_size);
    let scaled = font.as_scaled(scale);

    let mut glyphs = Vec::with_capacity(label.len());
    let mut total_w = 0.0f32;
    for c in label.chars() {
        let id = font.glyph_id(c);
        let adv = scaled.h_advance(id);
        glyphs.push((id, total_w));
        total_w += adv;
    }

    let start_x = (width - total_w) / 2.0;
    let baseline_y = (height + scaled.ascent() * 0.76) / 2.0;

    let w_usize = buffer.width() as usize;
    let h_usize = buffer.height() as usize;
    let pixels = buffer.make_mut_slice();

    let halo_radius = (height / 40.0).round().max(1.0) as i32;

    // Subtle white halo for crisp contrast against clock hands and dial
    for (id, offset_x) in &glyphs {
        for dy in -halo_radius..=halo_radius {
            for dx in -halo_radius..=halo_radius {
                if dx == 0 && dy == 0 || dx * dx + dy * dy > halo_radius * halo_radius + 1 {
                    continue;
                }
                let g = id.with_scale_and_position(
                    scale,
                    Point {
                        x: start_x + offset_x + dx as f32,
                        y: baseline_y + dy as f32,
                    },
                );
                if let Some(og) = font.outline_glyph(g) {
                    let bounds = og.px_bounds();
                    og.draw(|gx, gy, c| {
                        let px = (bounds.min.x as i32 + gx as i32) as usize;
                        let py = (bounds.min.y as i32 + gy as i32) as usize;
                        if px < w_usize && py < h_usize {
                            let idx = py * w_usize + px;
                            let p = &mut pixels[idx];
                            let factor = c * 0.85;
                            let r = (p.r as f32 * (1.0 - factor) + 255.0 * factor).round() as u8;
                            let g = (p.g as f32 * (1.0 - factor) + 255.0 * factor).round() as u8;
                            let b = (p.b as f32 * (1.0 - factor) + 255.0 * factor).round() as u8;
                            *p = slint::Rgba8Pixel::new(r, g, b, 255);
                        }
                    });
                }
            }
        }
    }

    // Smooth anti-aliased blue text
    for (id, offset_x) in glyphs {
        let g = id.with_scale_and_position(
            scale,
            Point {
                x: start_x + offset_x,
                y: baseline_y,
            },
        );
        if let Some(og) = font.outline_glyph(g) {
            let bounds = og.px_bounds();
            og.draw(|gx, gy, c| {
                let px = (bounds.min.x as i32 + gx as i32) as usize;
                let py = (bounds.min.y as i32 + gy as i32) as usize;
                if px < w_usize && py < h_usize {
                    let idx = py * w_usize + px;
                    let p = &mut pixels[idx];
                    let r = (p.r as f32 * (1.0 - c) + 0.0 * c).round() as u8;
                    let g = (p.g as f32 * (1.0 - c) + 0.0 * c).round() as u8;
                    let b = (p.b as f32 * (1.0 - c) + 255.0 * c).round() as u8;
                    *p = slint::Rgba8Pixel::new(r, g, b, 255);
                }
            });
        }
    }

    slint::Image::from_rgba8(buffer)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minute_overlay_preserves_size_and_resets_to_base() {
        let base = slint::Image::from_rgba8(slint::SharedPixelBuffer::new(32, 32));
        let original = base.to_rgba8().unwrap();
        assert_eq!(
            numbered_icon(&base, 0).to_rgba8().unwrap().as_bytes(),
            original.as_bytes()
        );
        for minutes in [1, 10, 11, 99] {
            let icon = numbered_icon(&base, minutes);
            let pixels = icon.to_rgba8().unwrap();
            assert_eq!(icon.size(), base.size());
            assert!(
                pixels
                    .as_slice()
                    .iter()
                    .any(|pixel| pixel.b == 255 && pixel.a == 255)
            );
            assert_ne!(pixels.as_bytes(), original.as_bytes());
        }
        assert_ne!(
            numbered_icon(&base, 1).to_rgba8().unwrap().as_bytes(),
            numbered_icon(&base, 2).to_rgba8().unwrap().as_bytes()
        );
    }
}
