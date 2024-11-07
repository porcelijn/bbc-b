use bbc_b::memory::{Address, MemoryBus, ram::RAM};
use bbc_b::mos6502::{CPU, registers::Registers};

#[test]
fn test_recurse() {
  let mut mem = RAM::new();
  let addr = Address::from(0x0800);
  mem.load_bin_at("images/recurse.bin", addr);
  let mut cpu = CPU::new();
  cpu.registers.pc = addr;
  let stop = |cpu: &CPU, _: &dyn MemoryBus| cpu.registers.pc.to_u16() == 0x0816;
  cpu.run(&mut mem, &stop);
  let r = &cpu.registers;
  assert_eq!(r.a, 0);
  assert_eq!(r.x, 0);
  assert_eq!(r.y, 0);
  assert!(r.p.has::<'Z'>());
  assert_eq!(cpu.cycles, 1786);
  let mut b = Address::from(0x915); // start of b[16]
  for v in 0..16 {
    assert_eq!(mem.read(b), v);
    b.inc_by(1);
  }
}
