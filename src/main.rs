mod app;
mod device;

use std::sync::mpsc::channel;
use std::thread;
use winit::event_loop::EventLoop;

fn main() {
    let (sender, receiver) = channel();

    thread::spawn(move || {
        let event = receiver.recv().unwrap();
        let mut device = match event {
            device::Event::On(window) => device::Device::new(window),
            _ => panic!("First event must be `On`"),
        };

        device.run(receiver);
    });

    let event_loop = EventLoop::new().unwrap();
    let mut app = app::App::new(6, sender);
    event_loop.run_app(&mut app).unwrap();
}
