use std::io::{stdout, Write};
use std::rc::Rc;
use std::cell::RefCell;

use bbc_b::mos6502::{CPU, stop_at};
use bbc_b::devices::{ClockedDevices, DevicePage, SheilaPage};
use bbc_b::devices::keyboard::Keyboard;
use bbc_b::host::Screen;
use bbc_b::memory::{Address, MemoryBus, PageDispatcher, read_address};
use bbc_b::memory::ram::RAM;

fn vdu_to_terminal(a_register: u8) {
  let mut out = stdout();
  match a_register {
    0x20..0x80  => {
      out.write(&[a_register]).unwrap();
      out.flush().unwrap();
    },
    0x0A => {
      println!();
    },
    // for now, ignore multitude of control codes
    _ => {},
  }
}

#[derive(PartialEq)]
struct BufIter(u8); // X register

impl BufIter {
  // Keyboard buffer 3E0-3FF (32 bytes)
  const BASE_ADDRESS:  Address = Address::from(0x0300);
  const START_OFFSET:       u8 = 0xe0;
  const CAPACITY:           u8 = 32;
  const EMPTY_FLAG:    Address = Address::from(0x02cf);
  const START_POINTER: Address = Address::from(0x02d8);
  const END_POINTER:   Address = Address::from(0x02e1);

  fn start(memory: &dyn MemoryBus) -> Self {
    Self(memory.read(Self::START_POINTER))
  }

  fn end(memory: &dyn MemoryBus) -> Self {
    Self(memory.read(Self::END_POINTER))
  }

  fn empty(memory: &dyn MemoryBus) -> bool {
    memory.read(Self::EMPTY_FLAG) & 0b1000_0000 != 0
  }

  fn size(memory: &dyn MemoryBus) -> u8 {
    Self::distance(Self::start(memory), Self::end(memory))
  }

  const fn distance(start: Self, end: Self) -> u8 {
    if start.0 <= end.0 {
      end.0 - start.0
    } else {
      Self::CAPACITY - (start.0 - end.0)
    }
  }

  fn address(&self) -> Address {
    let mut address = Self::BASE_ADDRESS;
    address.inc_by(self.0);
    address
  }

  fn next(&mut self) {
    if self.0 == 0xFF {
      self.0 = Self::START_OFFSET;
    } else {
      self.0 += 1;
    }
  }

  fn save_end(&self, memory: &mut dyn MemoryBus) {
    memory.write(Self::END_POINTER, self.0);
  }
}

fn dump_keyboard_buffer(memory: &dyn MemoryBus) {
  let mut first = BufIter::start(memory);
  let last = BufIter::end(memory);
  let empty = BufIter::empty(memory);
  let size = BufIter::size(memory);
  print!("keyboard buffer, empty={empty}, size={size}:");
  while first != last {
    let value = memory.read(first.address());
    print!(" '{}' ({value}),", value as char);
    first.next();
  }
  println!();
}

fn insert_keyboard_buffer(memory: &mut dyn MemoryBus, value: u8) {
  let first = BufIter::start(memory);
  let mut last = BufIter::end(memory);
  memory.write(last.address(), value);
  last.next();
  assert!(last != first);
  last.save_end(memory);
  memory.write(BufIter::EMPTY_FLAG, 0);
}

fn main() {
//println!("My first BBC-B emulator");
  let mut ram = RAM::new();
  ram.load_bin_at("images/os120.bin", Address::from(0xC000));
  ram.load_bin_at("images/Basic2.rom", Address::from(0x8000));

  let irq_vector = Address::from(0xFFFE);
  assert_eq!(read_address(&ram, irq_vector).to_u16(), 0xDC1C); // as per MOS
  let mut mem = PageDispatcher::new(Box::new(ram));
  let mut keyboard = Keyboard::new();
  // start in MODE 4. lower 3 bits reflect mode, inverted
  keyboard.set_dip_switch(0b0000_0011);
  let keyboard = Rc::new(RefCell::new(keyboard));
  let mut sheila = SheilaPage::new(keyboard.clone());
  sheila.use_alt_system_via = false;//true;
  let irq_level = sheila.irq.clone();
  let mut clocked_devices: ClockedDevices = sheila.get_clocked_devices();
  mem.add_backend(SheilaPage::page(), Box::new(sheila));

  // intercept calls to "OS write character" (ie. BBC Basic II VDU commands)
  // and translate to STDOUT
//let break_oswrch = stop_at::<0xFFEE>;
  let break_oswrch = stop_at::<0xE0A4>; // Basic bypasses vectored OSWRCH entry 
  let mut cpu = CPU::new();
  cpu.irq_level = irq_level;
  cpu.handle_rst(&mut mem);

  let mem = Rc::new(RefCell::new(mem));
  let screen = Screen::new("BBC-B", mem.clone());
  let screen = Rc::new(RefCell::new(screen));
  clocked_devices.push(screen.clone());
  let mut last_key: Option<u8> =  None;
  let mut wait_a_while: u32 = 0;
  loop {
    if break_oswrch(&cpu, &*mem.borrow()) {
      vdu_to_terminal(cpu.registers.a);
    }
    cpu.step(&mut *mem.borrow_mut());
    for cd in clocked_devices.iter() {
      cd.borrow_mut().step(cpu.cycles);
    }

    if wait_a_while == 0 {
      wait_a_while = 100;
      let new_key = screen.borrow().try_read();
      if new_key != last_key {
        if let Some(key) = last_key {
          println!("Release '{}' ({key})", key as char);
          keyboard.borrow_mut().release_key_ascii(key);
        }

        if let Some(key) = new_key {
          println!("Press '{}' ({key})", key as char);
          keyboard.borrow_mut().press_key_ascii(key);
          insert_keyboard_buffer(&mut *mem.borrow_mut(), key);
          wait_a_while = 1000;
        }

        dump_keyboard_buffer(&*mem.borrow());
      }
      last_key = new_key;
    } else {
      wait_a_while -= 1;
    }
  }
}
