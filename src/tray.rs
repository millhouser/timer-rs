use crate::TimerTray;
use font8x8::UnicodeFonts;

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
    let width = buffer.width() as usize;
    let height = buffer.height() as usize;
    let label = minutes.to_string();
    let scale = (width / (label.len() * 8 + 8)).min(height / 16).max(1);
    let left = width.saturating_sub(label.len() * 8 * scale) / 2;
    let top = height.saturating_sub(8 * scale) / 2;
    let pixels = buffer.make_mut_slice();
    for (index, digit) in label.chars().enumerate() {
        if let Some(glyph) = font8x8::BASIC_FONTS.get(digit) {
            for (row, bits) in glyph.iter().enumerate() {
                for column in 0..8 {
                    if bits & (1 << column) != 0 {
                        for offset_y in 0..scale {
                            for offset_x in 0..scale {
                                let x = left + (index * 8 + column) * scale + offset_x;
                                let y = top + row * scale + offset_y;
                                if x < width && y < height {
                                    pixels[y * width + x] = slint::Rgba8Pixel::new(0, 0, 255, 255);
                                }
                            }
                        }
                    }
                }
            }
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
