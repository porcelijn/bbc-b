use bbc_b::mos6502::{CPU, stop_after};
use bbc_b::memory::{Address, MemoryBus, ram::RAM, read_address, slice};

fn main() {
  println!("My first BBC-B emulator");
  let mut cpu = CPU::new();
  let mut ram = RAM::new();
  // poke IRQ/BRK vector 0xDEAD
  ram.write(Address::from_le_bytes(0xfe, 0xff), 0xAD);
  ram.write(Address::from(0xffff), 0xDE);
  let irq_vector = Address::from(0xFFFE);
  assert_eq!(slice(&ram, irq_vector, 2), [0xAD, 0xDE]);
  assert_eq!(read_address(&ram, irq_vector).to_u16(), 0xDEAD);
  println!("- {:?}", cpu);
  cpu.run(&mut ram, &stop_after::<3>); // execute BRK at 0x0000, 0xDEAD, 0xDEAD
  println!("- {:?}", cpu);
  assert_eq!(cpu.registers.pc, Address::from(0xDEAD));
  assert_eq!(cpu.registers.s.to_u8(), 0xFF - 3 * 3);
}
