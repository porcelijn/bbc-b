mod mos6502;
mod memory;

use mos6502::CPU;
use memory::{Address, MemoryBus};
use memory::ram::{RAM, read_address, slice};

fn main() {
  println!("My first BBC-B emulator");
  let mut cpu = CPU::new();
  let mut ram = RAM::new();
  println!("- {:?}", cpu);
  cpu.step(&mut ram, 3);
  println!("- {:?}", cpu);
  ram.write(Address::from_le_bytes(03, 00), 0xAE);
  assert_eq!(read_address(&ram, cpu.registers.pc).lo_u8(), 0xAE);
  assert_eq!(read_address(&ram, cpu.registers.pc).hi_u8(), 0);
  assert_eq!(slice(&ram, cpu.registers.pc, 2), [0xAE, 0]);
}
