use core::slice;

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

    pub fn draw_mandelbrot_set(&mut self) {
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

extern {
    fn lfb_init(size_phy_x: *mut u32, size_phy_y: *mut u32,
                size_virt_x: *mut u32, size_virt_y: *mut u32,
                offset_x: *mut u32, offset_y: *mut u32,
                depth: *mut u32,
                pitch: *mut u32,
                ptr: *mut u32) -> i32;
}

pub fn init() -> Option<Display> {
    let mut size_phy_x:  u32 = 1024;
    let mut size_phy_y:  u32 = 768;
    let mut size_virt_x: u32 = 1024;
    let mut size_virt_y: u32 = 768;
    let mut offset_x:    u32 = 0;
    let mut offset_y:    u32 = 0;
    let mut depth:       u32 = 32;
    let mut pitch:       u32 = 0;
    let mut ptr:         u32 = 0;

    let result = unsafe {
        lfb_init(&mut size_phy_x as *mut u32, &mut size_phy_y as *mut u32,
                 &mut size_virt_x as *mut u32, &mut size_virt_y as *mut u32,
                 &mut offset_x as *mut u32, &mut offset_y as *mut u32,
                 &mut depth as *mut u32,
                 &mut pitch as *mut u32,
                 &mut ptr as *mut u32) };

    if result == 1 {
        let slice = unsafe {
            slice::from_raw_parts_mut(ptr as *mut u8,
                                      pitch as usize * size_virt_y as usize) };

        Some(Display {
            size_phy: (size_phy_x, size_phy_y),
            size_virt: (size_virt_x, size_virt_y),
            offset: (offset_x, offset_y),
            depth: depth,
            pitch: pitch,
            ptr: ptr,
            buffer: slice
        })
    } else {
        None
    }
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