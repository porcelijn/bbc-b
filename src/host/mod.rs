use std::io;
use std::io::Read;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;
use std::thread;

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

