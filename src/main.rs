
struct chip8 {
    memory: [u8; 4096],
    registers: [u8; 16],
    I: u16,
    lprogram_counter: u16,
    stack_pointer: u8,
    stack: [u16; 16],
}

fn main() {
    

}

fn program_loop() {

}

// clear display
fn _00E0() {

}

// Return from a subroutine.
// sets the program counter to the address at the top of the stack,
// then subtracts 1 from the stack pointer.
fn _00EE() {

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
    if(vx + vy > 255) {
        vf = 1;
    } else {
        vf = 0;
    }

    vx += vy
}

fn _8xy5(mut vx: u8, vy: u8, mut vf: u8) {
    if(vx > vy) {
        vf = 1;
    } else {
        vf = 0;
    }

    vx -= vy
}

fn _8xy6(mut vx: u8, mut vf: u8) {
    if((vx & 1) == 1) {
        vf = 1;
    } else {
        vf = 0;
    }

    vx = vx >> 1
}

fn _8xy7(mut vx: u8, vy: u8, mut vf: u8) {
    if(vy > vx) {
        vf = 1;
    } else {
        vf = 0;
    }

    vx = vy - vx
}

fn _8xye(mut &vx: u8, vy: u8, mut vf: u8) {
    if((vx << 7) & 1 == 1) {
        vf = 1;
    } else {
        vf = 0;
    }

    vx = vx << 1
}

