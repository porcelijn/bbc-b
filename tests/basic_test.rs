
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

