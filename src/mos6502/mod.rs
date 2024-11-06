mod registers;
use registers::Registers;

#[derive(Debug)]
pub struct CPU {
  registers: Registers,
  cycles: u64,
}

impl CPU {
  pub fn new() -> Self {
    CPU { registers: Registers::new(), cycles: 0 }
  }

  pub fn step(&mut self, ticks: u64) {
    while self.cycles < ticks {
      self.registers.pc.inc_by(1);
      self.cycles += 1;
    }
  }
}
