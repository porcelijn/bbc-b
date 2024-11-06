mod registers;
use registers::Registers;

use crate::memory::MemoryBus;

#[derive(Debug)]
pub struct CPU {
  pub registers: Registers,
  cycles: u64,
}

impl CPU {
  pub fn new() -> Self {
    CPU { registers: Registers::new(), cycles: 0 }
  }

  pub fn step(&mut self, memory: &mut dyn MemoryBus, ticks: u64) {
    while self.cycles < ticks {
      memory.read(self.registers.pc);
      self.registers.pc.inc_by(1);
      self.cycles += 1;
    }
  }
}
