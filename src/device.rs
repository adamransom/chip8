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
    Key(u8, bool),
    Off,
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

pub struct Device {
    window: Arc<Window>,
    screen: Screen,
    memory: [u8; 4096],
    registers: [u8; 16],
    stack: [u16; 16],
    keys: [bool; 16],
    pc: u16,
    sp: usize,
    i: u16,
    dt: u8,
    st: u8,
    wait_key: u8,
    draw_flag: bool,
}

impl Device {
    pub fn new(window: Arc<Window>) -> Self {
        Self {
            window: window.clone(),
            screen: Screen::new(window),
            memory: [0; 4096],
            registers: [0; 16],
            stack: [0; 16],
            keys: [false; 16],
            pc: 0x200,
            sp: 0,
            i: 0,
            dt: 0,
            st: 0,
            wait_key: 0xFF,
            draw_flag: false,
        }
    }

    pub fn load(&mut self, path: &str) {
        info!("Loading ROM '{}'", path);

        let mut file = File::open(path).unwrap();
        let program = &mut self.memory[0x200..0xFFF];
        let bytes = file.read(program).unwrap();

        info!("Loaded {} bytes", bytes);

        self.memory[..Self::FONT.len()].copy_from_slice(&Self::FONT);
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
                    // simulate blocking execution until
                    // key is pressed
                    if self.wait_key != 0xFF {
                        break;
                    }

                    self.tick();
                    cycles += 1;

                    // simulate waiting for screen refresh
                    // after drawing
                    if self.draw_flag {
                        break;
                    }
                }

                self.handle_delay();
                self.handle_sound();
                self.screen.refresh();
            }

            'events: loop {
                match channel.try_recv() {
                    Ok(event) => match event {
                        Event::Key(key, pressed) => self.handle_key(key, pressed),
                        Event::Off => break 'outer,
                        Event::On(_) => panic!("Should never receive `On`"),
                    },
                    Err(TryRecvError::Empty) => break 'events,
                    Err(TryRecvError::Disconnected) => break 'outer,
                }
            }
        }
    }

    fn handle_delay(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }
    }

    fn handle_sound(&mut self) {
        if self.st > 0 {
            self.window.set_title("🔊");
            self.st -= 1;
        } else {
            self.window.set_title("CHIP8");
        }
    }

    fn handle_key(&mut self, key: u8, pressed: bool) {
        self.keys[usize::from(key)] = pressed;

        if self.wait_key != 0xFF && !pressed {
            self.registers[usize::from(self.wait_key)] = key;
            self.wait_key = 0xFF;
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

    fn tick(&mut self) {
        self.draw_flag = false;

        let opcode = self.fetch();

        match opcode.code {
            0x0000 => match opcode.kk {
                0xEE => self.op_00ee(),
                0xE0 => self.op_00e0(),
                0x00 => {}
                _ => panic!("unknown opcode {:04x}", opcode.raw),
            },
            0x1000 => self.op_1nnn(opcode.nnn),
            0x2000 => self.op_2nnn(opcode.nnn),
            0x3000 => self.op_3xkk(opcode.x, opcode.kk),
            0x4000 => self.op_4xkk(opcode.x, opcode.kk),
            0x5000 => self.op_5xy0(opcode.x, opcode.y),
            0x6000 => self.op_6xkk(opcode.x, opcode.kk),
            0x7000 => self.op_7xkk(opcode.x, opcode.kk),
            0x8000 => match opcode.n {
                0x0 => self.op_8xy0(opcode.x, opcode.y),
                0x1 => self.op_8xy1(opcode.x, opcode.y),
                0x2 => self.op_8xy2(opcode.x, opcode.y),
                0x3 => self.op_8xy3(opcode.x, opcode.y),
                0x4 => self.op_8xy4(opcode.x, opcode.y),
                0x5 => self.op_8xy5(opcode.x, opcode.y),
                0x6 => self.op_8xy6(opcode.x, opcode.y),
                0x7 => self.op_8xy7(opcode.x, opcode.y),
                0xE => self.op_8xye(opcode.x, opcode.y),
                _ => panic!("unknown opcode {:04x}", opcode.raw),
            },
            0x9000 => self.op_9xy0(opcode.x, opcode.y),
            0xA000 => self.op_annn(opcode.nnn),
            0xB000 => self.op_bnnn(opcode.nnn),
            0xC000 => self.op_cxkk(opcode.x, opcode.kk),
            0xD000 => self.op_dxyn(opcode.x, opcode.y, opcode.n),
            0xE000 => match opcode.kk {
                0x9e => self.op_ex9e(opcode.x),
                0xa1 => self.op_exa1(opcode.x),
                _ => panic!("unknown opcode {:04x}", opcode.raw),
            },
            0xF000 => match opcode.kk {
                0x07 => self.op_fx07(opcode.x),
                0x0A => self.op_fx0a(opcode.x),
                0x15 => self.op_fx15(opcode.x),
                0x18 => self.op_fx18(opcode.x),
                0x1e => self.op_fx1e(opcode.x),
                0x29 => self.op_fx29(opcode.x),
                0x33 => self.op_fx33(opcode.x),
                0x55 => self.op_fx55(opcode.x),
                0x65 => self.op_fx65(opcode.x),
                _ => panic!("unknown opcode {:04x}", opcode.raw),
            },
            _ => panic!("unknown opcode {:04x}", opcode.raw),
        }
    }

    // Return from a subroutine
    fn op_00ee(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp];
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

    // Call subroutine at nnn
    fn op_2nnn(&mut self, nnn: u16) {
        self.stack[self.sp] = self.pc;
        self.sp += 1;
        self.pc = nnn;
    }

    // Skip next instruction if Vx = kk
    fn op_3xkk(&mut self, x: u8, kk: u8) {
        if self.register(x) == kk {
            self.pc += 2;
        }
    }

    // Skip next instruction if Vx != kk
    fn op_4xkk(&mut self, x: u8, kk: u8) {
        if self.register(x) != kk {
            self.pc += 2;
        }
    }

    // Skip next instruction if Vx = Vy
    fn op_5xy0(&mut self, x: u8, y: u8) {
        if self.register(x) == self.register(y) {
            self.pc += 2;
        }
    }

    // Set Vx = kk
    fn op_6xkk(&mut self, x: u8, kk: u8) {
        self.registers[usize::from(x)] = kk;
    }

    // Set Vx = Vx + kk
    fn op_7xkk(&mut self, x: u8, kk: u8) {
        self.registers[usize::from(x)] = self.register(x).overflowing_add(kk).0;
    }

    // Set Vx = Vy
    fn op_8xy0(&mut self, x: u8, y: u8) {
        self.registers[usize::from(x)] = self.register(y);
    }

    // Set Vx = Vx OR Vy
    fn op_8xy1(&mut self, x: u8, y: u8) {
        self.registers[usize::from(x)] |= self.register(y);
        self.set_flag(false); // Quirk
    }

    // Set Vx = Vx AND Vy
    fn op_8xy2(&mut self, x: u8, y: u8) {
        self.registers[usize::from(x)] &= self.register(y);
        self.set_flag(false); // Quirk
    }

    // Set Vx = Vx XOR Vy
    fn op_8xy3(&mut self, x: u8, y: u8) {
        self.registers[usize::from(x)] ^= self.register(y);
        self.set_flag(false); // Quirk
    }

    // Set Vx = Vx + Vy, set VF = carry
    fn op_8xy4(&mut self, x: u8, y: u8) {
        let (result, carry) = self.register(x).overflowing_add(self.register(y));

        self.registers[usize::from(x)] = result;
        self.set_flag(carry);
    }

    // Set Vx = Vx - Vy, set VF = NOT borrow
    fn op_8xy5(&mut self, x: u8, y: u8) {
        let (result, carry) = self.register(x).overflowing_sub(self.register(y));

        self.registers[usize::from(x)] = result;
        self.set_flag(!carry);
    }

    // Set Vx = Vx SHR 1
    fn op_8xy6(&mut self, x: u8, y: u8) {
        let lsb = self.register(y) & 0b0000_0001;

        self.registers[usize::from(x)] = self.register(y) >> 1;
        self.set_flag(lsb);
    }

    // Set Vx = Vy - Vx, set VF = NOT borrow
    fn op_8xy7(&mut self, x: u8, y: u8) {
        let (result, carry) = self.register(y).overflowing_sub(self.register(x));

        self.registers[usize::from(x)] = result;
        self.set_flag(!carry);
    }

    // Set Vx = Vx SHR 1
    fn op_8xye(&mut self, x: u8, y: u8) {
        let msb = self.register(y) >> 7;

        self.registers[usize::from(x)] = self.register(y) << 1;
        self.set_flag(msb);
    }

    // Skip next instruction if Vx != Vy
    fn op_9xy0(&mut self, x: u8, y: u8) {
        if self.register(x) != self.register(y) {
            self.pc += 2;
        }
    }

    // Set I = nnn
    fn op_annn(&mut self, nnn: u16) {
        self.i = nnn;
    }

    // Jump to location nnn + V0
    fn op_bnnn(&mut self, nnn: u16) {
        self.pc = nnn + u16::from(self.register(0));
    }

    // Set Vx = random byte AND kk
    fn op_cxkk(&mut self, x: u8, kk: u8) {
        self.registers[usize::from(x)] = kk & rand::random::<u8>();
    }

    // Display n-byte sprite starting at memory location I at (Vx, Vy)
    fn op_dxyn(&mut self, x: u8, y: u8, n: u8) {
        let x_pos = self.register(x);
        let y_pos = self.register(y);

        let sprite = &self.memory[usize::from(self.i)..usize::from(self.i + n as u16)];

        let collision = self.screen.draw(x_pos, y_pos, sprite);
        self.set_flag(collision);

        self.draw_flag = true;
    }

    // Skip the next instruction if key with the value of Vx is pressed
    fn op_ex9e(&mut self, x: u8) {
        if self.keys[usize::from(self.register(x))] {
            self.pc += 2
        }
    }

    // Skip the next instruction if key with the value of Vx is not pressed
    fn op_exa1(&mut self, x: u8) {
        if !self.keys[usize::from(self.register(x))] {
            self.pc += 2
        }
    }

    // Set Vx = delay timer value
    fn op_fx07(&mut self, x: u8) {
        self.registers[usize::from(x)] = self.dt;
    }

    // Wait for a key press, store the value of the key in Vx
    fn op_fx0a(&mut self, x: u8) {
        self.wait_key = x;
    }

    // Set delay timer = Vx
    fn op_fx15(&mut self, x: u8) {
        self.dt = self.register(x);
    }

    // Set sound timer = Vx
    fn op_fx18(&mut self, x: u8) {
        self.st = self.register(x);
    }

    // Set I = I + Vx
    fn op_fx1e(&mut self, x: u8) {
        self.i += u16::from(self.register(x));
    }

    // Set I = location of sprite for digit Vx
    fn op_fx29(&mut self, x: u8) {
        self.i = u16::from(self.register(x)) * 5;
    }

    // Store BCD representation of Vx in memory locations I, I+1, and I+2
    fn op_fx33(&mut self, x: u8) {
        let vx = self.register(x);

        self.memory[usize::from(self.i)] = vx / 100;
        self.memory[usize::from(self.i + 1)] = vx % 100 / 10;
        self.memory[usize::from(self.i + 2)] = vx % 10;
    }

    // Store registers V0 through Vx in memory starting at location I
    fn op_fx55(&mut self, x: u8) {
        self.memory[usize::from(self.i)..=usize::from(self.i + u16::from(x))]
            .copy_from_slice(&self.registers[0..=usize::from(x)]);

        self.i += u16::from(x) + 1;
    }

    // Read registers V0 through Vx from memory starting at location I
    fn op_fx65(&mut self, x: u8) {
        self.registers[0..=usize::from(x)].copy_from_slice(
            &self.memory[usize::from(self.i)..=usize::from(self.i + u16::from(x))],
        );

        self.i += u16::from(x) + 1;
    }

    fn register(&self, index: u8) -> u8 {
        self.registers[usize::from(index)]
    }

    fn set_flag<T: Into<u8>>(&mut self, value: T) {
        self.registers[0xF] = value.into();
    }

    const FONT: [u8; 80] = [
        0b11110000,
        0b10010000,
        0b10010000,
        0b10010000,
        0b11110000,

        0b00100000,
        0b01100000,
        0b00100000,
        0b00100000,
        0b01110000,

        0b11110000,
        0b00010000,
        0b11110000,
        0b10000000,
        0b11110000,

        0b11110000,
        0b00010000,
        0b11110000,
        0b00010000,
        0b11110000,

        0b10010000,
        0b10010000,
        0b11110000,
        0b00010000,
        0b00010000,

        0b11110000,
        0b10000000,
        0b11110000,
        0b00010000,
        0b11110000,

        0b11110000,
        0b10000000,
        0b11110000,
        0b10010000,
        0b11110000,

        0b11110000,
        0b00010000,
        0b00100000,
        0b01000000,
        0b01000000,

        0b11110000,
        0b10010000,
        0b11110000,
        0b10010000,
        0b11110000,

        0b11110000,
        0b10010000,
        0b11110000,
        0b00010000,
        0b11110000,

        0b11110000,
        0b10010000,
        0b11110000,
        0b10010000,
        0b10010000,

        0b11100000,
        0b10010000,
        0b11100000,
        0b10010000,
        0b11100000,

        0b11110000,
        0b10000000,
        0b10000000,
        0b10000000,
        0b11110000,

        0b11100000,
        0b10010000,
        0b10010000,
        0b10010000,
        0b11100000,

        0b11110000,
        0b10000000,
        0b11110000,
        0b10000000,
        0b11110000,

        0b11110000,
        0b10000000,
        0b11110000,
        0b10000000,
        0b10000000,
    ];
}
