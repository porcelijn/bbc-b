use crate::mos6502::registers::Status;

pub const fn add_with_carry(register: u8, value: u8, carry: bool)
          -> (u8, bool, bool) {
  let mut result = register as u16;
  result += value as u16;
  if carry {
    result += 1;
  }

  let carry: bool    =  result & 0b_1_0000_0000 != 0;
  let result: u8     = (result & 0b_0_1111_1111) as u8;
  let overflow: bool = (result ^ register)
                     & (result ^ value)
                     & 0b1000_0000 != 0;

  (result, carry, overflow)
}

pub const fn add_decimal_with_carry(register: u8, value: u8, carry: bool)
          -> (u8, bool, bool) {
  const fn lo(value: u8) -> u8 { (value & 0b0000_1111) >> 0 }
  const fn hi(value: u8) -> u8 { (value & 0b1111_0000) >> 4 }
  const fn add_bcd_nibble(lhs: u8, rhs: u8, carry: u8) -> u8 {
    let result = lhs + rhs + carry;
    if result >= 0xA {
      0x10 + lo(result + 6)
    } else {
      result
    }
  }

  let carry          = carry as u8;
  let lo_nibble      = add_bcd_nibble(lo(register), lo(value), carry);
  let hi_nibble      = add_bcd_nibble(hi(register), hi(value), hi(lo_nibble));
  let carry: bool    = hi(hi_nibble) != 0;
  let result: u8     = lo(hi_nibble) << 4 | lo(lo_nibble);
  let overflow: bool = (result ^ register)
                     & (result ^ value)
                     & 0b1000_0000 != 0;

  (result, carry, overflow)
}

#[test]
fn adc_decimal() {
  assert_eq!(add_decimal_with_carry(0x09, 0x09, true),  (0x19, false, false));
  assert_eq!(add_decimal_with_carry(0x98, 0x01, true),  (0x00, true, false));
  assert_eq!(add_decimal_with_carry(0x79, 0x00, true),  (0x80, false, true));
  assert_eq!(add_decimal_with_carry(0x24, 0x56, false), (0x80, false, true));
  assert_eq!(add_decimal_with_carry(0x82, 0x93, false), (0x75, true, true));
  // out of bound / invalid input: C, but no V (both 80, f0 and d0 are negative)
  assert_eq!(add_decimal_with_carry(0x80, 0xf0, false), (0xd0, true, false));
  // overflow first nibble carries 1, not 2, into high byte
  assert_eq!(add_decimal_with_carry(0x0f, 0x0f, true),  (0x15, false, false));
}

pub const fn and(accumulator: u8, value: u8) -> u8 {
  accumulator & value
}

pub const fn eor(accumulator: u8, value: u8) -> u8 {
  accumulator ^ value
}

pub const fn ora(accumulator: u8, value: u8) -> u8 {
  accumulator | value
}

pub fn bit(accumulator: u8, value: u8, mut status: Status) -> Status {
  // A is ANDed with the value in memory to set or clear the zero flag
  status.set::<'Z'>(accumulator & value == 0);
   // Overflow is set to bit 6 of the memory value
  status.set::<'V'>(0b0100_0000 & value != 0);
  // Negative is set to bit 7 of the memory value
  status.set::<'N'>(0b1000_0000 & value != 0);
  status
}

pub const fn inc(register: u8) -> u8 {
  register.wrapping_add(1)
}

pub const fn dec(register: u8) -> u8 {
  register.wrapping_sub(1)
}

pub const fn asl(value: u8) -> (u8, bool) {
  let out_carry: bool = value & 0b1000_0000 != 0; // new carry is bit 7
  let result = (value & 0b0111_1111) << 1;
  (result, out_carry)
}

pub const fn rol(value: u8, in_carry: bool) -> (u8, bool) {
  let (mut result, out_carry) = asl(value);
  if in_carry {
    result |= 0b0000_0001; // shift old carry into bit 0
  }
  (result, out_carry)
}

pub const fn lsr(value: u8) -> (u8, bool) {
  let out_carry: bool = value & 0b0000_0001 != 0; // new carry is bit 0
  let result = (value & 0b1111_1110) >> 1;
  (result, out_carry)
}

pub const fn ror(value: u8, in_carry: bool) -> (u8, bool) {
  let (mut result, out_carry) = lsr(value);
  if in_carry {
    result |= 0b1000_0000; // shift old carry into bit 7
  }
  (result, out_carry)
}

pub const fn sub_with_carry(register: u8, value: u8, carry: bool)
          -> (u8, bool, bool) {
  let mut result = 0b_1_0000_0000_u16 | register as u16;
  result -= value as u16;
  if !carry {
    result -= 1;
  }

  let carry: bool =  result & 0b_1_0000_0000 != 0;
  let result: u8  = (result & 0b_0_1111_1111) as u8;
  let overflow: bool = (result ^ register)
                     & (result ^ value)
                     & 0b1000_0000 != 0;

  (result, carry, overflow)
}

#[test]
fn test_sbc() {
  assert_eq!(sub_with_carry(0, 0, true), (0, true, false));
  assert_eq!(sub_with_carry(1, 0, true), (1, true, false));
  assert_eq!(sub_with_carry(0, 1, true), (255, false, true));
  assert_eq!(sub_with_carry(1, 1, true), (0, true, false));
}

pub const fn sub_decimal_with_carry(register: u8, value: u8, carry: bool)
          -> (u8, bool, bool) {
  const fn lo(value: u8) -> u8 { (value & 0b0000_1111) >> 0 }
  const fn hi(value: u8) -> u8 { (value & 0b1111_0000) >> 4 }
  const fn sub_bcd_nibble(lhs: u8, rhs: u8, carry: u8) -> u8 {
    let result = lhs.wrapping_sub(rhs + 1 - carry);
    if hi(result) != 0 {
      lo(result.wrapping_sub(6)) // borrow
    } else {
      0x10 | result // carry = 1  means: no borrow
    }
  }

  let lo_nibble      = sub_bcd_nibble(lo(register), lo(value), carry as u8);
  let hi_nibble      = sub_bcd_nibble(hi(register), hi(value), hi(lo_nibble));
  let carry: bool    = hi(hi_nibble) != 0;
  let result: u8     = lo(hi_nibble) << 4 | lo(lo_nibble);
  // http://www.6502.org/tutorials/vflag.html
  // For SBC, the V flag in decimal flag behaves as though we're in binary mode
  let binary_result  = register.wrapping_sub(value + 1 - carry as u8);
  let overflow: bool = (binary_result ^ register)
                     & (binary_result ^ value)
                     & 0b1000_0000 != 0;

  (result, carry, overflow)
}

#[test]
fn sbc_decimal() {
  assert_eq!(sub_decimal_with_carry(0, 0, true), (0x00, true, false));
  assert_eq!(sub_decimal_with_carry(1, 0, true), (0x01, true, false));
  assert_eq!(sub_decimal_with_carry(0, 1, true), (0x99, false, true));
  assert_eq!(sub_decimal_with_carry(1, 1, true), (0x00, true, false));
  assert_eq!(sub_decimal_with_carry(0x09, 0x09, true),  (0x0, true, false));
  assert_eq!(sub_decimal_with_carry(0x50, 0x50, true),  (0x0, true, false));
  assert_eq!(sub_decimal_with_carry(0x79, 0x79, true),  (0x0, true, false));
  assert_eq!(sub_decimal_with_carry(0x80, 0x80, true),  (0x0, true, true)); // V=0 virtual6502
//assert_eq!(sub_decimal_with_carry(0x90, 0x90, true),  (0x0, false, false));
//assert_eq!(sub_decimal_with_carry(0x98, 0x23, true),  (0x75, true, true));
//assert_eq!(sub_decimal_with_carry(0x80, 0x00, false), (0x79, true, true));
//assert_eq!(sub_decimal_with_carry(0x80, 0x01, false), (0x78, true, true));
//assert_eq!(sub_decimal_with_carry(0x80, 0x56, false), (0x23, true, true));
//assert_eq!(sub_decimal_with_carry(0x56, 0x80, false), (0x75, false, true));
  // out of bound / invalid input: C, but no V (both 80, f0 and d0 are negative)
  assert_eq!(sub_decimal_with_carry(0x80, 0xf0, false), (0x29, false, false));
  // overflow first nibble carries 1, not 2, into high byte
//assert_eq!(sub_decimal_with_carry(0x0f, 0x15, true),  (0x85, false, false));
//assert_eq!(sub_decimal_with_carry(0x90, 0x0b, false), (0x7e, true, false));
}

#[test]
fn shift_left() {
  assert_eq!(asl(0b1000_0000),                 (0, true));
  assert_eq!(asl(0b0000_0000),                 (0, false));
  assert_eq!(asl(0b0000_0001),        (0b000_0010, false));
  assert_eq!(rol(0b1000_0000, false),          (0, true));
  assert_eq!(rol(0b1000_0000, true),           (1, true));
  assert_eq!(rol(0b0000_0001, false),          (2, false));
  assert_eq!(rol(0b0000_0001, true), (0b0000_0011, false));
  assert_eq!(rol(0b0100_0000, false),(0b1000_0000, false));
}

#[test]
fn shift_right() {
  assert_eq!(lsr(0b0000_0001),                 (0, true));
  assert_eq!(lsr(0b0000_0000),                 (0, false));
  assert_eq!(lsr(0b1000_0000),       (0b0100_0000, false));
  assert_eq!(lsr(0b1000_0001),       (0b0100_0000, true));
  assert_eq!(ror(0b0000_0001, false),          (0, true));
  assert_eq!(ror(0b0000_0001, true), (0b1000_0000, true));
  assert_eq!(ror(0b0000_0010, false),(0b0000_0001, false));
  assert_eq!(ror(0b1000_0000, false),(0b0100_0000, false));
  assert_eq!(ror(0b1000_0000, true), (0b1100_0000, false));
}

