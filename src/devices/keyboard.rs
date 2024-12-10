use std::cell::{Cell, RefCell};
use std::rc::Rc;

use b_em_sysvia::Keyboard as B_em;

// The keyboard consists of 8 rows, 10 columns of wires
// bottom row (a. o. SHIFT, CTRL) does not cause interrupts
// bottom row 2-9 is wired to a dip switch that controls boot options
//
const MAX_COL: u8 = 10;

#[derive(Debug)]
pub struct Keyboard {
  matrix: [u8; MAX_COL as usize],
  hack_key: Rc<Cell<(u8, bool)>>, // for callback
  pub b_em: Rc<RefCell<B_em>>,
}

impl Keyboard {
  const fn mask(row: u8) -> u8 {
    1 << row
  }

  pub fn new() -> Self {
    let hack_key = Rc::new(Cell::new((0, false)));
    let press = hack_key.clone();
    // dummy, because we'll poke straight into bbcmatrix 
    let dummy_keypress = Box::new(move || { press.get() });
    let b_em = Rc::new(RefCell::new(B_em::new(dummy_keypress)));
    Keyboard { matrix: [0; MAX_COL as usize], hack_key, b_em }
  }

  fn read(&self, row: u8, col: u8) -> bool {
    // row connects to System VIA PA4..PA6 through IC2 data selector (74LS251)
    // col connects to System VIA PA0..PA3 through IC1 synch bin ctr (74LS163)
    assert!(row < 8 && col < MAX_COL);
    let value = self.matrix[col as usize] & Self::mask(row) != 0;
    log::trace!("Keyboard: {row}, {col} = {value}");
    let (r, c) = (self.b_em.borrow().keyrow, self.b_em.borrow().keycol); // backup
    self.b_em.borrow_mut().keyrow = row as u32;
    self.b_em.borrow_mut().keycol = col as u32;
    assert_eq!(value, self.b_em.borrow().scan_key());
    self.b_em.borrow_mut().keyrow = r; // restore
    self.b_em.borrow_mut().keycol = c; // restore
    value
  }

  fn write(&mut self, row: u8, col: u8, value: bool) {
    assert!(row < 8 && col < MAX_COL);
    if value {
      self.matrix[col as usize] |= Self::mask(row);
    } else {
      self.matrix[col as usize] &= !Self::mask(row);
    }
    self.hack_key.set((row << 4 | col, value));
    self.b_em.borrow_mut().update_keys();
  }

  pub fn is_key_pressed(&self, key_code: u8) -> bool {
    if key_code == 15 {
      // Just before scanning the keyboard:
      //   0xF0F0  LDA #15 ; select a non-existent keyboard column 15 (0-9 only!)
      //   0xF0F2  STA .systemVIARegisterANoHandshake
      return false;
    }

    let (row, col) = Self::decode(key_code);
    let pressed = self.read(row, col);
    pressed
  }

  pub fn press_key(&mut self, key_code: u8) {
    let (row, col) = Self::decode(key_code);
    self.write(row, col, true);
  }

  pub fn release_key(&mut self, key_code: u8) {
    let (row, col) = Self::decode(key_code);
    self.write(row, col, false);
  }

  pub fn press_key_ascii(&mut self, ascii: u8) {
    let (key_code, shift) = ascii_to_key_code(ascii as char);
    if shift {
      self.write(0, 0, true); // press SHIFT
    }
    let (row, col) = Self::decode(key_code);
    self.write(row, col, true);
  }

  pub fn release_key_ascii(&mut self, ascii: u8) {
    let (key_code, _) = ascii_to_key_code(ascii as char);
    let (row, col) = Self::decode(key_code);
    self.write(row, col, false);
    self.write(0, 0, false); // release SHIFT
  }

  // if true, send CA1 to system VIA (ic32 KB autoscan disabled)
  pub fn scan_column(&self, col: u8) -> bool {
    let c = self.b_em.borrow().keycol; // backup
    self.b_em.borrow_mut().keycol = col as u32;
    let b_em_value =self.b_em.borrow().scan_col();
    self.b_em.borrow_mut().keycol = c; // restore
 
    if col < MAX_COL {
      // mask out row 0 (SHIFT, CRTL, dip switches)
      let result = self.matrix[col as usize] & 0b1111_1110 != 0;
      assert_eq!(result, b_em_value);
      return result;
    }

    // .loopKeyboardColumns (MOS 0xFE03)
    // selects a non-existent keyboard column 15 (0-9 only!)
    assert_eq!(col, 15);
    assert!( ! b_em_value);
    false
  }

  // if true, send CA1 to system VIA (ic32 KB autoscan enabled)
  pub fn scan_interrupt(&self) -> bool {
    let b_em_value = self.b_em.borrow().scan_all();
    for col in 0 .. MAX_COL {
      if self.scan_column(col) {
        assert!(b_em_value);
        return true;
      }
    }
    assert!( ! b_em_value);
    false
  }

  // Key bit     off/set                      on/clear
  // ------------------------------------------------------------------
  //  0                              SHIFT
  //  1                              CTRL
  // ------------------------------------------------------------------
  //  2   7 128  Default: DFS                 Default: NFS
  //  3   6  64                      Not used
  //  4   5  32  Disc drive timings           Disc drive timings
  //  5   4  16  Disc drive timings           Disc drive timings
  //  6   3   8  SHIFT-BREAK to boot          BREAK to boot
  //  7   2   4  +4 to screen mode            +0 to screen mode
  //  8   1   2  +2 to screen mode            +0 to screen mode
  //  9   0   1  +1 to screen mode            +0 to screen mode
  pub fn set_dip_switch(&mut self, bits: u8) {
    let row = 0u8;
    let mut mask = 0b1000_0000u8;
    for col in 2u8 .. 10u8 {
      let value = bits & mask != 0;
      self.write(row, col, value);
      mask >>= 1;
    }
  }

  pub const fn decode(key_code: u8) -> (u8, u8) {
    let col = (0b0000_1111 & key_code) >> 0;
    let row = (0b0111_0000 & key_code) >> 4;
    (row, col)
  }
}

const fn ascii_to_key_code(ascii: char) -> (u8, bool) {
  let mut i = 0_usize;
  while i < ASCII_TO_KEY_CODE.len() {
    let pair = &ASCII_TO_KEY_CODE[i];
    if ascii == pair.0 { return (pair.2, false); }
    if ascii == pair.1 { return (pair.2, true); }
    i += 1
  }
  unimplemented!();
}

// translate lower case ASCII, upper case, 7 bit key code
const ASCII_TO_KEY_CODE: [(char, char, u8); 51] = [
    ( ' ',  ' ',  0x62 ), // SPACE
    ( '\t', '\t', 0x60 ), // TAB
    ( '\n', '\r', 0x49 ), // Newline / RETURN
    ( '0',  '0',  0x27 ),
    ( '1',  '!',  0x30 ),
    ( '2',  '"',  0x31 ),
    ( '3',  '#',  0x11 ),
    ( '4',  '$',  0x12 ),
    ( '5',  '%',  0x13 ),
    ( '6',  '&',  0x34 ),
    ( '7',  '\'', 0x24 ),
    ( '8',  '(',  0x15 ),
    ( '9',  ')',  0x26 ),
    ( '-',  '=',  0x17 ),
    ( '^',  '~',  0x18 ),
    ( '_',  '₤',  0x28 ),
    ( '[',  '{',  0x38 ),
    ( '@',  '@',  0x47 ),
    ( ':',  '*',  0x48 ),
    ( ';',  '+',  0x57 ),
    ( ']',  '}',  0x58 ),
    ( ',',  '<',  0x66 ),
    ( '.',  '>',  0x67 ),
    ( '/',  '/',  0x68 ),
    ( '\\', '|',  0x78 ),
    ( 'a',  'A',  0x41 ),
    ( 'b',  'B',  0x64 ),
    ( 'c',  'C',  0x52 ),
    ( 'd',  'D',  0x32 ),
    ( 'e',  'E',  0x22 ),
    ( 'f',  'F',  0x43 ),
    ( 'g',  'G',  0x53 ),
    ( 'h',  'H',  0x54 ),
    ( 'i',  'I',  0x25 ),
    ( 'j',  'J',  0x45 ),
    ( 'k',  'K',  0x46 ),
    ( 'l',  'L',  0x56 ),
    ( 'm',  'M',  0x65 ),
    ( 'n',  'N',  0x55 ),
    ( 'o',  'O',  0x36 ),
    ( 'p',  'P',  0x37 ),
    ( 'q',  'Q',  0x10 ),
    ( 'r',  'R',  0x33 ),
    ( 's',  'S',  0x51 ),
    ( 't',  'T',  0x23 ),
    ( 'u',  'U',  0x35 ),
    ( 'v',  'V',  0x63 ),
    ( 'w',  'W',  0x21 ),
    ( 'x',  'X',  0x42 ),
    ( 'y',  'Y',  0x44 ),
    ( 'z',  'Z',  0x61 ),

/* TODO:
    (  'Shift Lock',  0x50 ),
    (  'Shift',       0x00 ),
    (  'Shift',       0x00 ),
    (  'Delete',      0x59 ),
    (  'Copy',        0x69 ),
    (  'Left',        0x19 ),
    (  'Right',       0x79 ),
    (  'Up',          0x39 ),
    (  'Down',        0x29 ),
    (  'Caps Lock',   0x40 ),
    (  'CTRL',        0x01 ),

    (  'Escape',      0x70 ),
    (  'F0',          0x20 ),
    (  'F1',          0x71 ),
    (  'F2',          0x72 ),
    (  'F3',          0x73 ),
    (  'F4',          0x14 ),
    (  'F5',          0x74 ),
    (  'F6',          0x75 ),
    (  'F7',          0x16 ),
    (  'F8',          0x76 ),
    (  'F9',          0x77 ),
    (  'Break',       0xff ),
    */
];


#[test]
fn press_keyboard() {
  let mut kb = Keyboard::new();
  // diagonally press and release keys
  for key in 1..8 {
    assert_eq!(kb.scan_interrupt(), false);
    kb.write(key, key, true);
    assert_eq!(kb.scan_interrupt(), true);
    assert!(kb.is_key_pressed(key << 4 | key));
    kb.write(key, key, false);
    assert_eq!(kb.scan_interrupt(), false);
  }

  assert_eq!(kb.scan_interrupt(), false);
  for key in 0..74 {
    let col = key / 8;
    let row = key % 8;
    let key_code = row << 4 | col;
    println!("{key_code:x} {row} {col}");
    assert_eq!(kb.read(row, col), false);
    kb.press_key(key_code);
    let causes_interrupt = row != 0; // SHIFT, CTRL, and dip switches don't IRQ
    assert_eq!(kb.scan_interrupt(), causes_interrupt);
    assert_eq!(kb.read(row, col), true);
    kb.release_key(key_code);
    assert_eq!(kb.scan_interrupt(), false);
    assert_eq!(kb.read(row, col), false);
  }
}

#[test]
fn test_dip_switch() {
  let mut kb = Keyboard::new();
  assert_eq!(kb.read(0, 2), false);
  kb.set_dip_switch(0b1101_0101);
  assert_eq!(kb.scan_interrupt(), false);
  assert_eq!(kb.read(0, 2), true);
  assert_eq!(kb.read(0, 3), true);
  assert_eq!(kb.read(0, 4), false);
  assert_eq!(kb.read(0, 5), true);
  assert_eq!(kb.read(0, 6), false);
  assert_eq!(kb.read(0, 7), true);
  assert_eq!(kb.read(0, 8), false);
  assert_eq!(kb.read(0, 9), true);
}

