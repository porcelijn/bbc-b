
use bbc_b::mos6502::{CPU, stop_after};
use bbc_b::mos6502::disassemble::disassemble_with_address;
use bbc_b::memory::{Address, ram::RAM, slice};

#[test]
fn os120_reset() {
  let mut ram = RAM::new();

  // manually compiled from https://tobylobster.github.io/mos/os120_acme.a
  ram.load_bin_at("images/os120.bin", Address::from(0xC000));
  let mut cpu = CPU::new();
  cpu.reset(&mut ram);
  assert_eq!(cpu.registers.pc, Address::from(0xD9CD)); // .resetEntryPoint
  for _ in 0..10 {
    let slice = slice(&mut ram, cpu.registers.pc, 3);
    let dump = disassemble_with_address(cpu.registers.pc, &slice);
    println!("{dump}");
    cpu.step(&mut ram);
  }
  {
    let r = &cpu.registers;
    assert_eq!(r.a, 210);
    assert_eq!(r.x, 255);
    assert_eq!(r.y, 0);
    assert_eq!(r.p.to_u8(), 0b1010_0100);
    assert_eq!(r.s.to_u8(), 254);
    assert_eq!(r.pc.to_u16(), 0xD9DE);
  }

  cpu.run(&mut ram, &stop_after::<100_000>);
}
