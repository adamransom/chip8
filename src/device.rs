use crate::screen::Screen;

use log::info;
use std::fs::File;
use std::io::Read;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;
use std::sync::Arc;
use winit::window::Window;

pub enum Event {
    On(Arc<Window>),
    Off,
}

pub struct Device {
    screen: Screen,
    memory: [u8; 4096],
    registers: [u8; 16],
    pc: u16,
    i: u16,
    draw_flag: bool,
}

struct Opcode {
    raw: u16,
    code: u16,
    nnn: u16,
    x: u8,
    y: u8,
    kk: u8,
    n: u8,
}

impl Device {
    pub fn new(window: Arc<Window>) -> Self {
        Self {
            screen: Screen::new(window),
            memory: [0; 4096],
            registers: [0; 16],
            i: 0,
            pc: 0x200,
            draw_flag: false,
        }
    }

    pub fn load(&mut self, path: &str) {
        info!("Loading ROM '{}'", path);

        let mut file = File::open(path).unwrap();
        let program = &mut self.memory[0x200..0xFFF];
        let bytes = file.read(program).unwrap();

        info!("Loaded {} bytes", bytes);
    }

    pub fn run(&mut self, channel: Receiver<Event>) {
        let mut timer = std::time::Instant::now();

        'outer: loop {
            let elapsed = timer.elapsed().as_secs_f64();

            if elapsed >= 1.0 / 60.0 {
                timer = std::time::Instant::now();

                let mut cycles = 0;

                // about a 720Mhz clock speed
                while cycles < 12 {
                    self.tick();
                    cycles += 1;

                    // simulate waiting for screen refresh
                    // after drawing
                    if self.draw_flag {
                        break;
                    }
                }

                self.screen.refresh();
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

    fn fetch(&mut self) -> Opcode {
        let top = (self.memory[self.pc as usize] as u16) << 8;
        let bottom = self.memory[self.pc as usize + 1];
        let raw = top | bottom as u16;

        self.pc += 2;

        Opcode {
            raw,
            code: raw & 0xF000,
            nnn: raw & 0x0FFF,
            x: ((raw & 0x0F00) >> 8) as u8,
            y: ((raw & 0x00F0) >> 4) as u8,
            n: (raw & 0x000F) as u8,
            kk: bottom,
        }
    }

    pub fn tick(&mut self) {
        self.draw_flag = false;

        let opcode = self.fetch();
        match opcode.code {
            0x0000 => match opcode.kk {
                0xE0 => self.op_00e0(),
                _ => panic!("unknown opcode {:04x}", opcode.raw),
            },
            0x1000 => self.op_1nnn(opcode.nnn),
            0x6000 => self.op_6xkk(opcode.x, opcode.kk),
            0x7000 => self.op_7xkk(opcode.x, opcode.kk),
            0xA000 => self.op_annn(opcode.nnn),
            0xD000 => self.op_dxyn(opcode.x, opcode.y, opcode.n),
            _ => panic!("unknown opcode {:04x}", opcode.raw),
        }
    }

    // Clear the display
    fn op_00e0(&mut self) {
        self.screen.clear();

        self.draw_flag = true;
    }

    // Jump to location at nnn
    fn op_1nnn(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    // Set Vx = kk
    fn op_6xkk(&mut self, x: u8, kk: u8) {
        self.registers[usize::from(x)] = kk;
    }

    // Set Vx = Vx + kk
    fn op_7xkk(&mut self, x: u8, kk: u8) {
        self.registers[usize::from(x)] += kk;
    }

    // Set I = nnn
    fn op_annn(&mut self, nnn: u16) {
        self.i = nnn;
    }

    // Display n-byte sprite starting at memory location I at (Vx, Vy)
    fn op_dxyn(&mut self, x: u8, y: u8, n: u8) {
        let x_pos = self.register(x);
        let y_pos = self.register(y);

        let sprite = &self.memory[usize::from(self.i)..usize::from(self.i + n as u16)];

        // Set flag if collision detected
        if self.screen.draw(x_pos, y_pos, sprite) {
            self.set_flag(1);
        }

        self.draw_flag = true;
    }

    fn register(&mut self, index: u8) -> u8 {
        self.registers[usize::from(index)]
    }

    fn set_flag(&mut self, value: u8) {
        self.registers[0xF] = value;
    }
}
