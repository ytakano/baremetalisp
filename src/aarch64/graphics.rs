use super::mbox;

pub struct Display {
    pub size_phy: (u32, u32),
    pub size_virt: (u32, u32),
    pub offset: (u32, u32),
    pub depth: u32,            // bits per pixel
    pub pitch: u32,            // bytes per line
    pub ptr: u32,              // base address of frame buffer
    pub buffer: &'static mut [u8], // frame buffer
}

impl Display {
    pub fn set_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8) {
        let pos = (y * self.pitch + x * (self.depth >> 3)) as usize;
        self.buffer[pos + 0] = r;
        self.buffer[pos + 1] = g;
        self.buffer[pos + 2] = b;
    }

    pub fn plot_mandelbrot_set(&mut self) {
        let size = 2;
        let width = self.size_virt.0;
        let height = self.size_virt.1;
        let offset_x = -250.0;
        for x in 0..width {
            let cr = ((x as f32 + offset_x) * size as f32) / width as f32 - (size / 2) as f32; // real number
            for y in 0..height {
                let ci = (y * size) as f32 / height as f32 - (size / 2) as f32; // imaginal number
                let mut a = 0.0;
                let mut b = 0.0;
                let mut n = 0;
                for i in 0..50 {
                    let a2 = a * a - b * b + cr;
                    let b2 = 2.0 * a * b + ci;
                    a = a2;
                    b = b2;
                    n = i;
                    if a * a + b * b > 4.0 {
                        break;
                    }
                }
                if a * a + b * b > 4.0 {
                    let v = n as f32 / 50.0;
                    let rgb = hsv2rgb(1.0 - v, 1.0, 1.0);
                    self.set_pixel(x, y, rgb.0, rgb.1, rgb.2);
                } else {
                    self.set_pixel(x, y, 0, 0, 0);
                }
            }
        }
    }
}

// Set screen resolution to 1024x768
pub fn init() -> Option<Display> {
    mbox::set_display(1024, 768, 1024, 768, 0, 0)
}

pub fn hsv2rgb(mut h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let mut r = v;
    let mut g = v;
    let mut b = v;
    if s > 0.0 {
        h *= 6.0;
        let i = h as u16;
        let f = h - i as f32;
        match i {
            0 => {
                g *= 1.0 - s * (1.0 - f);
                b *= 1.0 - s;
            }
            1 => {
                r *= 1.0 - s * f;
                b *= 1.0 - s;
            }
            2 => {
                r *= 1.0 - s;
                b *= 1.0 - s * (1.0 - f);
            }
            3 => {
                r *= 1.0 - s;
                g *= 1.0 - s * f;
            }
            4 => {
                r *= 1.0 - s * (1.0 - f);
                g *= 1.0 - s;
            }
            5 => {
                g *= 1.0 - s;
                b *= 1.0 - s * f;
            }
            _ => {}
        }
    }

    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}