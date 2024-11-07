
use bbc_b::mos6502::{CPU, stop_when};
use bbc_b::memory::{Address, ram::RAM};

#[test]
fn test_program() {
  const PROGRAM: [u8; 38] = [
        // Code start
        0xA9, // LDA Immediate
        0x01, //     Immediate operand
        0x69, // ADC Immediate
        0x07, //     Immediate operand
        0x65, // ADC ZeroPage
        0x01, //     ZeroPage operand
        0xA2, // LDX Immediate
        0x01, //     Immediate operand
        0x75, // ADC ZeroPageX
        0x02, //     ZeroPageX operand
        0x6D, // ADC Absolute
        0x01, //     Absolute operand
        0x80, //     Absolute operand
        0xA2, // LDX immediate
        0x08, //     Immediate operand
        0x7D, // ADC AbsoluteX
        0x00, //     AbsoluteX operand
        0x80, //     AbsoluteX operand
        0xA0, // LDY immediate
        0x04, //     Immediate operand
        0x79, // ADC AbsoluteY
        0x00, //     AbsoluteY operand
        0x80, //     AbsoluteY operand
        0xA2, // LDX immediate
        0x05, //     Immediate operand
        0x61, // ADC IndexedIndirectX
        0x03, //     IndexedIndirectX operand
        0xA0, // LDY immediate
        0x10, //     Immediate operand
        0x71, // ADC IndirectIndexedY
        0x0F, //     IndirectIndexedY operand
        0x0A, // ASL A
        0xF0, // BEQ Relative
        0x02, //     pc + 2
        0x90, // BCC Relative
        0xFB, //     pc - 5
        0xEA, // NOP :)
        0xFF, // Something invalid -- the end!
    ];

  let start = Address::from(0);
  let mut ram = RAM::new();
  let _size = ram.load_at(&PROGRAM, start);
  let mut cpu = CPU::new();

  const NOP: u8 = 0xEA;
  cpu.run(&mut ram, &stop_when::<NOP>);
  let regs = &mut cpu.registers;
  assert_eq!(regs.a, 0);
  assert_eq!(regs.x, 5);
  assert_eq!(regs.y, 16);
  assert_eq!(regs.s.to_u8(), 0xFF);
  assert!(regs.p.has::<'C'>());
  assert!(!regs.p.has::<'V'>());
  assert!(!regs.p.has::<'N'>());
  assert!(regs.p.has::<'Z'>());
}

#[test]
fn another_test() {
  // addr instr     disass        |AC XR YR SP|nvdizc|#
  // EB14 38        SEC           |C0 07 00 F9|000111|2
  // EB15 E9 40     SBC #$40      |80 07 00 F9|100101|2
  // EB17 4A        LSR A         |40 07 00 F9|000100|2
  let mut ram = RAM::new();
  let start = Address::from(0xEB14);
  ram.load_at(&[ 0x38, 0xE9, 0x40, 0x4A ], start);
  let mut cpu = CPU::new();
  cpu.registers.a = 0xC0;
  cpu.registers.pc = start;
  cpu.registers.p.set::<'I'>(true);
  cpu.registers.p.set::<'Z'>(true);
  assert_eq!(cpu.registers.p.has::<'C'>(), false);
  cpu.step(&mut ram); // SEC
  assert_eq!(cpu.registers.p.has::<'C'>(), true);
  cpu.step(&mut ram);
  assert_eq!(cpu.registers.a, 0x80);
  assert_eq!(cpu.registers.p.has::<'C'>(), true);
  assert_eq!(cpu.registers.p.has::<'N'>(), true);
  assert_eq!(cpu.registers.p.has::<'V'>(), false);
  assert_eq!(cpu.registers.p.has::<'Z'>(), false);
  cpu.step(&mut ram);
  assert_eq!(cpu.registers.a, 0x40);
  assert_eq!(cpu.registers.p.has::<'C'>(), false);
  assert_eq!(cpu.registers.p.has::<'N'>(), false);
  assert_eq!(cpu.registers.p.has::<'V'>(), false);
  assert_eq!(cpu.registers.p.has::<'Z'>(), false);
}
  
