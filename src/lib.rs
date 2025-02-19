use display::Display;
use instruction::Instruction;
use rand::Rng;
use winit::window::Window;
pub mod display;
pub mod instruction;
pub const STARTING_PC: u16 = 0x200;
const MEMORY_SIZE: usize = 4096;
const FONT_OFFSET: usize = 0x050;
pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;

pub struct Chip8<'a> {
    pub pc: u16,
    pub display: Display<'a>,
    pub memory: [u8; MEMORY_SIZE],
    pub stack: Vec<u16>,
    pub delay: u8,
    pub sound: u8,
    pub i: u16,
    pub v: [u8; 16],
}

impl<'a> Chip8<'a> {
    pub fn new(window: &'a Window) -> Self {
        let mut chip8 = Self {
            pc: STARTING_PC,
            display: Display::new(window),
            memory: [0; MEMORY_SIZE],
            stack: vec![],
            delay: 0,
            sound: 0,
            i: 0,
            v: [0; 16],
        };
        chip8.load_into_memory(FONT_OFFSET, FONT.to_vec());
        chip8
    }

    pub fn process_instruction(&mut self, pressed_inputs: &[bool; 16]) {
        let instr = Instruction::new(
            ((self.memory[self.pc as usize] as u16) << 8)
                | self.memory[self.pc as usize + 1] as u16,
        );

        self.pc += 2;

        match instr.a() {
            0x0 => match (instr.x(), instr.y(), instr.n()) {
                (0x0, 0xE, 0x0) => self.display.clear(),
                (0x0, 0xE, 0xE) => self.pc = self.stack.pop().expect("stack empty"),
                _ => unimplemented!("{:x?}", instr),
            },

            // jump to NNN
            0x1 => self.pc = instr.nnn(),

            // call subroutine at NNN
            0x2 => {
                self.stack.push(self.pc);
                self.pc = instr.nnn();
            }

            // skip next if VX == NN
            0x3 => {
                if self.v[instr.x()] == instr.nn() {
                    self.pc += 2;
                }
            }

            // skip next if VX != NN
            0x4 => {
                if self.v[instr.x()] != instr.nn() {
                    self.pc += 2;
                }
            }

            // skip next if VX == VY
            0x5 => {
                if self.v[instr.x()] == self.v[instr.y()] {
                    self.pc += 2;
                }
            }

            // set VX to NN
            0x6 => self.v[instr.x()] = instr.nn(),

            // add NN to VX (carry flag is not changed)
            0x7 => self.v[instr.x()] = self.v[instr.x()].wrapping_add(instr.nn()),

            0x8 => match instr.n() {
                // set VX to value of VY
                0x0 => self.v[instr.x()] = self.v[instr.y()],

                // set VX to VX | VY
                0x1 => self.v[instr.x()] = self.v[instr.x()] | self.v[instr.y()],

                // set VX to VX & VY
                0x2 => self.v[instr.x()] = self.v[instr.x()] & self.v[instr.y()],

                // set VX to VX xor VY
                0x3 => self.v[instr.x()] = self.v[instr.x()] ^ self.v[instr.y()],

                // add VY to VX (set VF to 1 if overflow, 0 if not)
                0x4 => {
                    let (sum, overflowed) = self.v[instr.x()].overflowing_add(self.v[instr.y()]);
                    self.v[instr.x()] = sum;
                    self.v[0xF] = overflowed as u8;
                }

                // subtract VY from VX (set VF to 0 if underflow, 1 if not)
                0x5 => {
                    self.v[0xF] = (self.v[instr.x()] >= self.v[instr.y()]) as u8;
                    self.v[instr.x()] = self.v[instr.x()].wrapping_sub(self.v[instr.y()]);
                }

                // store least significant bit of VX into VF and shift VX right by 1
                0x6 => {
                    self.v[0xF] = self.v[instr.x()] & 1;
                    self.v[instr.x()] >>= 1;
                }

                // set VX to VY minus VX (set VF to 0 if underflow, 1 if not)
                0x7 => {
                    self.v[0xF] = (self.v[instr.y()] >= self.v[instr.x()]) as u8;
                    self.v[instr.x()] = self.v[instr.y()].wrapping_sub(self.v[instr.x()]);
                }

                // set VF to 1 if most signifant bit of VX was set, or 0 if not, and then shift VX left by 1
                0xE => {
                    self.v[0xF] = (self.v[instr.x()] >> 7) & 1;
                    self.v[instr.x()] <<= 1
                }
                _ => unimplemented!("{:x?}", instr),
            },

            // skip next if VX != VY
            0x9 => {
                if self.v[instr.x()] != self.v[instr.y()] {
                    self.pc += 2;
                }
            }

            // set I to address NNN
            0xA => self.i = instr.nnn(),

            // jump to address NNN plus V0
            0xB => self.pc = instr.nnn() + self.v[0] as u16,

            // set VX to result of bitwise AND operation with random number
            0xC => {
                let num = rand::rng().random_range(0..255);
                self.v[instr.x()] = num & instr.nn();
            }

            // Draws a sprite at coordinate (VX, VY)
            // that has a width of 8 pixels and a height of N pixels.
            // Each row of 8 pixels is read as bit-coded starting from memory location I;
            // I value does not change after the execution of this instruction.
            // As described above,
            // VF is set to 1 if any screen pixels are flipped from set to unset when the sprite is drawn,
            // and to 0 if that does not happen
            0xD => {
                let x = self.v[instr.x()].rem_euclid(DISPLAY_WIDTH as u8);
                let mut y = self.v[instr.y()].rem_euclid(DISPLAY_HEIGHT as u8);

                self.v[0xF] = 0;

                for row in 0..instr.n() {
                    if y == (DISPLAY_HEIGHT as u8 - 1) {
                        break;
                    }

                    let sprite_byte = self.memory[(self.i + row as u16) as usize];
                    let mut x = x;
                    for i in 0..8 {
                        if x == DISPLAY_WIDTH as u8 - 1 {
                            break;
                        }

                        let bit = ((sprite_byte >> (7 - i)) & 1) == 1;
                        if bit {
                            if self.display.get_pixel(x as usize, y as usize) {
                                self.display.set_pixel(x as usize, y as usize, false);
                                self.v[0xF] = 1;
                            } else {
                                self.display.set_pixel(x as usize, y as usize, true);
                            }
                        }
                        x += 1;
                    }
                    y += 1;
                }
            }

            0xE => match instr.nn() {
                // skip next if key stored in VX is pressed
                0x9E => {
                    if pressed_inputs[self.v[instr.x()] as usize] {
                        self.pc += 2;
                    }
                }

                // skip next if key stored in VX is not pressed
                0xA1 => {
                    if !pressed_inputs[self.v[instr.x()] as usize] {
                        self.pc += 2;
                    }
                }
                _ => unimplemented!("{:x?}", instr),
            },

            0xF => match instr.nn() {
                // set VX to value of delay timer
                0x07 => self.v[instr.x()] = self.delay,

                // wait for key press, then store in VX
                //   blocks all instructions, delay and sound timers should continue processing
                0x0A => {
                    let pressed_key = pressed_inputs.iter().position(|input| *input);
                    if let Some(key) = pressed_key {
                        self.v[instr.x()] = key as u8;
                    } else {
                        self.pc -= 2;
                    }
                }

                // set delay timer to VX
                0x15 => self.delay = self.v[instr.x()],

                // set sound timer to VX
                0x18 => self.sound = self.v[instr.x()],

                // add VX to I (VF is not affected)
                0x1E => self.i += self.v[instr.x()] as u16,

                // set I to location of sprite for character in VX (lowest nibble)
                // Characters 0-F are represented by a 4x5 font
                0x29 => self.i = FONT_OFFSET as u16 + (self.v[instr.x()] as u16 * 5),

                // store binary-coded decimal representation of VX in memory
                //   hundreds digit at location I
                //       tens digit at location I+1
                //       ones digit at location I+2
                0x33 => {
                    let num = self.v[instr.x()];
                    self.memory[self.i as usize] = num / 100;
                    self.memory[self.i as usize + 1] = (num / 10) % 10;
                    self.memory[self.i as usize + 2] = num % 10;
                }

                // store from V0 to VX (including VX) in memory, starting at address I
                // offset from I is increased by 1 for each value written, but I itself is left unmodified
                0x55 => {
                    for index in 0..(instr.x() + 1) {
                        self.memory[self.i as usize + index] = self.v[index];
                    }
                }

                // fill from V0 to VX (including VX) with values from memory, starting at address I
                // offset from I is increased by 1 for each value read, but I itself is left unmodified
                0x65 => {
                    for index in 0..(instr.x() + 1) {
                        self.v[index] = self.memory[self.i as usize + index];
                    }
                }
                _ => unimplemented!("{:x?}", instr),
            },
            _ => unimplemented!("{:x?}", instr),
        };
    }

    pub fn tick_timers_and_check_beep(&mut self) -> bool {
        if self.delay > 0 {
            self.delay -= 1;
        }
        if self.sound > 0 {
            self.sound -= 1;
            true
        } else {
            false
        }
    }

    pub fn load_into_memory(&mut self, address: usize, data: Vec<u8>) {
        self.memory[address..(address + data.len())].copy_from_slice(&data);
    }
}

const FONT: [u8; 80] = [
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
