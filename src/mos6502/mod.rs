mod addressing_modes;
mod alu;
mod instructions;
mod registers;

use registers::Registers;
use instructions::Instruction;

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
      let opcode = memory.read(self.registers.pc);
      let instruction = Instruction::lookup(opcode);
      instruction.execute(&mut self.registers, memory);
      self.cycles += 1;
    }
  }
}

pub fn stack_push(registers: &mut Registers, memory: &mut dyn MemoryBus, value: u8) {
  memory.write(registers.s.to_address(), value);
  registers.s.dec();
}

pub fn stack_pull(registers: &mut Registers, memory: &dyn MemoryBus) -> u8 {
  registers.s.inc();
  memory.read(registers.s.to_address())
}
