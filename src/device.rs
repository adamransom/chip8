use pixels::{Pixels, SurfaceTexture};
use rand::prelude::*;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;
use std::sync::Arc;
use winit::window::Window;

pub const SCREEN_WIDTH: u32 = 64;
pub const SCREEN_HEIGHT: u32 = 32;

pub enum Event {
    On(Arc<Window>),
    Off,
}

pub struct Device {
    pixels: Pixels,
}

impl Device {
    pub fn new(window: Arc<Window>) -> Self {
        let surface_texture = SurfaceTexture::new(
            window.inner_size().width,
            window.inner_size().height,
            &window,
        );

        Self {
            pixels: Pixels::new(SCREEN_WIDTH, SCREEN_HEIGHT, surface_texture).unwrap(),
        }
    }

    pub fn run(&mut self, channel: Receiver<Event>) {
        let mut timer = std::time::Instant::now();

        'outer: loop {
            let elapsed = timer.elapsed().as_secs_f64();

            if elapsed >= 1.0 / 60.0 {
                timer = std::time::Instant::now();

                self.update();
                self.render();
            }

            'events: loop {
                match channel.try_recv() {
                    Ok(event) => match event {
                        Event::Off => break 'outer,
                        Event::On(_) => panic!("Should never receive `On`"),
                    },
                    Err(TryRecvError::Empty) => break 'events,
                    Err(TryRecvError::Disconnected) => break 'outer,
                }
            }
        }
    }

    pub fn render(&mut self) {
        let mut rng = thread_rng();
        for pixel in self.pixels.frame_mut().chunks_exact_mut(4) {
            let rgba = [rng.gen(), rng.gen(), rng.gen(), 0xff];
            pixel.copy_from_slice(&rgba);
        }
        self.pixels.render().unwrap();
    }

    pub fn update(&self) {}
}
