use std::fmt;
use crate::memory::Address;
use crate::memory::MemoryBus;
use crate::memory::read_address;

use crate::mos6502::{stack_push, stack_pull};

use crate::mos6502::registers::Status;
use crate::mos6502::alu;
use crate::mos6502::addressing_modes::UseMode;
use crate::mos6502::addressing_modes::UseAddress;
use crate::mos6502::addressing_modes::UseValue;
use crate::mos6502::addressing_modes::UseReference;

use crate::mos6502::addressing_modes::UseImplied;
use crate::mos6502::addressing_modes::UseAccumulator;
use crate::mos6502::addressing_modes::UseImmediate;
use crate::mos6502::addressing_modes::UseZeroPage;
use crate::mos6502::addressing_modes::UseZeroPageWith;
use crate::mos6502::addressing_modes::UseRelative;
use crate::mos6502::addressing_modes::UseAbsolute;
use crate::mos6502::addressing_modes::UseAbsoluteWith;
use crate::mos6502::addressing_modes::UseIndirect;
use crate::mos6502::addressing_modes::UseIndexedIndirectX;
use crate::mos6502::addressing_modes::UseIndirectIndexedY;
use crate::mos6502::registers::Registers;

pub enum Mnemonic {
  ADC, // ADd with Carry
  AND, // logical AND (bitwise)
  ASL, // Arithmetic Shift Left
  BCC, // Branch if Carry Clear
  BCS, // Branch if Carry Set
  BEQ, // Branch if Equal (to zero?)
  BIT, // BIT test
  BMI, // Branch if Minus
  BNE, // Branch if Not Equal
  BPL, // Branch if Positive
  _BRA, // Unconditional BRAnch
  BRK, // BReaK
  BVC, // Branch if oVerflow Clear
  BVS, // Branch if oVerflow Set
  CLC, // CLear Carry flag
  CLD, // Clear Decimal Mode
  CLI, // Clear Interrupt Disable
  CLV, // Clear oVerflow flag
  CMP, // Compare
  CPX, // Compare X register
  CPY, // Compare Y register
  DEC, // DECrement memory
  DEX, // DEcrement X register
  DEY, // DEcrement Y register
  EOR, // Exclusive OR (bitwise)
  INC, // INCrement memory
  INX, // INcrement X register
  INY, // INcrement Y register
  JMP, // JuMP
  JSR, // Jump to SubRoutine
  LDA, // LoaD Accumulator
  LDX, // LoaD X register
  LDY, // LoaD Y register
  LSR, // Logical Shift Right
  NOP, // No OPeration
  ORA, // inclusive OR (bitwise)
  PHA, // PusH Accumulator
  PHP, // PusH Processor status
  _PHX, // PusH X
  _PHY, // PusH Y
  PLA, // PuLl Accumulator
  PLP, // PuLl Processor status
  _PLX, // PuLl X
  _PLY, // PuLl Y
  ROL, // ROtate Left
  ROR, // ROtate Right
  RTI, // ReTurn from Interrupt
  RTS, // ReTurn from Subroutine
  SBC, // SuBtract with Carry
  SEC, // SEt Carry flag
  SED, // SEt Decimal flag
  SEI, // SEt Interrupt disable
  STA, // STore Accumulator
  STX, // STore X register
  STY, // STore Y register
  _STZ, // STore Zero
  TAX, // Transfer Accumulator to X
  TAY, // Transfer Accumulator to Y
  _TRB, // Test and Reset Bits
  _TSB, // Test and Set Bits
  TSX, // Transfer Stack pointer to X
  TXA, // Transfer X to Accumulator
  TXS, // Transfer X to Stack pointer
  TYA, // Transfer Y to Accumulator
}

impl Mnemonic {
  const fn to_str(&self) -> &'static str
  {
    match self {
      Self::ADC => "ADC",
      Self::AND => "AND",
      Self::ASL => "ASL",
      Self::BCC => "BCC",
      Self::BCS => "BCS",
      Self::BEQ => "BEQ",
      Self::BIT => "BIT",
      Self::BMI => "BMI",
      Self::BNE => "BNE",
      Self::BPL => "BPL",
      Self::_BRA => "BRA",
      Self::BRK => "BRK",
      Self::BVC => "BVC",
      Self::BVS => "BVS",
      Self::CLC => "CLC",
      Self::CLD => "CLD",
      Self::CLI => "CLI",
      Self::CLV => "CLV",
      Self::CMP => "CMP",
      Self::CPX => "CPX",
      Self::CPY => "CPY",
      Self::DEC => "DEC",
      Self::DEX => "DEX",
      Self::DEY => "DEY",
      Self::EOR => "EOR",
      Self::INC => "INC",
      Self::INX => "INX",
      Self::INY => "INY",
      Self::JMP => "JMP",
      Self::JSR => "JSR",
      Self::LDA => "LDA",
      Self::LDX => "LDX",
      Self::LDY => "LDY",
      Self::LSR => "LSR",
      Self::NOP => "NOP",
      Self::ORA => "ORA",
      Self::PHA => "PHA",
      Self::PHP => "PHP",
      Self::_PHX => "PHX",
      Self::_PHY => "PHY",
      Self::PLA => "PLA",
      Self::PLP => "PLP",
      Self::_PLX => "PLX",
      Self::_PLY => "PLY",
      Self::ROL => "ROL",
      Self::ROR => "ROR",
      Self::RTI => "RTI",
      Self::RTS => "RTS",
      Self::SBC => "SBC",
      Self::SEC => "SEC",
      Self::SED => "SED",
      Self::SEI => "SEI",
      Self::STA => "STA",
      Self::STX => "STX",
      Self::STY => "STY",
      Self::_STZ => "STZ",
      Self::TAX => "TAX",
      Self::TAY => "TAY",
      Self::_TRB => "TRB",
      Self::_TSB => "TSB",
      Self::TSX => "TSX",
      Self::TXA => "TXA",
      Self::TXS => "TXS",
      Self::TYA => "TYA",
    }
  }
}

impl fmt::Display for Mnemonic {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.to_str())
  }
}

pub enum AddressingMode {
   // work directly on accumulator, e. g. `lsr a`.
  Accumulator,

  // BRK
  Implied,

  // 8-bit constant in instruction, e. g. `lda #10`.
  Immediate,

  // zero-page address, e. g. `lda $00`.
  ZeroPage,

  // address is X register + 8-bit constant, e. g. `lda $80,x`.
  ZeroPageX,

  // address is Y register + 8-bit constant, e. g. `ldx $10,y`.
  ZeroPageY,

  // branch target as signed relative offset, e. g. `bne label`.
  Relative,

  // full 16-bit address, e. g. `jmp $1000`.
  Absolute,

  // full 16-bit address plus X register, e. g. `sta $1000,X`.
  AbsoluteX,

  // full 16-bit address plus Y register, e. g. `sta $1000,Y`.
  AbsoluteY,

  // jump to address stored at address, e. g. `jmp ($1000)`.
  Indirect,

  // load from address stored at (constant zero page address plus X register), e. g. `lda ($10,X)`.
  IndexedIndirectX,

  // load from (address stored at constant zero page address) plus Y register, e. g. `lda ($10),Y`.
  IndirectIndexedY,

  // Address stored at constant zero page address
  _ZeroPageIndirect,
}


macro_rules! static_dispatch_addressing_mode {
  ( $function:ident ( $( $arg:ident : $itype:ty ),* ) $( -> $otype: ty )? ) => {
    #[allow(unused)]
    pub fn $function (&self, $( $arg : $itype ),*) $( -> $otype )? {
      match self {
        AddressingMode::Accumulator => UseAccumulator::$function( $($arg),* ),
        AddressingMode::Implied => UseImplied::$function( $($arg),* ),
        AddressingMode::Immediate => UseImmediate::$function( $($arg),* ),
        AddressingMode::ZeroPage => UseZeroPage::$function( $($arg),* ),
        AddressingMode::ZeroPageX => UseZeroPageWith::<'X'>::$function( $($arg),* ),
        AddressingMode::ZeroPageY => UseZeroPageWith::<'Y'>::$function( $($arg),* ),
        AddressingMode::Relative => UseRelative::$function( $($arg),* ),
        AddressingMode::Absolute => UseAbsolute::$function( $($arg),* ),
        AddressingMode::AbsoluteX => UseAbsoluteWith::<'X'>::$function( $($arg),* ),
        AddressingMode::AbsoluteY => UseAbsoluteWith::<'Y'>::$function( $($arg),* ),
        AddressingMode::Indirect => UseIndirect::$function( $($arg),* ),
        AddressingMode::IndexedIndirectX => UseIndexedIndirectX::$function( $($arg),* ),
        AddressingMode::IndirectIndexedY => UseIndirectIndexedY::$function( $($arg),* ),
        _ => unimplemented!(),
      }
    }
  }
}

impl AddressingMode {
  static_dispatch_addressing_mode!(get_size() -> u8);
  static_dispatch_addressing_mode!(get_operand(bytes: &[u8]) -> String);
  static_dispatch_addressing_mode!(get_name() -> &'static str);
}

type Instr = fn(registers: &mut Registers, memory: &mut dyn MemoryBus);

trait AccOp {
  fn call(accumulator: &mut u8, status: &mut Status, value: u8);
}

struct Adc;
impl AccOp for Adc {
  fn call(accumulator: &mut u8, status: &mut Status, value: u8) {
    let (result, carry, overflow) =
      alu::add_with_carry(*accumulator, value, status.has::<'C'>());
    let negative = result & 0b0_1000_0000 != 0;

    *accumulator = result;
    status.set::<'C'>(carry);
    status.set::<'N'>(negative);
    status.set::<'V'>(overflow);
    status.set::<'Z'>(result == 0);
  }
}

#[test]
fn test_adc_70_plus_70() {
  let mut accumulator = 0x70;
  let mut status = Status::new();
  assert!(!status.has::<'C'>());
  Adc::call(&mut accumulator, &mut status, 0x70);
  assert_eq!(accumulator, 0xE0);
  assert!(!status.has::<'C'>());
  assert!(status.has::<'N'>());
  assert!(status.has::<'V'>());
  assert!(!status.has::<'Z'>());
}

#[test]
fn test_adc_d0_plus_90() {
  let mut accumulator = 0xD0;
  let mut status = Status::new();
  assert!(!status.has::<'C'>());
  Adc::call(&mut accumulator, &mut status, 0x90);
  assert_eq!(accumulator, 0x60);
  assert!(status.has::<'C'>());
  assert!(!status.has::<'N'>());
  assert!(status.has::<'V'>());
  assert!(!status.has::<'Z'>());
}

#[test]
fn test_adc_d1_plus_d1() {
  let mut accumulator = 0xD1;
  let mut status = Status::new();
  assert!(!status.has::<'C'>());
  Adc::call(&mut accumulator, &mut status, 0xD1);
  assert_eq!(accumulator, 0xA2);
  assert!(status.has::<'C'>());
  assert!(status.has::<'N'>());
  assert!(!status.has::<'V'>());
  assert!(!status.has::<'Z'>());
}


struct And;
impl AccOp for And {
  fn call(accumulator: &mut u8, status: &mut Status, value: u8) {
    let result = alu::and(*accumulator, value);
    status.set_nz_from_u8(result);
    *accumulator = result;
  }
}

struct Eor;
impl AccOp for Eor {
  fn call(accumulator: &mut u8, status: &mut Status, value: u8) {
    let result = alu::eor(*accumulator, value);
    status.set_nz_from_u8(result);
    *accumulator = result;
  }
}

struct Ora;
impl AccOp for Ora {
  fn call(accumulator: &mut u8, status: &mut Status, value: u8) {
    let result = alu::ora(*accumulator, value);
    status.set_nz_from_u8(result);
    *accumulator = result;
  }
}

struct Sbc;
impl AccOp for Sbc {
  fn call(accumulator: &mut u8, status: &mut Status, value: u8) {
    let (result, carry, overflow) =
      alu::subtract_with_carry(*accumulator, value, status.has::<'C'>());
    let negative = result & 0b0_1000_0000 != 0;

    *accumulator = result;
    status.set::<'C'>(carry);
    status.set::<'N'>(negative);
    status.set::<'V'>(overflow);
    status.set::<'Z'>(result == 0);
  }
}

#[test]
fn test_sbc_50_minus_f0() {
  let mut accumulator = 0x50;
  let mut status = Status::new();
  status.set_flag::<'Z', true>();
  status.set_flag::<'C', true>();
  Sbc::call(&mut accumulator, &mut status, 0xF0);
  assert_eq!(accumulator, 0x60);
  assert!(!status.has::<'C'>());
  assert!(!status.has::<'N'>());
  assert!(!status.has::<'V'>()); // + minus - = +, no overflow
  assert!(!status.has::<'Z'>());
}

#[test]
fn test_sbc_bf_minus_40() {
  let mut accumulator = 0xBF;
  let mut status = Status::new();
  status.set_flag::<'C', true>();
  Sbc::call(&mut accumulator, &mut status, 0x40);
  assert_eq!(accumulator, 0x7F);
  assert!(status.has::<'C'>()); // no borrow
  assert!(!status.has::<'N'>());
  assert!(!status.has::<'V'>());
  assert!(!status.has::<'Z'>());
}

#[test]
fn test_sbc_d0_minus_70() {
  //    D0 = 208 | -48
  //    70 = 112 | 112
  //    -------------- -
  // (1)60 = 96  |  96
  let mut accumulator = 0xd0;
  let mut status = Status::new();
  status.set_flag::<'C', true>();
  Sbc::call(&mut accumulator, &mut status, 0x70);
  assert_eq!(accumulator, 0x60);
  assert!(status.has::<'C'>()); // no borrow
  assert!(!status.has::<'N'>());
  assert!(!status.has::<'V'>());
  assert!(!status.has::<'Z'>());
}

fn by_acc<AO: AccOp, AM: UseMode + UseValue>(registers: &mut Registers, memory: &mut dyn MemoryBus) {
  let value = AM::get_value(&registers, memory);
  registers.pc.inc_by(AM::get_size());
  AO::call(&mut registers.a, &mut registers.p, value);
}

trait RefOp {
  fn call(data: &mut u8, status: &mut Status);
}

struct ShiftLeft<const CARRY_INTO_BIT0: bool>;
impl<const CARRY_INTO_BIT0: bool> RefOp for ShiftLeft<CARRY_INTO_BIT0> {
  fn call(data: &mut u8, status: &mut Status) {
    let (result, carry) = if CARRY_INTO_BIT0 {
      alu::rol(*data, status.has::<'C'>())
    } else {
      alu::asl(*data)
    };
    status.set::<'C'>(carry);
    status.set_nz_from_u8(result);
    *data = result;
  }
}

struct ShiftRight<const CARRY_INTO_BIT7: bool>;
impl<const CARRY_INTO_BIT7: bool> RefOp for ShiftRight<CARRY_INTO_BIT7> {
  fn call(data: &mut u8, status: &mut Status) {
    let (result, carry) = if CARRY_INTO_BIT7 {
      alu::ror(*data, status.has::<'C'>())
    } else {
      alu::lsr(*data)
    };
    status.set::<'C'>(carry);
    status.set_nz_from_u8(result);
    *data = result;
  }
}

struct Increment;
impl RefOp for Increment {
  fn call(data: &mut u8, status: &mut Status) {
    let result = alu::inc(*data);
    status.set_nz_from_u8(result);
    *data = result;
  }
}

struct Decrement;
impl RefOp for Decrement {
  fn call(data: &mut u8, status: &mut Status) {
    let result = alu::dec(*data);
    status.set_nz_from_u8(result);
    *data = result;
  }
}

fn by_ref<RO: RefOp, AM: UseMode + UseReference + UseValue>(registers: &mut Registers, memory: &mut dyn MemoryBus) {
  let mut value = AM::get_value(registers, memory);
  RO::call(&mut value, &mut registers.p);
  AM::write(registers, memory, value);
  registers.pc.inc_by(AM::get_size());
}

fn compare<const REGISTER: char, AM: UseMode + UseValue>(registers: &mut Registers, memory: &mut dyn MemoryBus) {
  let lhs = match REGISTER {
    'a'|'A' => registers.a,
    'x'|'X' => registers.x,
    'y'|'Y' => registers.y,
    _ => unimplemented!()
  };
  let rhs = AM::get_value(&registers, memory);
  const CARRY: bool = true;

  let (result, carry, overflow) =
    alu::subtract_with_carry(lhs, rhs, CARRY);
  let negative = result & 0b0_1000_0000 != 0;

  registers.p.set::<'C'>(carry);
  registers.p.set::<'N'>(negative);
  registers.p.set::<'V'>(overflow);
  registers.p.set::<'Z'>(result == 0);

  registers.pc.inc_by(AM::get_size());
}

fn bit<AM: UseMode + UseValue>(registers: &mut Registers, memory: &mut dyn MemoryBus) {
  let value = AM::get_value(&registers, memory);
  let status = alu::bit(registers.a, value, registers.p);
  registers.p = status;
  registers.pc.inc_by(AM::get_size());
}

fn inc_register<const XY: char, AM: UseMode>(registers: &mut Registers, _: &mut dyn MemoryBus) {
  let register: &mut u8 = match XY {
    'x'|'X' => &mut registers.x,
    'y'|'Y' => &mut registers.y,
    _ => unreachable!()
  };
  let result = alu::inc(*register);
  registers.p.set_nz_from_u8(result);
  *register = result;
  registers.pc.inc_by(AM::get_size());
}

fn dec_register<const XY: char, AM: UseMode>(registers: &mut Registers, _: &mut dyn MemoryBus) {
  let register: &mut u8 = match XY {
    'x'|'X' => &mut registers.x,
    'y'|'Y' => &mut registers.y,
    _ => unreachable!()
  };
  let result = alu::dec(*register);
  registers.p.set_nz_from_u8(result);
  *register = result;
  registers.pc.inc_by(AM::get_size());
}

fn jump<AM: UseMode + UseAddress>(registers: &mut Registers, memory: &mut dyn MemoryBus) {
  let jump_address = AM::get_address(&registers, memory);
//registers.pc.inc_by(AM::get_size());
  registers.pc = jump_address;
}

fn jump_sub<AM: UseMode + UseAddress>(registers: &mut Registers, memory: &mut dyn MemoryBus) {
  let jump_address = AM::get_address(&registers, memory);
  let mut return_address = registers.pc;
  return_address.inc_by(AM::get_size()); // next instruction
  return_address.dec_by(1);              // 6502 pushes last byte of instruction!
  stack_push(registers, memory, return_address.hi_u8());
  stack_push(registers, memory, return_address.lo_u8());
  registers.pc = jump_address;
}

fn return_sub<AM: UseMode>(registers: &mut Registers, memory: &mut dyn MemoryBus) {
  //registers.pc.inc_by(AM::get_size());
  let lo = stack_pull(registers, memory);
  let hi = stack_pull(registers, memory);
  let return_address = Address::from_le_bytes(lo, hi); // last byte of JSR instruction!

  registers.pc = return_address.next();
}

fn return_interrupt<AM: UseMode>(registers: &mut Registers, memory: &mut dyn MemoryBus) {
//registers.pc.inc_by(AM::get_size());

  let status = stack_pull(registers, memory);
  let lo = stack_pull(registers, memory);
  let hi = stack_pull(registers, memory);

  registers.p = Status::from(status);
  registers.p.set::<'b'>(false); // clear BREAK flag
  registers.pc = Address::from_le_bytes(lo, hi);
}

fn branch<const FLAG: char, const SET: bool, AM: UseMode>(registers: &mut Registers, memory: &mut dyn MemoryBus) {
  let address = UseRelative::get_address(registers, memory);
  registers.pc.inc_by(AM::get_size());
  let address = address.next();
  alu::branch::<FLAG, SET>(&mut registers.pc, &registers.p, address);
}

fn set_flag<const FLAG: char, const SET: bool, AM: UseMode>(registers: &mut Registers, _: &mut dyn MemoryBus) {
  registers.p.set_flag::<FLAG, SET>();
}

fn load<const REGISTER: char, AM: UseMode + UseValue>(registers: &mut Registers, memory: &mut dyn MemoryBus) {
  let value = AM::get_value(&registers, memory);
  let register_ref: &mut u8 = match REGISTER {
    'a'|'A' => &mut registers.a,
    'x'|'X' => &mut registers.x,
    'y'|'Y' => &mut registers.y,
    _ => unimplemented!()
  };
  registers.p.set_nz_from_u8(value);
  *register_ref = value;
  registers.pc.inc_by(AM::get_size());
}

fn transfer<const FROM: char, const TO: char, AM: UseMode>(registers: &mut Registers, _: &mut dyn MemoryBus) {
  const fn value<const FROM: char>(registers: &Registers) -> u8 {
    match FROM {
      'a'|'A' => registers.a,
      'x'|'X' => registers.x,
      'y'|'Y' => registers.y,
      's'|'S' => registers.s.to_u8(),
      _       => unimplemented!()
    }
  }

  let value = value::<FROM>(registers);
  
  let reference = match TO {
    'a'|'A' => &mut registers.a,
    'x'|'X' => &mut registers.x,
    'y'|'Y' => &mut registers.y,
    's'|'S' => registers.s.borrow_mut(),
    _       => unimplemented!()
  };
 
  registers.p.set_nz_from_u8(value);
  *reference = value;
  registers.pc.inc_by(AM::get_size());
}
 
fn push_register<const REGISTER: char, AM: UseMode>(registers: &mut Registers, memory: &mut dyn MemoryBus) {
  const fn value<const REGISTER: char>(registers: &Registers) -> u8 {
    match REGISTER {
      'a'|'A' => registers.a,
      'p'|'P' => {
        // B is 0 when pushed by interrupts (NMI and IRQ) and 1 when pushed by
        // instructions (BRK and PHP).
        Status::get_mask::<'B'>() | registers.p.to_u8()
      },
      _       => unimplemented!()
    }
  }
  let value = value::<REGISTER>(registers);
  if REGISTER == 'p' || REGISTER == 'P' {
    //println!("{:b} == {:b}", value ,0b00010000| registers.p.to_u8());
    assert_eq!(value, 0b00010000|registers.p.to_u8());
  }

  stack_push(registers, memory, value);
  registers.pc.inc_by(AM::get_size());
}

fn pull_accumulator<AM: UseMode>(registers: &mut Registers, memory: &mut dyn MemoryBus) {
  let value = stack_pull(registers, memory);
  registers.a = value;
  registers.p.set_nz_from_u8(value);
  registers.pc.inc_by(AM::get_size());
}

fn pull_status<AM: UseMode>(registers: &mut Registers, memory: &mut dyn MemoryBus) {
  let value = stack_pull(registers, memory);
  registers.p = Status::from(value);
  assert_eq!(registers.p.to_u8(), value);
  registers.pc.inc_by(AM::get_size());
}

// fn pull_register(..): See inline
fn store<const REGISTER: char, AM: UseMode + UseAddress>(registers: &mut Registers, memory: &mut dyn MemoryBus) {
  const fn value<const REGISTER: char>(registers: &Registers) -> u8 {
    match REGISTER {
      'a'|'A' => registers.a,
      'x'|'X' => registers.x,
      'y'|'Y' => registers.y,
      _       => unimplemented!()
    } 
  }

  let value = value::<REGISTER>(registers);
  let address = AM::get_address(registers, memory);
  memory.write(address, value);
  registers.pc.inc_by(AM::get_size());
}

fn not_implemented<AM: UseMode>(registers: &mut Registers, memory: &mut dyn MemoryBus) {
  // skip back one byte before operand
  let mut address = registers.pc;
  address.dec_by(1);
  let operation = memory.read(address);
  // decode
  let instruction = Instruction::lookup(operation);
  let mnemonic = &instruction.mnemonic;
  let mode_name = instruction.addressing_mode.get_name();
  unimplemented!("{operation:#04x} {mnemonic} {mode_name}");
}

fn no_operation<AM: UseMode>(registers: &mut Registers, _: &mut dyn MemoryBus) {
//panic!("NOP instruction");
  registers.pc.inc_by(AM::get_size());
}
fn handle_interrupt<const VECTOR: u16>(registers: &mut Registers, memory: &mut dyn MemoryBus) {
  stack_push(registers, memory, registers.pc.hi_u8());
  stack_push(registers, memory, registers.pc.lo_u8());
  stack_push(registers, memory, registers.p.to_u8());
  registers.pc = read_address(memory, Address::from(VECTOR));
  registers.p.set_flag::<'I', true>();
}

fn handle_brk(registers: &mut Registers, memory: &mut dyn MemoryBus) {
//panic!("BRK instruction");
  registers.p.set_flag::<'B', true>();
  handle_interrupt::<0xFFFE>(registers, memory);
}

#[allow(unused)]
pub fn handle_irq(registers: &mut Registers, memory: &mut dyn MemoryBus) {
  registers.p.set_flag::<'B', false>();
  handle_interrupt::<0xFFFE>(registers, memory);
}

#[allow(unused)]
pub fn handle_nmi(registers: &mut Registers, memory: &mut dyn MemoryBus) {
  registers.p.set_flag::<'B', false>();
  handle_interrupt::<0xFFFA>(registers, memory);
}

pub struct Instruction {
  pub mnemonic: Mnemonic,
  pub addressing_mode: AddressingMode,
  pub instr: Instr,
}

impl Instruction {
  pub const fn new(mnemonic: Mnemonic, addressing_mode: AddressingMode, instr: Instr) -> Instruction {
    Instruction { mnemonic, addressing_mode, instr }
  }

  pub const fn lookup(byte: u8) -> &'static Instruction {
    &INSTRUCTIONS[byte as usize]
  }

  pub fn execute(&self, registers: &mut Registers, memory: &mut dyn MemoryBus) {
    registers.pc = registers.pc.next();
    (self.instr)(registers, memory);
  }
}

use Mnemonic::*;
const UND: Instruction = Instruction::new(NOP, AddressingMode::Implied, not_implemented::<UseImplied>);
const INSTRUCTIONS: [Instruction; 256] = [
  Instruction::new(BRK, AddressingMode::Implied, handle_brk), //0x00
  Instruction::new(ORA, AddressingMode::IndexedIndirectX, by_acc::<Ora, UseIndexedIndirectX>),
  UND,
  UND,
  UND, // 0x04 TSB, ZeroPage
  Instruction::new(ORA, AddressingMode::ZeroPage, by_acc::<Ora, UseZeroPage>),
  Instruction::new(ASL, AddressingMode::ZeroPage, by_ref::<ShiftLeft<false>, UseZeroPage>),
  UND,
  Instruction::new(PHP, AddressingMode::Implied, push_register::<'p', UseImplied>), // 0x08
  Instruction::new(ORA, AddressingMode::Immediate, by_acc::<Ora, UseImmediate>),
  Instruction::new(ASL, AddressingMode::Accumulator, by_ref::<ShiftLeft<false>, UseAccumulator>),
  UND,
  UND, // 0x0c TSB, absolute
  Instruction::new(ORA, AddressingMode::Absolute, by_acc::<Ora, UseAbsolute>),
  Instruction::new(ASL, AddressingMode::Absolute, by_ref::<ShiftLeft<false>, UseAbsolute>),
  UND,
  Instruction::new(BPL, AddressingMode::Relative, branch::<'n', false, UseRelative>), // 0x10
  Instruction::new(ORA, AddressingMode::IndirectIndexedY, by_acc::<Ora, UseIndirectIndexedY>),
  UND,
  UND,
  UND, // 0x14 TRB, zero page
  Instruction::new(ORA, AddressingMode::ZeroPageX, by_acc::<Ora, UseZeroPageWith<'X'>>),
  Instruction::new(ASL, AddressingMode::ZeroPageX, by_ref::<ShiftLeft<false>, UseZeroPageWith::<'X'>>),
  UND,
  Instruction::new(CLC, AddressingMode::Implied, set_flag::<'C', false, UseImplied>), // 0x18
  Instruction::new(ORA, AddressingMode::AbsoluteY, by_acc::<Ora, UseAbsoluteWith<'Y'>>),
  UND, // 0x1a INC, Accumulator
  UND,
  UND, // 0x1c TRB, absolute
  Instruction::new(ORA, AddressingMode::AbsoluteX, by_acc::<Ora, UseAbsoluteWith<'X'>>),
  Instruction::new(ASL, AddressingMode::AbsoluteX, by_ref::<ShiftLeft<false>, UseAbsoluteWith::<'X'>>),
  UND,
  Instruction::new(JSR, AddressingMode::Absolute, jump_sub::<UseAbsolute>), // 0x20
  Instruction::new(AND, AddressingMode::IndexedIndirectX, by_acc::<And, UseIndexedIndirectX>),
  UND,
  UND,
  Instruction::new(BIT, AddressingMode::ZeroPage, bit::<UseZeroPage>),
  Instruction::new(AND, AddressingMode::ZeroPage, by_acc::<And, UseZeroPage>),
  Instruction::new(ROL, AddressingMode::ZeroPage, by_ref::<ShiftLeft<true>, UseZeroPage>),
  UND,
  Instruction::new(PLP, AddressingMode::Implied, pull_status::<UseImplied>), // 0x28
  Instruction::new(AND, AddressingMode::Immediate, by_acc::<And, UseImmediate>),
  Instruction::new(ROL, AddressingMode::Accumulator, by_ref::<ShiftLeft<true>, UseAccumulator>),
  UND,
  Instruction::new(BIT, AddressingMode::Absolute, bit::<UseAbsolute>),
  Instruction::new(AND, AddressingMode::Absolute, by_acc::<And, UseAbsolute>),
  Instruction::new(ROL, AddressingMode::Absolute, by_ref::<ShiftLeft<true>, UseAbsolute>),
  UND,
  Instruction::new(BMI, AddressingMode::Relative, branch::<'N', true, UseRelative>), // 0x30
  Instruction::new(AND, AddressingMode::IndirectIndexedY, by_acc::<And, UseIndirectIndexedY>),
  UND,
  UND,
  UND,
  Instruction::new(AND, AddressingMode::ZeroPageX, by_acc::<And, UseZeroPageWith<'X'>>),
  Instruction::new(ROL, AddressingMode::ZeroPageX, by_ref::<ShiftLeft<true>, UseZeroPageWith<'X'>>),
  UND,
  Instruction::new(SEC, AddressingMode::Implied, set_flag::<'C', true, UseImplied>), // 0x38
  Instruction::new(AND, AddressingMode::AbsoluteY, by_acc::<And, UseAbsoluteWith<'Y'>>),
  UND, // DEC accumulator // 0x3a
  UND,
  UND,
  Instruction::new(AND, AddressingMode::AbsoluteX, by_acc::<And, UseAbsoluteWith<'X'>>),
  Instruction::new(ROL, AddressingMode::AbsoluteX, by_ref::<ShiftLeft<true>, UseAbsoluteWith<'X'>>), //0x3e
  UND,
  Instruction::new(RTI, AddressingMode::Implied, return_interrupt::<UseImplied>),
  Instruction::new(EOR, AddressingMode::IndexedIndirectX, by_acc::<Eor, UseIndexedIndirectX>),
  UND, // 0x42
  UND, // 0x43
  UND, // 0x44
  Instruction::new(EOR, AddressingMode::ZeroPage, by_acc::<Eor, UseZeroPage>),
  Instruction::new(LSR, AddressingMode::ZeroPage, by_ref::<ShiftRight<false>, UseZeroPage>),
  UND, // 0x47
  Instruction::new(PHA, AddressingMode::Implied, push_register::<'A', UseImplied>),
  Instruction::new(EOR, AddressingMode::Immediate, by_acc::<Eor, UseImmediate>),
  Instruction::new(LSR, AddressingMode::Accumulator, by_ref::<ShiftRight<false>, UseAccumulator>),
  UND, // 0x4b
  Instruction::new(JMP, AddressingMode::Absolute, jump::<UseAbsolute>),
  Instruction::new(EOR, AddressingMode::Absolute, by_acc::<Eor, UseAbsolute>),
  Instruction::new(LSR, AddressingMode::Absolute, by_ref::<ShiftRight<false>, UseAbsolute>),
  UND, // 0x4f
  Instruction::new(BVC, AddressingMode::Relative, branch::<'V', false, UseRelative>),
  Instruction::new(EOR, AddressingMode::IndirectIndexedY, by_acc::<Eor, UseIndirectIndexedY>),
  UND, // 0x52
  UND, // 0x53
  UND, // 0x54
  Instruction::new(EOR, AddressingMode::ZeroPageX, by_acc::<Eor, UseZeroPageWith<'X'>>),
  Instruction::new(LSR, AddressingMode::ZeroPageX, by_ref::<ShiftRight<false>, UseZeroPageWith<'X'>>),
  UND, // 0x57
  Instruction::new(CLI, AddressingMode::Implied, set_flag::<'I', false, UseImplied>),
  Instruction::new(EOR, AddressingMode::AbsoluteY, by_acc::<Eor, UseAbsoluteWith<'Y'>>),
  UND, // 0x5a
  UND, // 0x5b
  UND, // 0x5c
  Instruction::new(EOR, AddressingMode::AbsoluteX, by_acc::<Eor, UseAbsoluteWith<'X'>>),
  Instruction::new(LSR, AddressingMode::AbsoluteX, by_ref::<ShiftRight<false>, UseAbsoluteWith<'X'>>),
  UND, // 0x5f
  Instruction::new(RTS, AddressingMode::Implied, return_sub::<UseImplied>),
  Instruction::new(ADC, AddressingMode::IndexedIndirectX, by_acc::<Adc, UseIndexedIndirectX>),
  UND, // 0x62
  UND, // 0x63
  UND, // 0x64 STZ, ZeroPage
  Instruction::new(ADC, AddressingMode::ZeroPage, by_acc::<Adc, UseZeroPage>),
  Instruction::new(ROR, AddressingMode::ZeroPage, by_ref::<ShiftRight<true>, UseZeroPage>),
  UND, // 0x67
  Instruction::new(PLA, AddressingMode::Implied, pull_accumulator::<UseImplied>),
  Instruction::new(ADC, AddressingMode::Immediate, by_acc::<Adc, UseImmediate>),
  Instruction::new(ROR, AddressingMode::Accumulator, by_ref::<ShiftRight<true>, UseAccumulator>),
  UND, // 0x6b
  Instruction::new(JMP, AddressingMode::Indirect, jump::<UseIndirect>),
  Instruction::new(ADC, AddressingMode::Absolute, by_acc::<Adc, UseAbsolute>),
  Instruction::new(ROR, AddressingMode::Absolute, by_ref::<ShiftRight<true>, UseAbsolute>),
  UND, // 0x6f
  Instruction::new(BVS, AddressingMode::Relative, branch::<'V', true, UseRelative>),
  Instruction::new(ADC, AddressingMode::IndirectIndexedY, by_acc::<Adc, UseIndirectIndexedY>),
  UND, // 0x72
  UND, // 0x73
  UND, // 0x74 STZ, ZeroPageX
  Instruction::new(ADC, AddressingMode::ZeroPageX, by_acc::<Adc, UseZeroPageWith<'X'>>),
  Instruction::new(ROR, AddressingMode::ZeroPageX, by_ref::<ShiftRight<true>, UseZeroPageWith<'X'>>),
  UND, // 0x77
  Instruction::new(SEI, AddressingMode::Implied, set_flag::<'I', true, UseImplied>),
  Instruction::new(ADC, AddressingMode::AbsoluteY, by_acc::<Adc, UseAbsoluteWith<'Y'>>),
  UND, // 0x7a (PLY, implied)
  UND, // 0x7b
  UND, // 0x7c
  Instruction::new(ADC, AddressingMode::AbsoluteX, by_acc::<Adc, UseAbsoluteWith<'X'>>),
  Instruction::new(ROR, AddressingMode::AbsoluteX, by_ref::<ShiftRight<true>, UseAbsoluteWith<'X'>>),
  UND, // 0x7f
  UND, // 0x80
  Instruction::new(STA, AddressingMode::IndexedIndirectX, store::<'a', UseIndexedIndirectX>),
  UND, // 0x82
  UND, // 0x83
  Instruction::new(STY, AddressingMode::ZeroPage, store::<'y', UseZeroPage>),
  Instruction::new(STA, AddressingMode::ZeroPage, store::<'a', UseZeroPage>),
  Instruction::new(STX, AddressingMode::ZeroPage, store::<'x', UseZeroPage>),
  UND, // 0x87
  Instruction::new(DEY, AddressingMode::Implied, dec_register::<'Y', UseImplied>),
  UND, // 0x89
  Instruction::new(TXA, AddressingMode::Implied, transfer::<'X', 'A', UseImplied>),
  UND, // 0x8b
  Instruction::new(STY, AddressingMode::Absolute, store::<'y', UseAbsolute>),
  Instruction::new(STA, AddressingMode::Absolute, store::<'a', UseAbsolute>),
  Instruction::new(STX, AddressingMode::Absolute, store::<'x', UseAbsolute>),
  UND, // 0x8f
  Instruction::new(BCC, AddressingMode::Relative, branch::<'c', false, UseRelative>),
  Instruction::new(STA, AddressingMode::IndirectIndexedY, store::<'a', UseIndirectIndexedY>),
  UND, // 0x92
  UND, // 0x93
  Instruction::new(STY, AddressingMode::ZeroPageX, store::<'y', UseZeroPageWith<'X'>>),
  Instruction::new(STA, AddressingMode::ZeroPageX, store::<'a', UseZeroPageWith<'X'>>),
  Instruction::new(STX, AddressingMode::ZeroPageY, store::<'x', UseZeroPageWith<'Y'>>),
  UND, // 0x97
  Instruction::new(TYA, AddressingMode::Implied, transfer::<'Y', 'A', UseImplied>),
  Instruction::new(STA, AddressingMode::AbsoluteY, store::<'A', UseAbsoluteWith<'Y'>>),
  Instruction::new(TXS, AddressingMode::Implied, transfer::<'X', 'S', UseImplied>),
  UND, // 0x9b
  UND, // 0x9c STZ, Absolute
  Instruction::new(STA, AddressingMode::AbsoluteX, store::<'a', UseAbsoluteWith<'X'>>),
  UND, // 0x9e STZ, AbsoluteX
  UND, // 0x9f
  Instruction::new(LDY, AddressingMode::Immediate, load::<'y', UseImmediate>),
  Instruction::new(LDA, AddressingMode::IndexedIndirectX, load::<'a', UseIndexedIndirectX>),
  Instruction::new(LDX, AddressingMode::Immediate, load::<'x', UseImmediate>),
  UND, // 0xa3
  Instruction::new(LDY, AddressingMode::ZeroPage, load::<'y', UseZeroPage>),
  Instruction::new(LDA, AddressingMode::ZeroPage, load::<'a', UseZeroPage>),
  Instruction::new(LDX, AddressingMode::ZeroPage, load::<'x', UseZeroPage>),
  UND, // 0xa7
  Instruction::new(TAY, AddressingMode::Implied, transfer::<'A', 'Y', UseImplied>),
  Instruction::new(LDA, AddressingMode::Immediate, load::<'a', UseImmediate>),
  Instruction::new(TAX, AddressingMode::Implied, transfer::<'A', 'X', UseImplied>),
  UND, // 0xab
  Instruction::new(LDY, AddressingMode::Absolute, load::<'Y', UseAbsolute>),
  Instruction::new(LDA, AddressingMode::Absolute, load::<'A', UseAbsolute>),
  Instruction::new(LDX, AddressingMode::Absolute, load::<'X', UseAbsolute>),
  UND, // 0xaf
  Instruction::new(BCS, AddressingMode::Relative, branch::<'c', true, UseRelative>),
  Instruction::new(LDA, AddressingMode::IndirectIndexedY, load::<'a', UseIndirectIndexedY>),
  UND, // 0xb2
  UND, // 0xb3
  Instruction::new(LDY, AddressingMode::ZeroPageX, load::<'Y', UseZeroPageWith<'X'>>),
  Instruction::new(LDA, AddressingMode::ZeroPageX, load::<'a', UseZeroPageWith<'X'>>),
  Instruction::new(LDX, AddressingMode::ZeroPageY, load::<'x', UseZeroPageWith<'Y'>>),
  UND, // 0xb7
  Instruction::new(CLV, AddressingMode::Implied, set_flag::<'V', false, UseImplied>),
  Instruction::new(LDA, AddressingMode::AbsoluteY, load::<'a', UseAbsoluteWith<'Y'>>),
  Instruction::new(TSX, AddressingMode::Implied, transfer::<'S', 'X', UseImplied>),
  UND, // 0xbb
  Instruction::new(LDY, AddressingMode::AbsoluteX, load::<'Y', UseAbsoluteWith<'X'>>),
  Instruction::new(LDA, AddressingMode::AbsoluteX, load::<'a', UseAbsoluteWith<'X'>>),
  Instruction::new(LDX, AddressingMode::AbsoluteY, load::<'X', UseAbsoluteWith<'Y'>>),
  UND, // 0xbf
  Instruction::new(CPY, AddressingMode::Immediate, compare::<'Y', UseImmediate>),
  Instruction::new(CMP, AddressingMode::IndexedIndirectX, compare::<'A', UseIndexedIndirectX>),
  UND, // 0xc2
  UND, // 0xc3
  Instruction::new(CPY, AddressingMode::ZeroPage, compare::<'Y', UseZeroPage>),
  Instruction::new(CMP, AddressingMode::ZeroPage, compare::<'A', UseZeroPage>),
  Instruction::new(DEC, AddressingMode::ZeroPage, by_ref::<Decrement, UseZeroPage>),
  UND, // 0xc7
  Instruction::new(INY, AddressingMode::Implied, inc_register::<'Y', UseImplied>),
  Instruction::new(CMP, AddressingMode::Immediate, compare::<'A', UseImmediate>),
  Instruction::new(DEX, AddressingMode::Implied, dec_register::<'X', UseImplied>),
  UND, // 0xcb
  Instruction::new(CPY, AddressingMode::Absolute, compare::<'Y', UseAbsolute>),
  Instruction::new(CMP, AddressingMode::Absolute, compare::<'A', UseAbsolute>),
  Instruction::new(DEC, AddressingMode::Absolute, by_ref::<Decrement, UseAbsolute>),
  UND, // 0xcf
  Instruction::new(BNE, AddressingMode::Relative, branch::<'z', false, UseRelative>),
  Instruction::new(CMP, AddressingMode::IndirectIndexedY, compare::<'A', UseIndirectIndexedY>),
  UND, // 0xd2
  UND, // 0xd3
  UND, // 0xd4
  Instruction::new(CMP, AddressingMode::ZeroPageX, compare::<'A', UseZeroPageWith<'X'>>),
  Instruction::new(DEC, AddressingMode::ZeroPageX, by_ref::<Decrement, UseZeroPageWith<'X'>>),
  UND, // 0xd7
  Instruction::new(CLD, AddressingMode::Implied, set_flag::<'D', false, UseImplied>),
  Instruction::new(CMP, AddressingMode::AbsoluteY, compare::<'A', UseAbsoluteWith<'Y'>>),
  UND, // 0xda
  UND, // 0xdb
  UND, // 0xdc
  Instruction::new(CMP, AddressingMode::AbsoluteX, compare::<'A', UseAbsoluteWith<'X'>>),
  Instruction::new(DEC, AddressingMode::AbsoluteX, by_ref::<Decrement, UseAbsoluteWith<'X'>>),
  UND, // 0xdf
  Instruction::new(CPX, AddressingMode::Immediate, compare::<'X', UseImmediate>),
  Instruction::new(SBC, AddressingMode::IndexedIndirectX, not_implemented::<UseIndexedIndirectX>),
  UND, // 0xe2
  UND, // 0xe3
  Instruction::new(CPX, AddressingMode::ZeroPage, compare::<'X', UseZeroPage>),
  Instruction::new(SBC, AddressingMode::ZeroPage, by_acc::<Sbc, UseZeroPage>),
  Instruction::new(INC, AddressingMode::ZeroPage, by_ref::<Increment, UseZeroPage>),
  UND, // 0xe7
  Instruction::new(INX, AddressingMode::Implied, inc_register::<'X', UseImplied>),
  Instruction::new(SBC, AddressingMode::Immediate, by_acc::<Sbc, UseImmediate>),
  Instruction::new(NOP, AddressingMode::Implied, no_operation::<UseImplied>),
  UND, // 0xeb
  Instruction::new(CPX, AddressingMode::Absolute, compare::<'X', UseAbsolute>),
  Instruction::new(SBC, AddressingMode::Absolute, by_acc::<Sbc, UseAbsolute>),
  Instruction::new(INC, AddressingMode::Absolute, by_ref::<Increment, UseAbsolute>),
  UND, // 0xef
  Instruction::new(BEQ, AddressingMode::Relative, branch::<'Z', true, UseRelative>),
  Instruction::new(SBC, AddressingMode::IndirectIndexedY, by_acc::<Sbc, UseIndirectIndexedY>),
  UND, // 0xf2 SBC, ZeroPageIndirect
  UND, // 0xf3
  UND, // 0xf4
  Instruction::new(SBC, AddressingMode::ZeroPageX, by_acc::<Sbc, UseZeroPageWith<'X'>>),
  Instruction::new(INC, AddressingMode::ZeroPageX, by_ref::<Increment, UseZeroPageWith<'X'>>),
  UND, // 0xf7
  Instruction::new(SED, AddressingMode::Implied, set_flag::<'D', true, UseImplied>),
  Instruction::new(SBC, AddressingMode::AbsoluteY, by_acc::<Sbc, UseAbsoluteWith<'Y'>>),
  UND, // 0xfa PLX, implied
  UND, // 0xfb
  UND, // 0xfc
  Instruction::new(SBC, AddressingMode::AbsoluteX, by_acc::<Sbc, UseAbsoluteWith<'X'>>),
  Instruction::new(INC, AddressingMode::AbsoluteX, by_ref::<Increment, UseAbsoluteWith<'X'>>),
  UND, // 0xff
];

#[test]
fn step_by_step() {
  use crate::mos6502::CPU;
  use crate::memory::{Address, MemoryBus, ram::RAM};

  let mut cpu = CPU::new();
  let mut mem = RAM::new();
  let addr = Address::from(0);
 
  mem.write(addr, 0x69); // ADC #0xFF
  mem.write(addr.next(), 0xFF);
  mem.write(addr.next().next(), 0x65); // ADC &0x00 (=0x69)

  assert_eq!(cpu.registers.a, 0);
  assert!(!cpu.registers.p.has::<'C'>());
  assert_eq!(cpu.registers.pc.to_u16(), 0);
  let inst = Instruction::lookup(mem.read(addr));
  inst.execute(&mut cpu.registers, &mut mem);
  assert_eq!(cpu.registers.a, 0xFF);
  assert!(!cpu.registers.p.has::<'C'>());
  assert_eq!(cpu.registers.pc.to_u16(), 2);

  let inst = Instruction::lookup(mem.read(cpu.registers.pc));
  inst.execute(&mut cpu.registers, &mut mem);
  assert_eq!(cpu.registers.a, 0x68);
  assert!(cpu.registers.p.has::<'C'>());
  assert_eq!(cpu.registers.pc.to_u16(), 4);

  cpu.registers.pc = addr; // reset; add 0xFF + carry
  assert_eq!(cpu.registers.a, 0x68);
  assert!(cpu.registers.p.has::<'C'>());
  assert_eq!(cpu.registers.pc.to_u16(), 0);
  let inst = Instruction::lookup(mem.read(cpu.registers.pc));
  inst.execute(&mut cpu.registers, &mut mem);
  assert_eq!(cpu.registers.a, 0x68);
  assert!(cpu.registers.p.has::<'C'>());
  assert_eq!(cpu.registers.pc.to_u16(), 2);

  cpu.registers.pc = addr; // reset; add 0xFF without carry
  cpu.registers.p.set::<'C'>(false);
  assert_eq!(cpu.registers.a, 0x68);
  assert!(!cpu.registers.p.has::<'C'>());
  assert_eq!(cpu.registers.pc.to_u16(), 0);
  let inst = Instruction::lookup(mem.read(cpu.registers.pc));
  inst.execute(&mut cpu.registers, &mut mem);
  assert_eq!(cpu.registers.a, 0x67); // 0x68 - 1
  assert!(cpu.registers.p.has::<'C'>());
  assert_eq!(cpu.registers.pc.to_u16(), 2);
}


