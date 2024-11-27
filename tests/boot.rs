
use std::path::Path;
use std::io::prelude::*;
use std::fs::File;

use bbc_b::devices::{DevicePage, SheilaPage};
use bbc_b::mos6502::{CPU, stop_after};
use bbc_b::mos6502::disassemble::disassemble_with_address;
use bbc_b::memory::{Address, PageDispatcher, ram::RAM, slice};

fn dump(filename: &str, bytes: &[u8]) {
  let path = Path::new(filename);
  let mut file = File::create(&path).expect("could not create file");
  file.write_all(bytes).expect("could not write file");
}

#[test]
fn os120_reset() {
  let mut ram = RAM::new();

  // manually compiled from https://tobylobster.github.io/mos/os120_acme.a
  ram.load_bin_at("images/os120.bin", Address::from(0xC000));
  // Use whatever garbage is in the MOS 1.20 image at SHEILA addresses

  let mut cpu = CPU::new();
  cpu.handle_rst(&mut ram);
  assert_eq!(cpu.registers.pc, Address::from(0xD9CD)); // .resetEntryPoint
  for _ in 0..10 {
    let slice = slice(&mut ram, cpu.registers.pc, 3);
    let dump = disassemble_with_address(cpu.registers.pc, &slice);
    println!("{dump}");
    cpu.step(&mut ram);
  }
  {
    let r = &cpu.registers;
    assert_eq!(r.a, 210);
    assert_eq!(r.x, 255);
    assert_eq!(r.y, 0);
    assert_eq!(r.p.to_u8(), 0b1010_0100);
    assert_eq!(r.s.to_u8(), 254);
    assert_eq!(r.pc.to_u16(), 0xD9DE); // skip reset memory
  }

  cpu.run(&mut ram, &stop_after::<1_000_000>);
}


#[test]
fn os120_reset_with_sheila() {
  use std::rc::Rc;
  use bbc_b::devices::keyboard::Keyboard;
//simple_logger::init_with_level(log::Level::Trace).unwrap();
  let mut ram = RAM::new();
  ram.load_bin_at("images/os120.bin", Address::from(0xC000));
  let mut mem = PageDispatcher::new(Box::new(ram));
  let sheila = SheilaPage::new(Rc::new(Keyboard::new()));
  mem.add_backend(SheilaPage::page(), Box::new(sheila));

  let mut cpu = CPU::new();
  cpu.handle_rst(&mut mem);
  assert_eq!(cpu.registers.pc, Address::from(0xD9CD)); // .resetEntryPoint
  for _ in 0..10 {
    let slice = slice(&mut mem, cpu.registers.pc, 3);
    let dump = disassemble_with_address(cpu.registers.pc, &slice);
    println!("{dump}");
    cpu.step(&mut mem);
  }
  {
    let r = &cpu.registers;
    assert_eq!(r.a, 0);
    assert_eq!(r.x, 255);
    assert_eq!(r.y, 0);
    assert_eq!(r.p.to_u8(), 0b0010_0111);
    assert_eq!(r.s.to_u8(), 254);
    assert_eq!(r.pc.to_u16(), 0xD9E7);
  }

  cpu.run(&mut mem, &stop_after::<10_000_000>);
  // capture (max) screen area (20kB)
  dump("dump.bin", &slice(&mem, Address::from(0x3000), 0x5000));
}
