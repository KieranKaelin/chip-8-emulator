const DISPLAY_WIDTH: u8 = 64;
const DISPLAY_HEIGHT: u8 = 32;

use pixels::{Pixels, SurfaceTexture};
use winit::window::Window;

pub struct Display<'a> {
    pixels: Vec<bool>,
    buffer: Pixels<'a>,
}

impl<'a> Display<'a> {
    pub fn new(window: &'a Window) -> Self {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, window);
        let buffer = Pixels::new(DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32, surface_texture)
            .expect("Failed to create pixel buffer");

        Self {
            pixels: vec![false; DISPLAY_WIDTH as usize * DISPLAY_HEIGHT as usize],
            buffer,
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, value: bool) {
        self.pixels[y * DISPLAY_WIDTH as usize + x] = value;
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> bool {
        self.pixels[y * DISPLAY_WIDTH as usize + x]
    }

    pub fn clear(&mut self) {
        self.pixels.iter_mut().for_each(|pixel| *pixel = false);
    }

    // For debugging/display purposes
    pub fn draw(&self) {
        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                print!(
                    "{}",
                    if self.get_pixel(x as usize, y as usize) {
                        "█"
                    } else {
                        " "
                    }
                );
            }
            println!();
        }
    }

    pub fn render(&mut self) {
        let frame = self.buffer.frame_mut();
        for (i, pixel) in self.pixels.iter().enumerate() {
            let rgba = if *pixel {
                [255, 255, 255, 255]
            } else {
                [0, 0, 0, 255]
            };
            let offset = i * 4;
            frame[offset..offset + 4].copy_from_slice(&rgba);
        }
        self.buffer.render().expect("Failed to render");
    }
}
