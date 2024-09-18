use pixels::{Pixels, SurfaceTexture};
use std::sync::Arc;
use winit::window::Window;

pub const WIDTH: u8 = 64;
pub const HEIGHT: u8 = 32;

const BUFFER_SIZE: usize = WIDTH as usize * HEIGHT as usize;

pub struct Screen {
    pixels: Pixels,
    buffer: [bool; BUFFER_SIZE],
}

impl Screen {
    pub fn new(window: Arc<Window>) -> Self {
        let surface_texture = SurfaceTexture::new(
            window.inner_size().width,
            window.inner_size().height,
            &window,
        );

        Self {
            pixels: Pixels::new(u32::from(WIDTH), u32::from(HEIGHT), surface_texture).unwrap(),
            buffer: [false; BUFFER_SIZE],
        }
    }

    pub fn clear(&mut self) {
        self.buffer = [false; BUFFER_SIZE];
    }

    pub fn draw(&mut self, x: u8, y: u8, sprite: &[u8]) -> bool {
        let mut collision = false;

        let wrapped_x = (x % WIDTH) as usize;
        let wrapped_y = (y % HEIGHT) as usize;

        for (y_row, line) in sprite.iter().enumerate() {
            let y_pos = wrapped_y + y_row;

            // clip sprites
            if y_pos > HEIGHT.into() {
                break;
            }

            for x_column in 0..8_usize {
                let pixel = line & (0x80 >> x_column);
                let x_pos = wrapped_x + x_column;

                // clip sprites
                if x_pos > WIDTH.into() {
                    break;
                }

                if pixel != 0 {
                    let index = x_pos + (y_pos * usize::from(WIDTH));

                    if self.buffer[index] {
                        collision = true
                    }

                    self.buffer[index] ^= true;
                }
            }
        }

        collision
    }

    pub fn refresh(&mut self) {
        let frame = self.pixels.frame_mut();

        for (pixel, rgba) in self.buffer.into_iter().zip(frame.chunks_exact_mut(4)) {
            if pixel {
                rgba.copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF])
            } else {
                rgba.copy_from_slice(&[0x00, 0x00, 0x00, 0xFF])
            }
        }

        self.pixels.render().unwrap();
    }
}
