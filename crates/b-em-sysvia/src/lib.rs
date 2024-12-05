// a quasi object shim that delegates to C implementation singleton
pub struct Sysvia {
  via: *mut Cvia,
  state: *mut State,
}

pub type Keypress = dyn FnMut() -> (u8, bool);

impl Sysvia {
  pub fn new(callback: Box<Keypress>) -> Self {
    let state = unsafe { new_state() };
    let via = unsafe { sysvia_new(state) };
    unsafe {
      set_singleton(callback); // TODO: remove, this is bullshit
    }
    Sysvia{ via, state }
  }

  pub fn read(&self, address: u16) -> u8 {
    let value = unsafe { sysvia_read(self.via, address) };
    println!("B-em read: {address:x} -> {value:#04x}");
    value
  }

  pub fn write(&self, address: u16, value: u8) {
    println!("B-em write: {address:x} <- {value:#04x}");
    unsafe { sysvia_write(self.via, address, value) };
  }

  pub fn step(&self, ticks: u32) {
    unsafe { sysvia_poll(self.via, ticks) };
  }
}

impl Drop for Sysvia {
  fn drop(&mut self) {
    unsafe { sysvia_delete(self.via) };
    unsafe { free_state(self.state) };
  }
}

#[repr(C)]
pub struct State {
  /*Current state of IC32 output*/
  ic32: u8,
  /*Current effective state of the slow data bus*/
  sdbval: u8,
  /*What the System VIA is actually outputting to the slow data bus
    For use when contending with whatever else is outputting to the bus*/
  sysvia_sdb_out: u8,

  scrsize: u32,

  via: *mut Cvia,
//int kbdips;
  interrupt: u32,
}

#[repr(C)]
struct Cvia([u8; 0]); // opaque

extern {
  fn new_state() -> *mut State;
  fn free_state(state: *mut State);
//fn get_interrupt(state: *const State) -> u32;
  fn sysvia_new(state: *mut State) -> *mut Cvia;
  fn sysvia_delete(via: *mut Cvia);
  fn sysvia_read(via: *mut Cvia, address: u16) -> u8;
  fn sysvia_write(via: *mut Cvia, address: u16, value: u8);
  fn sysvia_set_ca2(via: *mut Cvia, level: u32);
  fn sysvia_poll(via: *mut Cvia, cycles: u32);
}

//static mut SYSVIA: *mut Cvia = std::ptr::null_mut();
static mut KEYROW: u32 = 0;
static mut KEYCOL: u32 = 0;
//static mut IC32: u32 = 0;
static mut BBCMATRIX: [[bool; 8]; 10] = [[false; 8]; 10];

#[no_mangle]
pub extern fn key_update(state: *mut State) {
  let maxcol = 10;
  let cvia = unsafe { (*state).via };
  if unsafe { (*state).ic32 & 8 } != 0 {
    /* autoscan mode */
    for col in 0..maxcol {
      for row in 1..8 {
        if unsafe { BBCMATRIX[col as usize][row as usize] } {
          unsafe { sysvia_set_ca2(cvia, 1) };
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
          unsafe { sysvia_set_ca2(cvia, 1) };
          return;
        }
      }
    }
  }
  unsafe { sysvia_set_ca2(cvia, 0); }
}

#[no_mangle]
pub extern fn key_scan(state: *mut State, row: u32, col: u32) {
  unsafe {
    KEYROW = row;
    KEYCOL = col;
  }
  key_update(state);
}

#[no_mangle]
pub extern fn key_is_down(_state: *mut State) -> bool {
  let keyrow = unsafe { KEYROW };
  let keycol = unsafe { KEYCOL };
  assert!(keyrow < 8);
  assert!(keycol < 10);
  if keyrow == 0 && keycol >= 2 && keycol <= 9 {
    let kbdips = 0b0000_0000; // TODO, stub
    return kbdips & (1 << (9 - keycol)) != 0;
  } else {
    unsafe {
      return BBCMATRIX[keycol as usize][keyrow as usize];
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
pub extern fn key_paste_poll(_state: *mut State) {
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
  let s = new_state();
  let via = sysvia_new(s);
  let v = sysvia_read(via, 0);
  assert_eq!(v, 0xFF); 
  sysvia_write(via, 0, 1);
  sysvia_poll(via, 1);

  assert_eq!((*s).interrupt, 0);
  sysvia_delete(via);
  free_state(s);
}

#[test]
fn test() {
  // do stuff
  unsafe { characterization_test() };

  let sysvia = Sysvia::new(Box::new(|| (123, true)));
  let v = sysvia.read(0);
  assert_eq!(v, 0xFF); // via_read_null()
  sysvia.write(0, 1);

//assert_eq!(unsafe { interrupt }, 0);
}

