use rand::Rng;
use std::fs;
use std::thread;
use std::time::Duration;
use ggez::graphics::{Canvas, Color, DrawMode, Mesh, Rect};
use ggez::event::{self, EventHandler};
use ggez::input::keyboard::KeyCode;
use ggez::{graphics, Context, GameResult};

pub const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

struct Chip8 {
    memory: [u8; 4096],
    registers: [u8; 16],
    display: [u8; 2048],
    i: usize,
    program_counter: usize,
    stack_pointer: usize,
    stack: [u16; 16],
    keypad: [bool; 16],
    waiting_for_keypress: bool,
    keypad_pressed: u8,
    delay_timer: u8,
    sound_timer: u8,
}

impl Chip8 {
    // Return from a subroutine.
    // sets the program counter to the address at the top of the stack,
    // then subtracts 1 from the stack pointer.
    fn op_00EE(&mut self) {
        self.program_counter = self.stack[self.stack_pointer] as usize;
        if self.stack_pointer > 0 {
            self.stack_pointer -= 1;
        }
    }

    // Clear display
    fn op_00E0(&mut self) {
        for i in self.display.iter_mut() {
            *i = 0;
        }
    }

    // The interpreter sets the program counter to nnn.
    fn op_1nnn(&mut self, nnn: usize) {
        self.program_counter = nnn
    }

    // The interpreter increments the stack pointer, then puts the current PC on the top of the stack. The PC is then set to nnn.
    fn op_2nnn(&mut self, nnn: usize) {
        self.stack_pointer += 1;
        self.stack[self.stack_pointer] = self.program_counter as u16;
        self.program_counter = nnn
    }

    // The interpreter compares register Vx to kk, and if they are equal, increments the program counter by 2.
    fn op_3xkk(&mut self, vx: u8, kk: u8) {
        if self.registers[vx as usize] == kk {
            self.program_counter += 2;
        }
    }

    // The interpreter compares register Vx to kk, and if they are not equal, increments the program counter by 2.
    fn op_4xkk(&mut self, vx: u8, kk: u8) {
        if self.registers[vx as usize] != kk {
            self.program_counter += 2;
        }
    }

    // The interpreter compares register Vx to register Vy, and if they are equal, increments the program counter by 2.
    fn op_5xy0(&mut self, vx: u8, vy: u8) {
        if self.registers[vx as usize] == self.registers[vy as usize] {
            self.program_counter += 2;
        }
    }

    // The interpreter puts the value kk into register Vx.
    fn op_6xkk(&mut self, vx: u8, kk: u8) {
        self.registers[vx as usize] = kk
    }

    // Adds the value kk to the value of register Vx, then stores the result in Vx.
    fn op_7xkk(&mut self, vx: u8, kk: u8) {
        self.registers[vx as usize] += kk
    }

    // Stores the value of register Vy in register Vx.
    fn op_8xy0(&mut self, vx: u8, vy: u8) {
        self.registers[vx as usize] = self.registers[vy as usize]
    }

    // VX = VX OR VY
    fn op_8xy1(&mut self, vx: u8, vy: u8) {
        self.registers[vx as usize] = self.registers[vx as usize] | self.registers[vy as usize]
    }

    // VX = VX & VY
    fn op_8xy2(&mut self, vx: u8, vy: u8) {
        self.registers[vx as usize] = self.registers[vx as usize] & self.registers[vy as usize]
    }

    // VX = VX XOR VY
    fn op_8xy3(&mut self, vx: u8, vy: u8) {
        self.registers[vx as usize] = self.registers[vx as usize] ^ self.registers[vy as usize]
    }

    // Add VY to VX and store the result in VX
    // If the sum is larger than 255 mark the carry flag to 1
    fn op_8xy4(&mut self, vx: u8, vy: u8) {
        if self.registers[vx as usize] + self.registers[vy as usize] > 255 {
            self.registers[15] = 1;
        } else {
            self.registers[15] = 0;
        }

        self.registers[vx as usize] += self.registers[vy as usize]
    }

    // If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx, and the results stored in Vx.
    fn op_8xy5(&mut self, vx: u8, vy: u8) {
        if self.registers[vx as usize] > self.registers[vy as usize] {
            self.registers[15] = 1;
        } else {
            self.registers[15] = 0;
        }

        self.registers[vx as usize] -= self.registers[vy as usize]
    }

    // If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2.
    fn op_8xy6(&mut self, vx: u8) {
        if (self.registers[vx as usize] & 1) == 1 {
            self.registers[15] = 1;
        } else {
            self.registers[15] = 0;
        }

        self.registers[vx as usize] = self.registers[vx as usize] >> 1
    }

    // If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy, and the results stored in Vx.
    fn op_8xy7(&mut self, vx: u8, vy: u8) {
        if self.registers[vy as usize] > self.registers[vx as usize] {
            self.registers[15] = 1;
        } else {
            self.registers[15] = 0;
        }

        self.registers[vx as usize] = self.registers[vy as usize] - self.registers[vx as usize]
    }

    // If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. Then Vx is multiplied by 2.
    fn op_8xye(&mut self, vx: u8) {
        if ((self.registers[vx as usize] >> 7) & 1) == 1 {
            self.registers[15] = 1;
        } else {
            self.registers[15] = 0;
        }

        self.registers[vx as usize] *= 2
    }

    // Skip next instruction if Vx != Vy.
    // The values of Vx and Vy are compared, and if they are not equal, the program counter is increased by 2.
    fn op_9xy0(&mut self, vx: u8, vy: u8) {
        if self.registers[vx as usize] != self.registers[vy as usize] {
            self.program_counter += 2;
        }
    }

    // The value of register I is set to nnn
    fn op_annn(&mut self, nnn: usize) {
        self.i = nnn
    }

    // The program counter is set to nnn plus the value of V0.
    fn op_bnnn(&mut self, nnn: usize) {
        self.program_counter = nnn + self.registers[0] as usize
    }

    // The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk. The results are stored in Vx.
    fn op_cxkk(&mut self, vx: u8, kk: u8) {
        let mut rng = rand::thread_rng();
        let rand_byte: u8 = rng.gen();

        self.registers[vx as usize] = rand_byte & kk;
    }

    // The values of I and Vx are added, and the results are stored in I.
    fn op_fx1e(&mut self, vx: u8) {
        self.i += self.registers[vx as usize] as usize
    }

    // The value of I is set to the location for the hexadecimal sprite corresponding to the value of Vx.
    fn op_fx29(&mut self, vx: u8) {
        self.i = (self.registers[vx as usize] * 5) as usize
    }

    // Skip next instruction if key with the value of Vx is pressed.
    fn op_ex9e(&mut self, vx: u8) {
        let key = self.registers[vx as usize];
        if (key as usize) < 16 && self.keypad[key as usize] {
            self.program_counter += 2;
        }
    }

    // Skip next instruction if key with the value of Vx is not pressed.
    fn op_exa1(&mut self, vx: u8) {
        let key = self.registers[vx as usize];
        if (key as usize) < 16 && !self.keypad[key as usize] {
            self.program_counter += 2;
        }
    }

    // Set Vx = delay timer value.
    fn op_fx07(&mut self, vx: u8) {
        self.registers[vx as usize] = self.delay_timer
    }

    //
    fn op_fx0a(&mut self, vx: u8) {
        self.waiting_for_keypress = true;
        self.keypad_pressed = vx;
    }

    // Set delay timer = Vx.
    fn op_fx15(&mut self, vx: u8) {
        self.delay_timer = self.registers[vx as usize]
    }

    // Set sound timer = Vx.
    fn op_fx18(&mut self, vx: u8) {
        self.sound_timer = self.registers[vx as usize]
    }

    // The interpreter takes the decimal value of Vx, and places the hundreds digit in memory at location in I,
    // the tens digit at location I+1, and the ones digit at location I+2.
    fn op_fx33(&mut self, vx: u8) {
        let val = self.registers[vx as usize];
        self.memory[self.i] = val / 100;
        self.memory[self.i + 1] = (val / 10) % 10;
        self.memory[self.i + 2] = val % 10;
    }

    // The interpreter copies the values of registers V0 through Vx into memory, starting at the address in I.
    fn op_fx55(&mut self, x: u8) {
        for i in 0..=x {
            self.memory[self.i as usize + i as usize] = self.registers[i as usize];
        }
    }

    // The interpreter reads values from memory starting at location I into registers V0 through Vx.
    fn op_fx65(&mut self, x: u8) {
        for i in 0..=x {
            self.registers[i as usize] = self.memory[self.i as usize + i as usize];
        }
    }

    // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    fn op_dxyn(&mut self, n: usize, vx: u8, vy: u8) {
        let x = self.registers[vx as usize] as usize % 64;
        let y = self.registers[vy as usize] as usize % 32;
        self.registers[15] = 0;

        for byte_idx in 0..n {
            let byte = self.memory[self.i + byte_idx];
            for bit in 0..8 {
                let px = (x + bit) % 64;
                let py = (y + byte_idx) % 32;
                let index = py * 64 + px;

                let sprite_pixel = (byte >> (7 - bit)) & 1;
                let screen_pixel = self.display[index];

                if screen_pixel == 1 && sprite_pixel == 1 {
                    self.registers[15] = 1;
                }

                self.display[index] ^= sprite_pixel;
            }
        }
    }

    // Load ROM into memory
    fn load(&mut self) {
        let input_bytes = fs::read("flightrunner.ch8").expect("Error reading input file");
        for (i, byte) in input_bytes.iter().enumerate() {
            if 0x200 + i < 4096 {
                self.memory[0x200 + i] = *byte;
            }
        }
    }

    fn tick(&mut self) {
        if self.waiting_for_keypress {
            for i in 0..16 {
                if self.keypad[i] {
                    self.waiting_for_keypress = false;
                    self.registers[self.keypad_pressed as usize] = i as u8;
                    break;
                }
            }
            return;
        }

        // Timers
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }

        // Fetch opcode
        let opcode = (self.memory[self.program_counter] as u16) << 8
            | (self.memory[self.program_counter + 1] as u16);
        self.program_counter += 2;

        // Decode
        let nnn = (opcode & 0x0FFF) as usize;
        let n = (opcode & 0x000F) as usize;
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let y = ((opcode & 0x00F0) >> 4) as u8;
        let kk = (opcode & 0x00FF) as u8;

        match opcode & 0xF000 {
            0x0000 => match opcode & 0x00FF {
                0x00E0 => self.op_00E0(),
                0x00EE => self.op_00EE(),
                _ => panic!("Unknown 0x0000 opcode: {:#04x}", opcode),
            },
            0x1000 => self.op_1nnn(nnn),
            0x2000 => self.op_2nnn(nnn),
            0x3000 => self.op_3xkk(x, kk),
            0x4000 => self.op_4xkk(x, kk),
            0x5000 => {
                if opcode & 0x000F == 0 {
                    self.op_5xy0(x, y);
                }
            }
            0x6000 => self.op_6xkk(x, kk),
            0x7000 => self.op_7xkk(x, kk),
            0x8000 => match opcode & 0x000F {
                0x0 => self.op_8xy0(x, y),
                0x1 => self.op_8xy1(x, y),
                0x2 => self.op_8xy2(x, y),
                0x3 => self.op_8xy3(x, y),
                0x4 => self.op_8xy4(x, y),
                0x5 => self.op_8xy5(x, y),
                0x6 => self.op_8xy6(x),
                0x7 => self.op_8xy7(x, y),
                0xE => self.op_8xye(x),
                _ => panic!("Unknown 0x8000 opcode: {:#04x}", opcode),
            },
            0x9000 => {
                if opcode & 0x000F == 0 {
                    self.op_9xy0(x, y);
                }
            }
            0xA000 => self.op_annn(nnn),
            0xB000 => self.op_bnnn(nnn),
            0xC000 => self.op_cxkk(x, kk),
            0xD000 => self.op_dxyn(n, x, y),
            0xE000 => match opcode & 0x00FF {
                0x9E => self.op_ex9e(x),
                0xA1 => self.op_exa1(x),
                _ => panic!("Unknown 0xE000 opcode: {:#04x}", opcode),
            },
            0xF000 => match opcode & 0x00FF {
                0x07 => self.op_fx07(x),
                0x0A => self.op_fx0a(x),
                0x15 => self.op_fx15(x),
                0x18 => self.op_fx18(x),
                0x1E => self.op_fx1e(x),
                0x29 => self.op_fx29(x),
                0x33 => self.op_fx33(x),
                0x55 => self.op_fx55(x),
                0x65 => self.op_fx65(x),
                _ => panic!("Unknown 0xF000 opcode: {:#04x}", opcode),
            },
            _ => panic!("Unknown opcode: {:#04x}", opcode),
        }
    }

    fn set_key(&mut self, keycode: KeyCode, pressed: bool) {
        let key = match keycode {
            KeyCode::Key1 => Some(0x1),
            KeyCode::Key2 => Some(0x2),
            KeyCode::Key3 => Some(0x3),
            KeyCode::Key4 => Some(0xC),
            KeyCode::Q => Some(0x4),
            KeyCode::W => Some(0x5),
            KeyCode::E => Some(0x6),
            KeyCode::R => Some(0xD),
            KeyCode::A => Some(0x7),
            KeyCode::S => Some(0x8),
            KeyCode::D => Some(0x9),
            KeyCode::F => Some(0xE),
            KeyCode::Z => Some(0xA),
            KeyCode::X => Some(0x0),
            KeyCode::C => Some(0xB),
            KeyCode::V => Some(0xF),
            _ => None,
        };

        if let Some(k) = key {
            self.keypad[k] = pressed;
        }
    }

    fn draw_ggez(&self, ctx: &mut Context, canvas: &mut Canvas) -> GameResult {
        let pixel_size = 10.0;

        for y in 0..32 {
            for x in 0..64 {
                let index = y * 64 + x;
                if self.display[index] == 1 {
                    let rect = Mesh::new_rectangle(
                        ctx,
                        DrawMode::fill(),
                        Rect::new(
                            x as f32 * pixel_size,
                            y as f32 * pixel_size,
                            pixel_size,
                            pixel_size,
                        ),
                        Color::WHITE,
                    )?;
                    canvas.draw(&rect, ggez::mint::Point2 { x: 0.0, y: 0.0 });
                }
            }
        }

        Ok(())
    }
}

struct MainState {
    chip8: Chip8,
}

impl MainState {
    fn new() -> GameResult<MainState> {
        let mut chip8 = Chip8 {
            memory: [0; 4096],
            registers: [0; 16],
            display: [0; 2048],
            i: 0,
            program_counter: 0x200,
            stack_pointer: 0,
            stack: [0; 16],
            keypad: [false; 16],
            waiting_for_keypress: false,
            keypad_pressed: 0,
            delay_timer: 0,
            sound_timer: 0,
        };

        // Load font + ROM
        for (i, byte) in FONT.iter().enumerate() {
            chip8.memory[i] = *byte;
        }
        chip8.load();

        Ok(MainState { chip8 })
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        // Run multiple ticks to simulate ~500Hz CPU
        for _ in 0..10 {
            self.chip8.tick();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([0.0, 0.0, 0.0, 1.0]));
        self.chip8.draw_ggez(ctx, &mut canvas)?;
        canvas.finish(ctx)?;
        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, input: ggez::input::keyboard::KeyInput, _repeat: bool) -> GameResult {
        if let Some(keycode) = input.keycode {
            self.chip8.set_key(keycode, true); // or false
        }
        Ok(())
    }

    fn key_up_event(&mut self, _ctx: &mut Context, input: ggez::input::keyboard::KeyInput) -> GameResult {
        if let Some(keycode) = input.keycode {
            self.chip8.set_key(keycode, true); // or false
        }
        Ok(())
    }
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("chip8", "your_name_here")
        .window_setup(ggez::conf::WindowSetup::default().title("CHIP-8 Emulator"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(640.0, 320.0)); // 64 x 10, 32 x 10

    let (ctx, event_loop) = cb.build()?;
    let state = MainState::new()?;
    event::run(ctx, event_loop, state)
}