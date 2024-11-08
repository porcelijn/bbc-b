use super::instructions::{Instruction, AddressingMode};
use crate::memory::Address;

// iterate over variable size &[u8] chunks, where each 1, 2 or 3 byte chunk is
// a 6502 instruction 
pub struct Chunks<'a> {
  bytes: &'a [u8],
  index: usize,
}

impl<'a> Chunks<'a> {
  pub fn new(bytes: &'a [u8]) -> Self {
    Chunks { bytes, index: 0 }
  }
}

impl<'a> Iterator for Chunks<'a> {
  type Item = &'a [u8];
  fn next(&mut self) -> Option<Self::Item> {
    if self.index < self.bytes.len() {
      let start = self.index;
      let operation = Instruction::lookup(self.bytes[start]);
      let end = start + 1 + operation.addressing_mode.get_size() as usize;
      self.index = end;
      Some(&self.bytes[start .. end])
    } else {
      None
    }
  }
}

fn hexdump(bytes: &[u8], size: usize) -> String {
  // trucate if byte slice is too large
  let bytes = if size < bytes.len() {
    &bytes[.. size]
  } else {
    bytes
  };

  let mut hexs: Vec<String> =
    bytes.iter().map(|b: &u8| format!("{b:02x}")).collect();

  // add filler is slice is too short
  while hexs.len() < size {
    hexs.push("??".to_string());
  }

  // add padding to align with max instruction size (3 bytes, for 6502)
  while hexs.len() < 3 {
    hexs.push("  ".to_string());
  }
  hexs.join(" ")
}

fn do_disassemble(bytes: &[u8], get_operand: impl Fn(&AddressingMode, &[u8]) -> String) -> String {
  assert!(bytes.len() > 0);
  let operation = Instruction::lookup(bytes[0]);
  let operand_size = operation.addressing_mode.get_size() as usize;
  let hexdump = hexdump(bytes, 1 + operand_size);
  let mnemonic = operation.mnemonic.to_str();
  let result = if bytes.len() < 1 + operand_size {
    let addressing_mode = operation.addressing_mode.get_name();
    format!("{hexdump} {mnemonic} {addressing_mode}")
  } else if operand_size > 0 {
    let operand = &bytes[1 .. 1 + operand_size];
    let addressing_mode = get_operand(&operation.addressing_mode, operand);
    format!("{hexdump} {mnemonic} {addressing_mode}")
  } else {
    format!("{hexdump} {mnemonic}")
  };
  if !operation.is_valid() {
    format!("{result} <-- invalid opcode")
  } else {
    result
  }
}

pub fn disassemble(bytes: &[u8]) -> String {
  let get_operand = |addressing_mode: &AddressingMode, bytes: &[u8]| -> String {
    addressing_mode.get_operand(bytes)
  };
  do_disassemble(bytes, get_operand)
}

pub fn disassemble_with_address(address: Address, bytes: &[u8]) -> String {
  let get_operand = |addressing_mode: &AddressingMode, bytes: &[u8]| -> String {
    match addressing_mode {
      AddressingMode::Relative => {
        assert!(bytes.len() == 1); // operand is pc-relative offset (1 x i8)
        let operand = bytes[0];
        let mut address = address;
        address.inc_by(2);
        if operand & 0b1000_0000 == 0 {
          address.inc_by(operand)
        } else {
          address.dec_by(!operand + 1)
        }
        format!("{address:?}")
      },
      _ => {
        addressing_mode.get_operand(bytes)
      }
    }
  };
  do_disassemble(bytes, get_operand)
}

#[test]
fn display_instructions()
{
  // bunch of interesting looking opcodes
  for opcode in 0xb4..0xc1 {
    println!("ADDR  {:}", disassemble(&[opcode, 0x12, 0x34]));
  }
}
