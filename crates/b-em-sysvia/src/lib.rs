// a quasi object shim that delegates to C implementation singleton
pub struct Sysvia;

pub type Keypress = dyn FnMut() -> (u8, bool);

impl Sysvia {
  pub fn new(callback: Box<Keypress>) -> Self {
    unsafe {
      set_singleton(callback);
      sysvia_reset();
    }
    Sysvia
  }

  pub fn read(&self, address: u16) -> u8 {
    let value = unsafe { sysvia_read(address) };
    println!("B-em read: {address:x} -> {value:#04x}");
    value
  }

  pub fn write(&self, address: u16, value: u8) {
    println!("B-em write: {address:x} <- {value:#04x}");
    unsafe { sysvia_write(address, value) };
  }

  pub fn step(&self, ticks: u32) {
    unsafe { sysvia_poll(ticks) };
  }
}

extern {
  static mut interrupt: u32;
  fn sysvia_reset();
  fn sysvia_read(address: u16) -> u8;
  fn sysvia_write(address: u16, value: u8);
  fn sysvia_set_ca2(level: u32);
  fn sysvia_poll(cycles: u32);
}

static mut KEYROW: u32 = 0;
static mut KEYCOL: u32 = 0;
static mut IC32: u32 = 0;
static mut BBCMATRIX: [[bool; 8]; 10] = [[false; 8]; 10];

#[no_mangle]
pub extern fn key_update() {
  let maxcol = 10;
  if unsafe { IC32 & 8 } != 0 {
    /* autoscan mode */
    for col in 0..maxcol {
      for row in 1..8 {
        if unsafe { BBCMATRIX[col as usize][row as usize] } {
          unsafe { sysvia_set_ca2(1) };
          return;
        }
      }
    }
  }
  else {
    /* scan specific key mode */
    if unsafe { KEYCOL } < maxcol {
      for row in 1..8 {
        if unsafe { BBCMATRIX[KEYCOL as usize][row as usize] } {
          unsafe { sysvia_set_ca2(1) };
          return;
        }
      }
    }
  }
  unsafe { sysvia_set_ca2(0); }
}

#[no_mangle]
pub extern fn key_scan(row: u32, col: u32) {
  unsafe {
    KEYROW = row;
    KEYCOL = col;
  }
  key_update();
}

#[no_mangle]
pub extern fn key_is_down() -> bool {
  unsafe {
    if KEYROW == 0 && KEYCOL >= 2 && KEYCOL <= 9 {
      let kbdips = 0b0000_0000; // TODO, stub
      return kbdips & (1 << (9 - KEYCOL)) != 0;
    } else {
      return BBCMATRIX[KEYCOL as usize][KEYROW as usize];
    }
  }
}

/*
#[repr(C)]
struct VIA {
  ora: u8, orb: u8, ira: u8, irb: u8, 
  ddra: u8, ddrb: u8, 
  sr: u8, 
  t1pb7: u8, 
  t1l: u32, t2l:  u32, 
  t1c: u32, t2c:  u32, 
  acr: u8, pcr: u8, ifr: u8, ier: u8, 
  t1hit: u32, t2hit:  u32, 
  ca1: u32, ca2: u32, cb1: u32, cb2: u32, 
  intnum: u32, 
  sr_count: u32, 
  uint8_t  (*read_portA)(void);
  uint8_t  (*read_portB)(void);
  void     (*write_portA)(uint8_t val);
  void     (*write_portB)(uint8_t val);

  void     (*set_ca1)(int level);
  void     (*set_ca2)(int level);
  void     (*set_cb1)(int level);
  void     (*set_cb2)(int level);
  void     (*timer_expire1)(void);
}
*/

#[no_mangle]
pub extern fn crtc_latchpen() {
  unimplemented!("6845 lightpen");
}

#[no_mangle]
pub extern fn sn_write(data: u8) {
  println!("sn76489 data: {data}");
}

#[no_mangle]
pub extern fn led_update(led_name: u32 /* led_name_t */, b: bool, ticks: u32) {
  println!("LED update: led_name={led_name}, b={b}, ticks={ticks}");
}

static mut CALLBACK: Option<Box<Keypress>> = None;
unsafe fn set_singleton(callback: Box<Keypress>) {
  assert!(CALLBACK.is_none());
  CALLBACK = Some(callback);
}

#[no_mangle]
pub extern fn key_paste_poll() {
  // stick as close to the b-em/src/keyboard.c implementation as possible, but
  // wire to keyboard through callback
  unsafe {
    #[allow(static_mut_refs)] // FIXME
    if let Some(callback) = &mut CALLBACK {
      let (key_code, pressed) = callback();
      let row = (key_code & 0b0111_0000) >> 4;
      let col = (key_code & 0b0000_1111) >> 0;
      BBCMATRIX[col as usize][row as usize] = pressed;
    }
  }
}

#[allow(unused)]
unsafe fn characterization_test() {
  sysvia_reset();
  let v = sysvia_read(0);
  assert_eq!(v, 0xFF); 
  sysvia_write(0, 1);

  assert_eq!(interrupt, 0);
}

#[test]
fn test() {
  // do stuff
  unsafe { characterization_test() };

  unsafe { sysvia_reset() };
  let v = unsafe { sysvia_read(0) };
  assert_eq!(v, 0xFF); 
  unsafe { sysvia_write(0, 1) };

  assert_eq!(unsafe { interrupt }, 0);
}

