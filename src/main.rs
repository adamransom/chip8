mod app;
mod device;
mod screen;

use log::{info, LevelFilter};
use std::sync::mpsc::channel;
use std::thread;
use winit::event_loop::EventLoop;

const WINDOW_SCALE: u32 = 6;

fn main() {
    env_logger::builder()
        .filter_module("chip8", LevelFilter::Debug)
        .init();

    let (sender, receiver) = channel();

    thread::spawn(move || {
        let event = receiver.recv().unwrap();

        info!("Booting device");

        let mut device = match event {
            device::Event::On(window) => device::Device::new(window),
            _ => panic!("First event must be `On`"),
        };

        device.load("roms/1-chip8-logo.ch8");
        device.run(receiver);
    });

    let event_loop = EventLoop::new().unwrap();
    let mut app = app::App::new(WINDOW_SCALE, sender);
    event_loop.run_app(&mut app).unwrap();
}
