# CHIP-8 Emulator in Rust

This is a fully functional CHIP-8 emulator written in Rust using the [`ggez`](https://github.com/ggez/ggez) game framework. It was built from scratch to deepen my understanding of systems programming, low-level architecture, and graphics/input processing.

### Systems Programming with Rust
- Managed memory safely without garbage collection
- Used enums and pattern matching to decode and execute 35+ opcodes
- Implemented a simple virtual CPU (program counter, instruction decoder, call stack, registers)

### Graphics & Input with ggez
- Built a pixel-based framebuffer and display logic for the CHIP-8’s 64×32 screen
- Handled input using `key_down_event` / `key_up_event`, correctly simulating key wait behavior
- Simulated a ~500Hz CPU with decoupled frame rendering

### Timing & Game Loop Architecture
- Created a fixed-timestep update loop to emulate real CHIP-8 CPU timing
- Correctly implemented timers (`DT` and `ST`) and coordinated them with system updates

### Debugging
- Debugged ROM behavior by logging program counter, opcode values, and register states
- Tracked down bugs like input flickering by understanding real emulator edge cases
- Verified correctness by running ROMs like `IBM Logo`, `Brix`, and `Pong`

## Features
- Loads and executes standard CHIP-8 ROMs
- Implements all official CHIP-8 instructions
- Accurate input handling and key wait (`Fx0A`)
- Pixel-accurate rendering with XOR-based sprite drawing
- Basic sound timer (placeholder or beeping optional)

## Tools & Libraries
- Rust
- ggez (graphics/input)
- cargo-watch (for fast iterative development)
