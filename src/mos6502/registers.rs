#[derive(Debug)]
pub struct Registers {
  pub a: u8,   // accumulator
  pub x: u8,
  pub y: u8,
  pub p: u8,   // status
  pub pc: u16, // program counter
  pub s: u8,   // page 0x01 stack pointer offset
}

impl Registers {
  pub const fn new() -> Self {
    Registers { a: 0, x: 0, y: 0, p: 0, pc: 0, s: 0xff }
  }
}
