use bbc_b::memory::{Address, MemoryBus, ram::RAM, slice};
use bbc_b::mos6502::{CPU, stop_after, stop_when};

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
  let b = slice(&mem, Address::from(0x915), 16); // start of b[16]
  assert_eq!(b, (0..16).collect::<Vec<u8>>());   // 0, 1, 2, 3, .. , 15
}

#[test]
fn test_fibo() {
  let mut mem = RAM::new();
  let addr = Address::from(0x0800);
  mem.load_bin_at("images/fibo_rec.bin", addr);
  let mut cpu = CPU::new();
  cpu.registers.pc = addr;
  cpu.run(&mut mem, &stop_when::<0x00>); // Stop on BRK
  let r = &cpu.registers;
  assert_eq!(r.a, 21); // = 8th fibonaci number
  assert_eq!(mem.read(Address::from(0x0080)), 0x08);
  assert_eq!(r.x, 0);
  assert_eq!(r.y, 1);
  assert!(!r.p.has::<'C'>());
  assert!(!r.p.has::<'V'>());
  assert!(!r.p.has::<'N'>());
  assert!(!r.p.has::<'Z'>());
}

#[test]
fn snake() {
  let mut mem = RAM::new();
  let (addr, _size) = mem.load_hex("images/snake.hex");
  let mut cpu = CPU::new();
  cpu.registers.pc = addr;
  cpu.run(&mut mem, &stop_after::<1_000_000>); // Stop on BRK
}

