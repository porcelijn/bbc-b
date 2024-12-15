use std::cell::RefCell;
use std::io;
use std::io::Read;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;
use std::thread;

use screen::Screen as Mode4;

use crate::devices::Clocked;
use crate::memory::{Address, MemoryBus};

pub struct KeyboardBuffer {
  rx: Receiver<u8>,
}

impl KeyboardBuffer {
  pub fn new() -> Self {
    let (tx, rx) = mpsc::channel::<u8>();
    thread::spawn(move || loop {
      let stdin = io::stdin();
      if let Some(byte) = stdin.bytes().next() {
        let byte = byte.unwrap();
        tx.send(byte).unwrap();
      } else {
        break;
      }
    });

    KeyboardBuffer{ rx }
  }

  pub fn try_read(&self) -> Option<u8> {
    match self.rx.try_recv() {
      Ok(key) => Some(key),
      Err(TryRecvError::Empty) => None,
      Err(TryRecvError::Disconnected) => panic!("disconnected"),
    }
  }
}

pub struct Screen{
  screen: Mode4,
  memory: Rc<RefCell<dyn MemoryBus>>,
  cycles: u64,
}

impl Screen {
  pub fn new(title: &str, memory: Rc<RefCell<dyn MemoryBus>>) -> Self {
    let screen = Mode4::new(title);
    Screen { screen, memory, cycles: 0 }
  }

  pub fn try_read(&self) -> Option<u8> {
    let keys = self.screen.get_keys();
    if keys.len() == 0 {
      None
    } else {
      Some(keys[0]) // return first key, ignore rest (TODO)
    }
  }

  pub fn blit(&mut self) {
    let (from, to) = (Address::from(0x3000), Address::from(0x8000));
    let memory = self.memory.borrow();
    let vram = memory.try_slice(from, to);
    let vram = vram.expect("Could not get VRAM slice");
    self.screen.blit(vram);
  }
}

impl Clocked for Screen {
  fn step(&mut self, us: u64) {
    const REFRESH: u64 = 20000; // 50Hz
    if self.cycles / REFRESH < us / REFRESH {
      self.blit();
      self.screen.show();
    }
    self.cycles = us;
  }
}

