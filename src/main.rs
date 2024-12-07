use std::io::{stdout, Write};
use std::rc::Rc;
use std::cell::RefCell;

use bbc_b::mos6502::{CPU, stop_at};
use bbc_b::devices::{ClockedDevices, DevicePage, SheilaPage};
use bbc_b::devices::keyboard::Keyboard;
use bbc_b::host::KeyboardBuffer;
use bbc_b::memory::{Address, PageDispatcher, read_address};
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

fn main() {
//println!("My first BBC-B emulator");
  let mut ram = RAM::new();
  ram.load_bin_at("images/os120.bin", Address::from(0xC000));
  ram.load_bin_at("images/Basic2.rom", Address::from(0x8000));

  let irq_vector = Address::from(0xFFFE);
  assert_eq!(read_address(&ram, irq_vector).to_u16(), 0xDC1C); // as per MOS
  let mut mem = PageDispatcher::new(Box::new(ram));
  let keyboard = Rc::new(RefCell::new(Keyboard::new()));
  let mut sheila = SheilaPage::new(keyboard.clone());
  sheila.use_alt_system_via = true;
  let irq_level = sheila.irq.clone();
  let clocked_devices: ClockedDevices = sheila.get_clocked_devices();
  mem.add_backend(SheilaPage::page(), Box::new(sheila));

  // intercept calls to "OS write character" (ie. BBC Basic II VDU commands)
  // and translate to STDOUT
  let break_oswrch = stop_at::<0xFFEE>;
  let mut cpu = CPU::new();
  cpu.irq_level = irq_level;
  cpu.handle_rst(&mut mem);

  let keyboard_buffer = KeyboardBuffer::new();
  let mut last_key: Option<u8> =  None;
  let mut wait_a_while: u32 = 0;
  loop {
    if break_oswrch(&cpu, &mem) {
      vdu_to_terminal(cpu.registers.a);
    }
    cpu.step(&mut mem);
    for cd in clocked_devices.iter() {
      cd.borrow_mut().step(cpu.cycles);
    }

    if wait_a_while == 0 {
      wait_a_while = 100;
      if let Some(key) = last_key {
        keyboard.borrow_mut().release_key_ascii(key);
      }

      last_key = keyboard_buffer.try_read();

      if let Some(key) = last_key {
        keyboard.borrow_mut().press_key_ascii(key);
        wait_a_while = 1000;
      }
    } else {
      wait_a_while -= 1;
    }
  }
}
