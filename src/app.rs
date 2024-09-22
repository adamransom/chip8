use crate::device;
use crate::screen;

use log::info;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
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
            u32::from(screen::WIDTH) * self.scale,
            u32::from(screen::HEIGHT) * self.scale,
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

    fn physical_to_chip8_key(&self, key: PhysicalKey) -> Option<u8> {
        match key {
            PhysicalKey::Code(code) => match code {
                KeyCode::Digit1 => Some(0x1),
                KeyCode::Digit2 => Some(0x2),
                KeyCode::Digit3 => Some(0x3),
                KeyCode::Digit4 => Some(0xc),
                KeyCode::KeyQ => Some(0x4),
                KeyCode::KeyW => Some(0x5),
                KeyCode::KeyE => Some(0x6),
                KeyCode::KeyR => Some(0xD),
                KeyCode::KeyA => Some(0x7),
                KeyCode::KeyS => Some(0x8),
                KeyCode::KeyD => Some(0x9),
                KeyCode::KeyF => Some(0xE),
                KeyCode::KeyZ => Some(0xA),
                KeyCode::KeyX => Some(0x0),
                KeyCode::KeyC => Some(0xB),
                KeyCode::KeyV => Some(0xF),
                _ => None,
            },
            _ => None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        info!("Creating window");

        let window = self.create_window(event_loop);

        self.send_event(device::Event::On(window));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key,
                        state,
                        repeat: false,
                        ..
                    },
                ..
            } => {
                if let Some(mapped_key) = self.physical_to_chip8_key(physical_key) {
                    let pressed = match state {
                        ElementState::Pressed => true,
                        ElementState::Released => false,
                    };

                    self.send_event(device::Event::Key(mapped_key, pressed));
                }
            }
            WindowEvent::CloseRequested => {
                self.send_event(device::Event::Off);
                event_loop.exit();
            }
            _ => (),
        }
    }
}
