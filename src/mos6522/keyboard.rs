// FIXME: The keyboard probably should not live under 6522 VIA

const MAX_COL: u8 = 10;

#[derive(Debug)]
pub struct Keyboard {
  matrix: [u8; MAX_COL as usize],
}

impl Keyboard {
  const fn mask(row: u8) -> u8 {
    1 << row
  }

  pub fn new() -> Self {
    Keyboard { matrix: [0; MAX_COL as usize] }
  }

  pub fn read(&self, row: u8, col: u8) -> bool {
    // row connects to System VIA PA4..PA6 through IC2 data selector (74LS251)
    // col connects to System VIA PA0..PA3 through IC1 synch bin ctr (74LS163)
    assert!(row < 8 && col < MAX_COL);
    let value = self.matrix[col as usize] & Self::mask(row) != 0;
    log::trace!("Keyboard: {row}, {col} = {value}");
    value
  }

  pub fn write(&mut self, row: u8, col: u8, value: bool) {
    assert!(row < 8 && col < MAX_COL);
    if value {
      self.matrix[col as usize] |= Self::mask(row);
    } else {
      self.matrix[col as usize] &= !Self::mask(row);
    }
  }

  pub fn is_key_pressed(&self, key_code: u8) -> bool {
    if key_code == 15 {
      // Just before scanning the keyboard:
      //   0xF0F0  LDA #15 ; select a non-existent keyboard column 15 (0-9 only!)
      //   0xF0F2  STA .systemVIARegisterANoHandshake
      return false;
    }

    let col = (0b0000_1111 & key_code) >> 0;
    let row = (0b0111_0000 & key_code) >> 4;
    let pressed = self.read(row, col);
    pressed
  }

  // if true, send CA1 to system VIA
  pub fn scan_interrupt(&self) -> bool {
    for col in 0 .. MAX_COL {
      // mask out row 0 (SHIFT, CRTL, dip switches)
      if self.matrix[col as usize] & 0b1111_1110 != 0 {
        return true;
      }
    }
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
}

#[test]
fn press_keyboard() {
  let mut kb = Keyboard::new();
  assert_eq!(kb.scan_interrupt(), false);
  for key in 0..74 {
    let col = key / 8;
    let row = key % 8;
    println!("{key} {row} {col}");
    assert_eq!(kb.read(row, col), false);
    kb.write(row, col, true);
    assert_eq!(kb.read(row, col), true);
  }
  assert_eq!(kb.scan_interrupt(), true);
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

