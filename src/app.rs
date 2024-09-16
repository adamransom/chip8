use crate::device;

use std::sync::mpsc::Sender;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowButtons, WindowId};

pub struct App {
    channel: Sender<device::Event>,
    window: Option<Arc<Window>>,
    scale: u32,
}

impl App {
    pub fn new(scale: u32, channel: Sender<device::Event>) -> Self {
        Self {
            window: None,
            channel,
            scale,
        }
    }

    fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Arc<Window> {
        let window_size = LogicalSize::new(
            device::SCREEN_WIDTH * self.scale,
            device::SCREEN_HEIGHT * self.scale,
        );

        let window_attributes = WindowAttributes::default()
            .with_title("CHIP8")
            .with_inner_size(window_size)
            .with_resizable(false)
            .with_enabled_buttons(WindowButtons::CLOSE | WindowButtons::MINIMIZE);

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());

        window.clone()
    }

    fn send_event(&self, event: device::Event) {
        self.channel.send(event).unwrap();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = self.create_window(event_loop);

        self.send_event(device::Event::On(window));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.send_event(device::Event::Off);
                event_loop.exit();
            }
            _ => (),
        }
    }
}
