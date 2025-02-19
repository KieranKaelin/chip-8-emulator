use std::fmt;

#[derive(Debug)]
pub struct Instruction(u16);

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:x}{:x}{:x}{:x}",
            self.a(),
            self.x(),
            self.y(),
            self.n()
        )
    }
}

impl Instruction {
    pub fn new(instruction: u16) -> Self {
        Self(instruction)
    }

    pub fn a(&self) -> u8 {
        ((self.0) >> 12) as u8
    }

    pub fn x(&self) -> usize {
        ((self.0 & 0x0F00) >> 8) as usize
    }

    pub fn y(&self) -> usize {
        ((self.0 & 0x00F0) >> 4) as usize
    }

    pub fn n(&self) -> u8 {
        (self.0 & 0x000F) as u8
    }

    pub fn nn(&self) -> u8 {
        (self.0 & 0x00FF) as u8
    }

    pub fn nnn(&self) -> u16 {
        self.0 & 0x0FFF
    }
}
