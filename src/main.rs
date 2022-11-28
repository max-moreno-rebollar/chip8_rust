use std::fs;

use rand::Rng;

struct Chip8 {
    memory: [u8; 4096],
    registers: [u8; 16],
    display: [u8; 2048],
    i: usize,
    program_counter: usize,
    stack_pointer: usize,
    stack: [u16; 16],
}

impl Chip8 {
    // Return from a subroutine.
    // sets the program counter to the address at the top of the stack,
    // then subtracts 1 from the stack pointer.
    fn op_00EE(&mut self) {
        self.program_counter = self.stack_pointer;
        self.stack_pointer = self.stack_pointer - 1
    }

    // Clear display
    fn op_00E0(&mut self) {
        for i in self.display.iter_mut() {
            *i = 0;
        }
    }

    // The interpreter sets the program counter to nnn.
    fn op_1nnn(&mut self, nnn: u16) {
        self.program_counter = nnn
    }

    // The interpreter increments the stack pointer, then puts the current PC on the top of the stack. The PC is then set to nnn.
    fn op_2nnn(&mut self, nnn: u16) {
        self.stack_pointer += 1;
        self.stack[self.stack_pointer as usize] = self.program_counter;
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
    fn op_7xkk(vx: &mut u8, kk: u8) {
        *vx += kk
    }

    // Stores the value of register Vy in register Vx.
    fn op_8xy0(&mut self, vx: u8, vy: u8) {
        self.registers[vx as usize] = self.registers[vy as usize]
    }

    fn op_8xy1(&mut self, vx: u8, vy: u8) {
        self.registers[vx as usize] = self.registers[vx as usize] | self.registers[vy as usize]
    }

    fn _8xy2(mut vx: u8, vy: u8) {
        vx = vx & vy
    }

    fn _8xy3(mut vx: u8, vy: u8) {
        vx = vx ^ vy
    }

    fn _8xy4(mut vx: u8, vy: u8, mut vf: u8) {
        if (vx + vy > 255) {
            vf = 1;
        } else {
            vf = 0;
        }

        vx += vy
    }

    fn _8xy5(mut vx: u8, vy: u8, mut vf: u8) {
        if (vx > vy) {
            vf = 1;
        } else {
            vf = 0;
        }

        vx -= vy
    }

    fn _8xy6(mut vx: u8, mut vf: u8) {
        if ((vx & 1) == 1) {
            vf = 1;
        } else {
            vf = 0;
        }

        vx = vx >> 1
    }

    fn _8xy7(mut vx: u8, vy: u8, mut vf: u8) {
        if (vy > vx) {
            vf = 1;
        } else {
            vf = 0;
        }

        vx = vy - vx
    }

    fn _8xye(mut vx: u8, vy: u8, mut vf: u8) {
        if ((vx << 7) & 1 == 1) {
            vf = 1;
        } else {
            vf = 0;
        }

        vx = vx * 2
    }

    // The value of register I is set to nnn
    fn _annn(mut I: u16, nnn: u16) {
        I = nnn
    }

    // The program counter is set to nnn plus the value of V0.
    fn _bnnn(mut program_counter: u16, nnn: u16, v0: u8) {
        program_counter = nnn + v0 as u16
    }

    // The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk. The results are stored in Vx.
    fn _cxkk(vx: &mut u8, kk: u8) {
        let mut rng = rand::thread_rng();
        let randByte: u8 = rng.gen();

        *vx = kk & randByte
    }

    // The values of I and Vx are added, and the results are stored in I.
    fn op_fx1e(&mut self, vx: u8) {
        self.i += self.registers[vx as usize] as u16
    }

    fn op_fx29(&mut self) {}

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
    fn op_dxyn(&mut self, n: u32, vx: u8, vy: u8, vf: u8) {
        let addr: usize = (self.registers[vy as usize] * 64 + self.registers[vx as usize]) as usize;
        self.registers[15] = 0;
        for i in 0..n {
            let byte: u8 = self.memory[self.i as usize + i as usize];

            if self.memory[addr + i as usize] != 0 {
                // conflict
                self.registers[15] = 1;
            }

            self.memory[addr + i as usize] = self.memory[addr + i as usize] ^ byte;
        }
    }
}
