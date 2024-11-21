mod addressing_modes;
mod alu;
pub mod disassemble;
mod instructions;
pub mod registers;

use disassemble::disassemble_with_address;
use instructions::{Instruction, handle_interrupt};
use registers::Registers;

use crate::memory::{Address, MemoryBus, read_address, slice};

#[derive(Debug)]
pub struct CPU {
  pub registers: Registers,
  pub cycles: u64,
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

#[allow(unused)]
pub fn stop_at<const ADDRESS: u16>(cpu: &CPU, mem: &dyn MemoryBus) -> bool {
  cpu.registers.pc.to_u16() == ADDRESS
}

impl CPU {
  const NMI_VECTOR:     u16 = 0xFFFA;
  const RESET_VECTOR:   u16 = 0xFFFC;
  const IRQ_BRK_VECTOR: u16 = 0xFFFE;
  pub fn new() -> Self {
    CPU { registers: Registers::new(), cycles: 0 }
  }

  pub fn step(&mut self, memory: &mut dyn MemoryBus) {
    let opcode = memory.read(self.registers.pc);
    let instruction = Instruction::lookup(opcode);
    instruction.execute(&mut self.registers, memory);
    self.cycles += 1;
//  self.trace(memory);
  }

  #[allow(unused)]
  pub fn handle_irq(&mut self, memory: &mut dyn MemoryBus) {
    self.registers.p.set_flag::<'B', false>();
    handle_interrupt::<{Self::IRQ_BRK_VECTOR}>(&mut self.registers, memory);
    self.cycles += 1;
  }

  #[allow(unused)]
  pub fn handle_nmi(&mut self, memory: &mut dyn MemoryBus) {
    self.registers.p.set_flag::<'B', false>();
    handle_interrupt::<{Self::NMI_VECTOR}>(&mut self.registers, memory);
    self.cycles += 1;
  }

  #[allow(unused)]
  pub fn reset(&mut self, memory: &mut dyn MemoryBus) {
    // https://www.pagetable.com/?p=410
    self.registers.a = 0xAA;
    *self.registers.s.borrow_mut() = 0xFD;           // cycles 0-5
    let address = Address::from(Self::RESET_VECTOR);
    let address = read_address(memory, address);     // cycles 6, 7
    self.registers.pc = address;                     // cycles 8, 9?
    self.cycles += 9;
  }

  pub fn run(&mut self, memory: &mut dyn MemoryBus, stop: &Breakpoint) {
    while !stop(&self, memory) {
      self.step(memory);
    }
  }

  #[allow(unused)]
  fn trace(&self, memory: &dyn MemoryBus) {
    let address = self.registers.pc;
    let operand = &slice(memory, address, 3);
    let disassembly = disassemble_with_address(address, operand);
    log::trace!("{disassembly: <40} {self:?}");
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
