# CHIP-8 Emulator

A CHIP-8 virtual machine written in Rust, split into a reusable core library
(`chip8_core`) and an SDL2-based desktop frontend (`desktop`).

## Features

- Full CHIP-8 instruction set (opcodes `0NNN`–`FX65`), including:
    - Arithmetic/logic ops with correct `VF` carry/borrow/shift flag behavior
    - Sprite drawing (`DXYN`) with XOR blitting, screen wrap-around, and collision detection
    - Subroutine call/return via a 16-level stack
    - Blocking key-wait (`FX0A`), BCD conversion (`FX33`), and register/memory block load-store (`FX55`/`FX65`)
- 64x32 monochrome framebuffer
- 16-key hex keypad mapped to a standard QWERTY layout
- Delay and sound timers ticked independently of the CPU clock
- Built-in 4x5 font set loaded at startup

## Project Structure

```
.
├── chip8_core/     # Core emulator: CPU, memory, display buffer, opcode execution
│   └── src/lib.rs
└── desktop/        # SDL2 frontend: window, rendering, input, main loop
    └── src/main.rs
```

## Requirements

- Rust (stable toolchain, via [rustup](https://rustup.rs))
- SDL2 development libraries installed on your system

## Build & Run

From the `desktop` directory:

```bash
cargo run -- path/to/rom
```

Example:

```bash
cargo run -- ../roms/INVADERS
```

Press `Esc` or close the window to quit.

## Controls

The original CHIP-8 hex keypad is mapped onto a standard keyboard as follows:

```
Keyboard Layout              CHIP-8 Hex Keypad
+---+---+---+---+            +---+---+---+---+
| 1 | 2 | 3 | 4 |            | 1 | 2 | 3 | C |
+---+---+---+---+            +---+---+---+---+
| Q | W | E | R |            | 4 | 5 | 6 | D |
+---+---+---+---+    ==>     +---+---+---+---+
| A | S | D | F |            | 7 | 8 | 9 | E |
+---+---+---+---+            +---+---+---+---+
| Z | X | C | V |            | A | 0 | B | F |
+---+---+---+---+            +---+---+---+---+
```

## Timing

- CPU executes `TICKS_PER_FRAME` (10) instructions per rendered frame.
- Delay and sound timers decrement at a fixed 60Hz, independent of the CPU tick rate.

## Available Games

The following ROMs are included in `roms/`:

- 15PUZZLE
- BLINKY
- BLITZ
- BRIX
- CONNECT4
- GUESS
- HIDDEN
- INVADERS
- KALEID
- MAZE
- MERLIN
- MISSILE
- PONG
- PONG2
- PUZZLE
- SYZYGY
- TANK
- TETRIS
- TICTAC
- UFO
- VBRIX
- VERS
- WIPEOFF

## Roadmap / Ideas

- [ ] Audio output on `sound_timer` expiry (currently a no-op hook)
- [ ] Configurable clock speed / cycles-per-frame
- [ ] Save/load emulator state
- [ ] Debug overlay (register/memory inspector)