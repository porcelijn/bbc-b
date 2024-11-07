
use crate::mos6502::registers::Registers;
use crate::memory::Address;
use crate::memory::MemoryBus;
use crate::memory::read_address;

pub trait UseMode {
  fn get_size() -> u8 where Self: Sized;
  fn get_operand(_: &[u8]) -> String where Self: Sized;
  fn get_name() -> &'static str;
}

pub trait UseValue {
  fn get_value(registers: &Registers, memory: &dyn MemoryBus) -> u8;
}

pub trait UseReference {
  fn write(registers: &mut Registers, memory: &mut dyn MemoryBus, value: u8);
}

pub trait UseAddress : UseValue + UseReference {
  fn get_address(registers: &Registers, memory: &dyn MemoryBus) -> Address;
}

impl<T: UseAddress> UseValue for T {
  fn get_value(registers: &Registers, memory: &dyn MemoryBus) -> u8 {
    let address = Self::get_address(registers, memory);
    memory.read(address)
  }
}

impl<T: UseAddress> UseReference for T {
  fn write(registers: &mut Registers, memory: &mut dyn MemoryBus, value: u8) {
    let address = Self::get_address(registers, memory);
    memory.write(address, value);
  }
}

pub struct UseImplied;
impl UseMode for UseImplied {
  fn get_size() -> u8 { 0 }
  fn get_operand(_: &[u8]) -> String where Self: Sized { "".to_string() }
  fn get_name() -> &'static str { "implied" }
}

pub struct UseAccumulator;
impl UseValue for UseAccumulator {
  fn get_value(registers: &Registers, _: &dyn MemoryBus) -> u8 {
    registers.a
  }
}
impl UseReference for UseAccumulator {
  fn write<'a>(registers: &mut Registers, _: &mut dyn MemoryBus, value: u8) {
    registers.a = value;
  }
}
impl UseMode for UseAccumulator {
  fn get_size() -> u8 { 0 }
  fn get_operand(_: &[u8]) -> String where Self: Sized { "A".to_string() }
  fn get_name() -> &'static str { "accumulator" }
}

pub struct UseImmediate;
impl UseValue for UseImmediate {
  fn get_value(registers: &Registers, memory: &dyn MemoryBus) -> u8 {
    let address = registers.pc;
    memory.read(address)
  }
}
impl UseMode for UseImmediate {
  fn get_size() -> u8 { 1 }
  fn get_operand(bytes: &[u8]) -> String {
    let value = bytes[0];
    format!("#{:#04x}", value)
  }
  fn get_name() -> &'static str { "immediate" }
}

pub struct UseZeroPage;
impl UseAddress for UseZeroPage {
  fn get_address(registers: &Registers, memory: &dyn MemoryBus) -> Address {
    let operand = memory.read(registers.pc);
    Address::from_le_bytes(operand, 0)
  }
}
impl UseMode for UseZeroPage {
  fn get_size() -> u8 { 1 }
  fn get_operand(bytes: &[u8]) -> String {
    let value = bytes[0];
    format!("&{:#04x}", value)
  }
  fn get_name() -> &'static str { "zero page" }
}

const fn get_index_register<const XY: char>(registers: &Registers) -> &u8 {
  match XY {
      'x' | 'X' => &registers.x,
      'y' | 'Y' => &registers.y,
      _         => unreachable!(),
  }
}

pub struct UseZeroPageWith<const XY: char>;
impl<const XY: char> UseAddress for UseZeroPageWith<XY> {
  fn get_address(registers: &Registers, memory: &dyn MemoryBus) -> Address {
    let register = get_index_register::<XY>(&registers);
    let operand = memory.read(registers.pc);
    let operand = operand.wrapping_add(*register);
    Address::from_le_bytes(operand, 0)
  }
}
impl<const XY: char> UseMode for UseZeroPageWith<XY> {
  fn get_size() -> u8 { 1 }
  fn get_operand(bytes: &[u8]) -> String {
    let value = bytes[0];
    format!("&{:#04x} + {}", value, XY)
  }
  fn get_name() -> &'static str {
    match XY {
      'x'|'X' => "zero page X",
      'y'|'Y' => "zero page Y",
      _         => unreachable!(),
    }
  }
}

pub struct UseRelative;
impl UseAddress for UseRelative {
  fn get_address(registers: &Registers, memory: &dyn MemoryBus) -> Address {
    let operand = memory.read(registers.pc);
    let mut address = registers.pc;
    if operand & 0b1000_0000 == 0 {
      address.inc_by(operand);
    } else {
      address.dec_by(!operand + 1)
    };
    address
  }
}
impl UseMode for UseRelative {
  fn get_size() -> u8 { 1 }
  fn get_operand(bytes: &[u8]) -> String {
    let value = bytes[0];
    if value & 0b1000_0000 == 0 {
      format!("pc + {}", value)
    } else {
      let value = !value + 1;
      format!("pc - {}", value)
    }
  }
  fn get_name() -> &'static str { "relative" }
}

pub struct UseAbsolute;
impl UseAddress for UseAbsolute {
  fn get_address(registers: &Registers, memory: &dyn MemoryBus) -> Address {
    let operand = read_address(memory, registers.pc);
    operand
  }
}
impl UseMode for UseAbsolute {
  fn get_size() -> u8 { 2 }
  fn get_operand(bytes: &[u8]) -> String {
    let lo = bytes[0];
    let hi = bytes[1];
    let value = ((hi as u16) << 8) | (lo as u16);
    format!("&{:#06x}", value)
  }
  fn get_name() -> &'static str { "absolute" }
}

pub struct UseAbsoluteWith<const XY: char>;
impl<const XY: char> UseAddress for UseAbsoluteWith<XY> {
  fn get_address(registers: &Registers, memory: &dyn MemoryBus) -> Address {
    let register = get_index_register::<XY>(&registers);
    let mut operand = read_address(memory, registers.pc);
    operand.inc_by(*register);
    operand
  }
}
impl<const XY: char> UseMode for UseAbsoluteWith<XY> {
  fn get_size() -> u8 { 2 }
  fn get_operand(bytes: &[u8]) -> String {
    let lo = bytes[0];
    let hi = bytes[1];
    let value = ((hi as u16) << 8) | (lo as u16);
    format!("&{:#06x} + {}", value, XY)
  }
  fn get_name() -> &'static str {
    match XY {
      'x'|'X' => "absolute X",
      'y'|'Y' => "absolute Y",
      _       => unreachable!(),
    }
  }
}

pub struct UseIndirect;
impl UseAddress for UseIndirect {
  fn get_address(registers: &Registers, memory: &dyn MemoryBus) -> Address {
//  let operand = memory.read(&registers.pc);
//  let address = Address::from_le_bytes(operand, 0x00);
    // always use 16 bit operand
    let address = read_address(memory, registers.pc);
    read_address(memory, address)
  }
}
impl UseMode for UseIndirect {
  fn get_size() -> u8 { 2 }
  fn get_operand(bytes: &[u8]) -> String {
    let lo = bytes[0];
    let hi = bytes[1];
    let value = ((hi as u16) << 8) | (lo as u16);
    format!("&({:#06x})", value)
  }
  fn get_name() -> &'static str { "indirect" }
}

pub struct UseIndexedIndirectX;
impl UseAddress for UseIndexedIndirectX {
  fn get_address(registers: &Registers, memory: &dyn MemoryBus) -> Address {
    let operand = memory.read(registers.pc);
    let zero_page_address = operand.wrapping_add(registers.x);
    let address = Address::from_le_bytes(zero_page_address, 0x00);
    read_address(memory, address)  // Should wrap at zero page boundary?
  }
}
impl UseMode for UseIndexedIndirectX {
  fn get_size() -> u8 { 1 }
  fn get_operand(bytes: &[u8]) -> String {
    let value = bytes[0];
    format!("(&{:#04x} + X)", value)
  }
  fn get_name() -> &'static str { "indexed indirect X" }
}

pub struct UseIndirectIndexedY;
impl UseAddress for UseIndirectIndexedY {
  fn get_address(registers: &Registers, memory: &dyn MemoryBus) -> Address {
    let operand = memory.read(registers.pc);
    let address = Address::from_le_bytes(operand, 0x00);
    let mut address = read_address(memory, address); // Should wrap at zero page boundary?
    address.inc_by(registers.y);
    address
  }
}
impl UseMode for UseIndirectIndexedY {
  fn get_size() -> u8 { 1 }
  fn get_operand(bytes: &[u8]) -> String {
    let value = bytes[0];
    format!("(&{:#04x}) + Y", value)
  }
  fn get_name() -> &'static str { "indirect indexed Y" }
}

