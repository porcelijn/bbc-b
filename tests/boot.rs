
use std::path::Path;
use std::io::prelude::*;
use std::fs::File;

use bbc_b::mos6502::{CPU, stop_after};
use bbc_b::mos6502::disassemble::disassemble_with_address;
use bbc_b::memory::{Address, ram::RAM, slice};

fn dump(filename: &str, bytes: &[u8]) {
  let path = Path::new(filename);
  let mut file = File::create(&path).expect("could not create file");
  file.write_all(bytes).expect("could not write file");
}

#[test]
fn os120_reset() {
  const CLEAR_SHEILA: bool = true;

  let mut ram = RAM::new();

  // manually compiled from https://tobylobster.github.io/mos/os120_acme.a
  ram.load_bin_at("images/os120.bin", Address::from(0xC000));
  if CLEAR_SHEILA {
    ram.load_at(&[0; 256], Address::from(0xFE00)); // clear SHEILA page
  }
  let mut cpu = CPU::new();
  cpu.reset(&mut ram);
  assert_eq!(cpu.registers.pc, Address::from(0xD9CD)); // .resetEntryPoint
  for _ in 0..10 {
    let slice = slice(&mut ram, cpu.registers.pc, 3);
    let dump = disassemble_with_address(cpu.registers.pc, &slice);
    println!("{dump}");
    cpu.step(&mut ram);
  }
  {
    let (a, p, pc) = match CLEAR_SHEILA {
      // Sheila is 0-initialized RAM
      true =>  (0,    0b0010_0110, 0xD9E7),
      // Use whatever garbage is in the MOS 1.20 image at SHEILA addresses
      false => (210,  0b1010_0100, 0xD9DE),
    };

    let r = &cpu.registers;
    assert_eq!(r.a, a);
    assert_eq!(r.x, 255);
    assert_eq!(r.y, 0);
    assert_eq!(r.p.to_u8(), p);
    assert_eq!(r.s.to_u8(), 254);
    assert_eq!(r.pc.to_u16(), pc);
  }

  cpu.run(&mut ram, &stop_after::<10_000_000>);
  // capture (max) screen area (20kB)
  dump("dump.bin", &slice(&ram, Address::from(0x3000), 0x5000));
}
