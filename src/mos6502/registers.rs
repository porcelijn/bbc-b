use std::fmt;
use crate::memory::Address;

#[derive(Clone, Copy)]
pub struct Status(u8);

impl Status {
  const NEGATIVE: u8  = 0b1000_0000;
  const OVERFLOW: u8  = 0b0100_0000;
  const RESERVED: u8  = 0b0010_0000; // Always set
  const BREAK: u8     = 0b0001_0000;
  const DECIMAL: u8   = 0b0000_1000;
  const INTERRUPT: u8 = 0b0000_0100;
  const ZERO: u8      = 0b0000_0010;
  const CARRY: u8     = 0b0000_0001;

  pub const fn new() -> Self { Self::from(0) }
  pub const fn from(value: u8) -> Self {
    Status(value | Self::RESERVED) }

  pub const fn get_mask<const FLAG: char>() -> u8 {
    const {
      match FLAG {
        'b'|'B' => Status::BREAK,
        'c'|'C' => Status::CARRY,
        'd'|'D' => Status::DECIMAL,
        'i'|'I' => Status::INTERRUPT,
        'n'|'N' => Status::NEGATIVE,
        'v'|'V' => Status::OVERFLOW,
        'z'|'Z' => Status::ZERO,
        _ => unreachable!(),
      }
    }
  }

  pub const fn has<const FLAG: char>(&self) -> bool {
    let mask = const { Status::get_mask::<FLAG >() };
    self.0 & mask != 0
  }

  pub fn set<const FLAG: char>(&mut self, value: bool) {
    let mask = const { Status::get_mask::<FLAG >() };
    if value {
      self.0 |= mask;
    } else {
      self.0 &= !mask;
    }
  }

  pub fn set_flag<const FLAG: char, const SET: bool>(&mut self) {
    let mask = const { Status::get_mask::<FLAG >() };
    if SET {
      self.0 |= mask;
    } else {
      self.0 &= !mask;
    }
  }

  // Convenience function for loads, transfers, ALU ops..
  pub fn set_nz_from_u8(&mut self, value: u8) {
    self.set::<'Z'>(value == 0);
    self.set::<'N'>(value & 0b1000_0000 != 0);
  }

  pub const fn to_u8(&self) -> u8 {
    self.0
  }
}

impl fmt::Debug for Status {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fn write_bit(v: u8, c1: char, c2: char, f: &mut fmt::Formatter<'_>) {
      write!(f, "{}", match v { 0 => c1, _ => c2 }).unwrap();
    }

    write!(f, "Status(").unwrap();
    write_bit(self.0 & Status::NEGATIVE, 'n', 'N', f);
    write_bit(self.0 & Status::OVERFLOW, 'v', 'V', f);
    write!(f, "_").unwrap();
    write_bit(self.0 & Status::BREAK,    'b', 'B', f);
    write_bit(self.0 & Status::DECIMAL,  'd', 'D', f);
    write_bit(self.0 & Status::INTERRUPT,'i', 'I', f);
    write_bit(self.0 & Status::ZERO,     'z', 'Z', f);
    write_bit(self.0 & Status::CARRY,    'c', 'C', f);
    write!(f, ")").unwrap();
    Ok(())
  }
}

#[derive(Debug)]
pub struct StackPointer(u8);

impl StackPointer {
  pub const fn to_address(&self) -> Address {
    Address::from_le_bytes(self.0, 0x01)
  }

  pub const fn to_u8(&self) -> u8 {
    self.0
  }

  pub fn borrow_mut(&mut self) -> &mut u8 {
    &mut self.0
  }

  // pull: increment before read
  pub fn inc(&mut self) {
    self.0 = self.0.wrapping_add(1);
  }

  // push: decrement after write
  pub fn dec(&mut self) {
    self.0 = self.0.wrapping_sub(1);
  }
}

#[derive(Debug)]
pub struct Registers {
  pub a: u8,           // accumulator
  pub x: u8,
  pub y: u8,
  pub p: Status,       // status
  pub pc: Address,     // program counter
  pub s: StackPointer, // page 0x01 stack pointer offset
}

impl Registers {
  pub const fn new() -> Self {
    let status = Status::new();
    let stack_pointer = StackPointer(0xFF);
    let address = Address::from(0);
    Registers { a: 0, x: 0, y: 0, p: status, pc: address, s: stack_pointer }
  }
}
