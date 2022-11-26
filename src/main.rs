struct chip8 {
    memory: [u16; 4096],
    registers: [u8; 16],
    display: [u8; 2048],
    I: u16,
    program_counter: u16,
    stack_pointer: u8, // Can be the values 0 - 15
    stack: [u16; 16],
}

fn main() {}

fn program_loop() {}

// clear display
fn _00E0(mut display: [u8; 2048]) {
    for i in display.iter_mut() {
        *i = 0;
    }
}

// Return from a subroutine.
// sets the program counter to the address at the top of the stack,
// then subtracts 1 from the stack pointer.
fn _00EE(mut program_counter: u16, mut stack_pointer: u8, stack: [u16; 16]) {
    program_counter = stack_pointer as u16;
    stack_pointer = stack_pointer - 1
}

// The interpreter sets the program counter to nnn.
fn _1nnn(mut program_counter: u16, nnn: u16) {
    program_counter = nnn
}

// The interpreter increments the stack pointer, then puts the current PC on the top of the stack. The PC is then set to nnn.
fn _2nnn(mut stack_pointer: u8, mut program_counter: u16, mut stack: [u16; 16], nnn: u16) {
    stack_pointer = stack_pointer + 1;
    stack[stack_pointer as usize] = program_counter;
    program_counter = nnn
}


// The interpreter compares register Vx to kk, and if they are equal, increments the program counter by 2.
fn _3xkk(vx: u8, kk: u8, mut program_counter: u16) {
    if vx == kk {
        program_counter = program_counter + 2;
    }
}

// The interpreter compares register Vx to kk, and if they are not equal, increments the program counter by 2.
fn _4xkk(vx: u8, kk: u8, mut program_counter: u16) {
    if vx != kk {
        program_counter = program_counter + 2;
    }
}

// The interpreter compares register Vx to register Vy, and if they are equal, increments the program counter by 2.
fn _5xy0(vx: u8, vy: u8, mut program_counter: u16) {
    if(vx == vy) {
        program_counter = program_counter + 2;
    }
}

// The interpreter puts the value kk into register Vx.
fn _6xkk(mut vx: u8, kk: u8) {
    vx = kk
}

// Adds the value kk to the value of register Vx, then stores the result in Vx.
fn _7xkk(mut vx: u8, kk: u8) {
    vx += kk
}

// Stores the value of register Vy in register Vx.
fn _8xy0(mut vx: u8, vy: u8) {
    vx = vy
}

fn _8xy1(mut vx: u8, vy: u8) {
    vx = vx | vy
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

    vx = vx << 1
}

// The interpreter copies the values of registers V0 through Vx into memory, starting at the address in I.
fn _fx55(x: u8, I: u16, registers: [u8; 16], mut memory: [u8; 4096]) {
    for i in 0..=x {
        memory[(I as usize) + (i as usize)] = registers[i as usize];
    }
}

// The interpreter reads values from memory starting at location I into registers V0 through Vx.
fn _fx65(x: u8, I: u16, mut registers: [u8; 16], memory: [u8; 4096]) {
    for i in 0..=x {
        registers[i as usize] = memory[(I as usize) + (i as usize)] as u8;
    }
}
