use std::io::{stdout, Write};

use bbc_b::mos6502::{CPU, stop_at};
use bbc_b::devices::{DevicePage, SheilaPage};
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
  let sheila = SheilaPage::new();
  mem.add_backend(SheilaPage::page(), Box::new(sheila));

  // intercept calls to "OS write character" (ie. BBC Basic II VDU commands)
  // and translate to STDOUT
  let break_oswrch = stop_at::<0xFFEE>;
  let mut cpu = CPU::new();
  cpu.reset(&mut mem);
  loop {
    cpu.run(&mut mem, &break_oswrch);
    vdu_to_terminal(cpu.registers.a);
    cpu.step(&mut mem);
  }
}
