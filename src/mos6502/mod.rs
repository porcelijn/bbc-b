mod addressing_modes;
mod alu;
pub mod instructions;
mod registers;

use registers::Registers;
use instructions::Instruction;

use crate::memory::MemoryBus;

#[derive(Debug)]
pub struct CPU {
  pub registers: Registers,
  cycles: u64,
}

type Breakpoint = dyn Fn(&CPU, &dyn MemoryBus) -> bool;

#[allow(unused)]
pub fn stop_when<const OPCODE: u8>(cpu: &CPU, mem: &dyn MemoryBus) -> bool {
  mem.read(cpu.registers.pc) == OPCODE
}

#[allow(unused)]
pub fn stop_after<const CYCLES: u64>(cpu: &CPU, mem: &dyn MemoryBus) -> bool {
  cpu.cycles >= CYCLES
}

impl CPU {
  pub fn new() -> Self {
    CPU { registers: Registers::new(), cycles: 0 }
  }

  pub fn step(&mut self, memory: &mut dyn MemoryBus) {
    let opcode = memory.read(self.registers.pc);
    let instruction = Instruction::lookup(opcode);
    instruction.execute(&mut self.registers, memory);
    self.cycles += 1;
  }

  pub fn run(&mut self, memory: &mut dyn MemoryBus, stop: &Breakpoint) {
    while !stop(&self, memory) {
      self.step(memory);
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
