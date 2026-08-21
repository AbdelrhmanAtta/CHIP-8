//! CHIP-8 emulator core.
//!
//! Implements the CPU, memory, display buffer, and instruction set for a
//! CHIP-8 virtual machine.

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const REGISTERS_NUMBER: usize = 16;
const STACK_SIZE: usize = 16;
const KEYS_NUMBER: usize = 16;
const START_ADDR: u16 = 0x200;
const FONTSET_SIZE: usize = 80;

/// Built-in hexadecimal digit sprites (0-F), each 4x5 pixels, loaded into
/// the bottom of RAM at emulator start-up.
const FONTSET: [u8; FONTSET_SIZE] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

/// A CHIP-8 virtual machine: CPU state, RAM, stack, keypad, timers, and
/// the monochrome frame buffer.
pub struct Emulator {
    /// Program counter, points at the next opcode to fetch.
    pc: u16,
    /// Main memory. The first `FONTSET_SIZE` bytes hold the built-in font.
    ram: [u8; RAM_SIZE],
    /// Monochrome frame buffer, `true` = pixel on. Indexed row-major.
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    /// General-purpose registers V0-VF. VF also serves as the flags
    /// register for carry/borrow/collision.
    v_registers: [u8; REGISTERS_NUMBER],
    /// Index register, typically holds a memory address.
    i_register: u16,
    /// Stack pointer, indexes the next free slot in `stack`.
    sp: u16,
    /// Call stack used to store return addresses for subroutine calls.
    stack: [u16; STACK_SIZE],
    /// State of the 16-key hex keypad, `true` = pressed.
    keys: [bool; KEYS_NUMBER],
    /// Counts down at 60Hz; used for game timing.
    delay_timer: u8,
    /// Counts down at 60Hz; beeps while non-zero.
    sound_timer: u8,
}

impl Emulator {
    /// Creates a new emulator with a clean RAM/register state and the
    /// built-in fontset loaded at the start of memory.
    pub fn new() -> Self {
        let mut new_emulator = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_registers: [0; REGISTERS_NUMBER],
            i_register: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; KEYS_NUMBER],
            delay_timer: 0,
            sound_timer: 0,
        };
        new_emulator.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        new_emulator
    }

    /// Fetches and executes a single instruction (one CPU cycle).
    pub fn tick(&mut self) {
        let op = self.fetch();
        self.execute(op);
    }

    /// Decrements the delay and sound timers. Should be called at 60Hz,
    /// independently of `tick`.
    pub fn tick_timer(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // Will Add Beep Later
            }
            self.sound_timer -= 1;
        }
    }

    /// Pushes a return address onto the call stack.
    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    /// Pops and returns the most recent address from the call stack.
    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    /// Reads the 16-bit opcode at `pc` (big-endian) and advances `pc`
    /// by two bytes.
    fn fetch(&mut self) -> u16 {
        let high_byte = self.ram[self.pc as usize] as u16;
        let low_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (high_byte << 8) | low_byte;
        self.pc += 2;
        op
    }

    /// Decodes and runs a single opcode.
    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            (0, 0, 0, 0) => return, // nop
            (0, 0, 0xE, 0) => {
                // clc: clear the screen
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            }
            (0, 0, 0xE, 0xE) => {
                // return: pop return address and jump to it
                let return_addr = self.pop();
                self.pc = return_addr;
            }
            (0x1, _, _, _) => {
                // jump: PC = NNN
                let nnn = op & 0xFFF;
                self.pc = nnn;
            }
            (0x2, _, _, _) => {
                // call sub-routine at NNN
                let nnn = op & 0xFFF;
                self.push(self.pc);
                self.pc = nnn;
            }
            (0x3, _, _, _) => {
                // skip next instruction if Vx == NN
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_registers[x] == nn {
                    self.pc += 2;
                }
            }
            (0x4, _, _, _) => {
                // skip next instruction if Vx != NN
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_registers[x] != nn {
                    self.pc += 2;
                }
            }
            (0x5, _, _, 0) => {
                // skip next instruction if Vx == Vy
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_registers[x] == self.v_registers[y] {
                    self.pc += 2;
                }
            }
            (0x6, _, _, _) => {
                // Vx = NN
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_registers[x] = nn;
            }
            (0x7, _, _, _) => {
                // Vx += NN (wrapping, no flag update)
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_registers[x] = self.v_registers[x].wrapping_add(nn);
            }
            (0x8, _, _, 0x0) => {
                // Vx = Vy
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_registers[x] = self.v_registers[y];
            }
            (0x8, _, _, 0x1) => {
                // Vx |= Vy
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_registers[x] |= self.v_registers[y];
            }
            (0x8, _, _, 0x2) => {
                // Vx &= Vy
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_registers[x] &= self.v_registers[y];
            }
            (0x8, _, _, 0x3) => {
                // Vx ^= Vy
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_registers[x] ^= self.v_registers[y];
            }
            (0x8, _, _, 0x4) => {
                // Vx += Vy, VF = carry
                let x = digit2 as usize;
                let y = digit3 as usize;
                let (new_vx, carry) =
                    self.v_registers[x].overflowing_add(self.v_registers[y]);
                let new_vf = if carry { 1 } else { 0 };
                self.v_registers[x] = new_vx;
                self.v_registers[0xF] = new_vf;
            }
            (0x8, _, _, 0x5) => {
                // Vx -= Vy, VF = NOT borrow
                let x = digit2 as usize;
                let y = digit3 as usize;
                let (new_vx, borrow) =
                    self.v_registers[x].overflowing_sub(self.v_registers[y]);
                let new_vf = if borrow { 0 } else { 1 };
                self.v_registers[x] = new_vx;
                self.v_registers[0xF] = new_vf;
            }
            (0x8, _, _, 0x6) => {
                // Vx >>= 1, VF = dropped bit
                let x = digit2 as usize;
                let dropped_bit = self.v_registers[x] & 1;
                self.v_registers[x] >>= 1;
                self.v_registers[0xF] = dropped_bit;
            }
            (0x8, _, _, 0x7) => {
                // Vx = Vy - Vx, VF = NOT borrow
                let x = digit2 as usize;
                let y = digit3 as usize;
                let (new_vx, borrow) =
                    self.v_registers[y].overflowing_sub(self.v_registers[x]);
                let new_vf = if borrow { 0 } else { 1 };
                self.v_registers[x] = new_vx;
                self.v_registers[0xF] = new_vf;
            }
            (0x8, _, _, 0xE) => {
                // Vx <<= 1, VF = dropped bit
                let x = digit2 as usize;
                let dropped_bit = (self.v_registers[x] >> 7) & 1;
                self.v_registers[x] <<= 1;
                self.v_registers[0xF] = dropped_bit;
            }
            (0x9, _, _, 0) => {
                // skip next instruction if Vx != Vy
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_registers[x] != self.v_registers[y] {
                    self.pc += 2;
                }
            }
            (0xA, _, _, _) => {
                // I = NNN
                let nnn = op & 0xFFF;
                self.i_register = nnn;
            }
            (0xB, _, _, _) => {
                // PC = V0 + NNN
                let nnn = op & 0xFFF;
                self.pc = (self.v_registers[0] as u16) + nnn;
            }
            (0xC, _, _, _) => {
                // Vx = rand() & NN
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                let rand: u8 = rand::random();
                self.v_registers[x] = rand & nn;
            }
            (0xD, _, _, _) => {
                // draw sprite at (Vx, Vy) with height N, VF = collision
                let x_col = self.v_registers[digit2 as usize] as u16;
                let y_col = self.v_registers[digit3 as usize] as u16;
                let row_n = digit4;
                let mut flipped = false;

                for y_line in 0..row_n {
                    let addr = self.i_register + y_line as u16;
                    let pixels = self.ram[addr as usize];

                    for x_line in 0..8 {
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            let x = (x_line + x_col) as usize % SCREEN_WIDTH;
                            let y = (y_line + y_col) as usize % SCREEN_HEIGHT;

                            let idx = (SCREEN_WIDTH * y) + x;
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }

                self.v_registers[0xF] = if flipped { 1 } else { 0 };
            }
            (0xE, _, 0x9, 0xE) => {
                // skip next instruction if key(Vx) is pressed
                let x = digit2 as usize;
                let vx = self.v_registers[x];
                let key = self.keys[vx as usize];
                if key {
                    self.pc += 2;
                }
            }
            (0xE, _, 0xA, 0x1) => {
                // skip next instruction if key(Vx) is not pressed
                let x = digit2 as usize;
                let vx = self.v_registers[x];
                let key = self.keys[vx as usize];
                if !key {
                    self.pc += 2;
                }
            }
            (0xF, _, 0x0, 0x7) => {
                // Vx = delay_timer
                let x = digit2 as usize;
                self.v_registers[x] = self.delay_timer;
            }
            (0xF, _, 0x0, 0xA) => {
                // Vx = get_key() (blocking: re-runs this opcode until a
                // key is pressed)
                let x = digit2 as usize;
                let mut pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_registers[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }
                if !pressed {
                    self.pc -= 2;
                }
            }
            (0xF, _, 0x1, 0x5) => {
                // delay_timer = Vx
                let x = digit2 as usize;
                self.delay_timer = self.v_registers[x];
            }
            (0xF, _, 0x1, 0x8) => {
                // sound_timer = Vx
                let x = digit2 as usize;
                self.sound_timer = self.v_registers[x];
            }
            (0xF, _, 0x1, 0xE) => {
                // I += Vx
                let x = digit2 as usize;
                let vx = self.v_registers[x] as u16;
                self.i_register = self.i_register.wrapping_add(vx);
            }
            (0xF, _, 0x2, 0x9) => {
                // I = address of the built-in font sprite for digit Vx
                let x = digit2 as usize;
                let vx = self.v_registers[x] as u16;
                self.i_register = vx * 5;
            }
            (0xF, _, 0x3, 0x3) => {
                // store the BCD representation of Vx at I, I+1, I+2
                let x = digit2 as usize;
                let vx = self.v_registers[x];

                let hundreds = vx / 100;
                let tens = (vx / 10) % 10;
                let ones = vx % 10;

                self.ram[self.i_register as usize] = hundreds;
                self.ram[(self.i_register + 1) as usize] = tens;
                self.ram[(self.i_register + 2) as usize] = ones;
            }
            (0xF, _, 0x5, 0x5) => {
                // reg_dump: store V0..=Vx to memory starting at I
                let x = digit2 as usize;
                let i = self.i_register as usize;
                for idx in 0..=x {
                    self.ram[i + idx] = self.v_registers[idx];
                }
            }
            (0xF, _, 0x6, 0x5) => {
                // reg_load: fill V0..=Vx from memory starting at I
                let x = digit2 as usize;
                let i = self.i_register as usize;
                for idx in 0..=x {
                    self.v_registers[idx] = self.ram[i + idx];
                }
            }
            (_, _, _, _) => unimplemented!("Unimplemented op-code: {:04X}", op),
        }
    }

    /// Resets the emulator to its initial state, as if freshly created.
    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_registers = [0; REGISTERS_NUMBER];
        self.i_register = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; KEYS_NUMBER];
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }
}
