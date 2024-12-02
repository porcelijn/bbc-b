use std::cell::RefCell;
use std::io::prelude::*;
use std::rc::Rc;

use bbc_b::devices::{ClockedDevices, DevicePage, SheilaPage};
use bbc_b::devices::keyboard::Keyboard;
use bbc_b::mos6502::{CPU, stop_when};
use bbc_b::mos6502::disassemble::disassemble_with_address;
use bbc_b::memory::{Address, MemoryBus, PageDispatcher, ram::RAM, slice};

#[test]
fn interrogate_keyboard() {
  // from MOS 1.20 &F02A, source: https://tobylobster.github.io/mos/os120_acme.a
  //
  // Read a single key's state from the keyboard   
  // On Entry:                                     
  //       X = key to test                         
  // On Exit:                                      
  //       A is preserved                          
  //       Carry is preserved                      
  //                                               
  //       X = $80 if key pressed (N set)          
  //           $00 otherwise      (N clear)    
  const MOS_CLIP: [u8; 17] = [
    0xa0, 0x03,       // LDY #3                             ; stop keyboard scanning
    0x8c, 0x40, 0xfe, // STY .systemVIARegisterB            ; by writing to system VIA
    0xa0, 0x7f,       // LDY #%01111111                     ; input on bit 7, output on bits 0 to 6
    0x8c, 0x43, 0xfe, // STY .systemVIADataDirectionRegisterA
    0x8e, 0x4f, 0xfe, // STX .systemVIARegisterANoHandshake ; write X to Port A system VIA
    0xae, 0x4f, 0xfe, // LDX .systemVIARegisterANoHandshake ; read back ($80 + internal key number) if key pressed (or zero otherwise)
    0x60,             // RTS
  ];

  let start = Address::from(0xF02A);
  const RTS: u8 = 0x60;
  let stop = stop_when::<RTS>;

  let mut ram = RAM::new();
  ram.load_at(&MOS_CLIP, start);
  let keyboard = Rc::new(RefCell::new(Keyboard::new()));
  let mut mem = PageDispatcher::new(Box::new(ram));
  let sheila = SheilaPage::new(keyboard.clone());

  // pre-condition: configure system VIA DDRB to IIIIOOOO, where O's map to ic32
  let system_via_ddrb = Address::from(0xFE42);
  mem.write(system_via_ddrb, 0b0000_1111); // lower nybble are output bits

  let irq_level = sheila.irq.clone();
  let cds = sheila.get_clocked_devices();
  fn step(cds: &ClockedDevices, cycles: u64) {
    for cd in cds.iter() {
      cd.borrow_mut().step(cycles);
    }
  }
  mem.add_backend(SheilaPage::page(), Box::new(sheila));

  let mut cpu = CPU::new();
  cpu.irq_level = irq_level;

  let interrogate_keyboard = |cpu: &mut CPU, mem: &mut dyn MemoryBus| {
    cpu.registers.pc = start;
    println!("Enterring .interrogate_keyboard with X={}", cpu.registers.x);
    while !stop(cpu, mem) {
      let slice = slice(mem, cpu.registers.pc, 3);
      let dump = disassemble_with_address(cpu.registers.pc, &slice);
      cpu.step(mem);
      step(&cds, cpu.cycles);
      println!("{dump} | a:{} x:{} y:{} p:{:?}", cpu.registers.a,
               cpu.registers.x, cpu.registers.y, cpu.registers.p,
      );
    }
  };

  const PRESSED: u8 = 0x80;

  // Press '0' with CA2 masked, because default interrupt enable register = 0
  if false {
    let key_code = 0x27; // '0'
    cpu.registers.x = key_code;
    interrogate_keyboard(&mut cpu, &mut mem);
    assert!(!cpu.registers.p.has::<'N'>()); // Not pressed
    assert_eq!(cpu.registers.x, 0x00);      // Not pressed

    keyboard.borrow_mut().press_key_ascii('0' as u8);

    cpu.registers.x = key_code;
    interrogate_keyboard(&mut cpu, &mut mem);
    assert!(cpu.registers.p.has::<'N'>());  // Pressed
    assert_eq!(cpu.registers.x, PRESSED);   // Pressed

    keyboard.borrow_mut().release_key_ascii('0' as u8);

    cpu.registers.x = key_code;
    interrogate_keyboard(&mut cpu, &mut mem);
    assert!(!cpu.registers.p.has::<'N'>()); // Released
    assert_eq!(cpu.registers.x, 0x00);      // Released
  }

  // Repeat with CA2 interrupt and keyboard autoscan enabled, now press 'A'
  {
    fn enable_keyboard_autoscan(mem: &mut dyn MemoryBus) {
      let system_via_orb  = Address::from(0xFE40);
      let system_via_ifr  = Address::from(0xFE4D);
      let system_via_ier  = Address::from(0xFE4E);

      mem.write(system_via_ifr, 0b0111_1111); // first clear all interrupt flags
      mem.write(system_via_orb, 0b0000_1011); // ic32[3] = 1 means kb autoscan
      mem.write(system_via_ier, 0b1000_0001); // enable CA2
    }

    let key_code = 0x41; // 'A'
    cpu.registers.x = key_code;
    enable_keyboard_autoscan(&mut mem);
    interrogate_keyboard(&mut cpu, &mut mem);
    assert!(!cpu.registers.p.has::<'N'>()); // Not pressed
    assert_eq!(cpu.registers.x, 0x00);      // Not pressed

    keyboard.borrow_mut().press_key_ascii('A' as u8);

    // attempt to read keyboard unsafely, without setting IRQ mask
    cpu.registers.x = key_code;
    enable_keyboard_autoscan(&mut mem);
    cpu.registers.pc = start;
    cpu.step(&mut mem); step(&cds, cpu.cycles);
    cpu.step(&mut mem); step(&cds, cpu.cycles);
    assert!(cpu.registers.p.has::<'I'>());    // WHOOPS, 6522 interrupted CPU
    assert_eq!(cpu.registers.pc.to_u16(), 0); // No BRK/IRQ vector was set

    // try again, but with interrupt masked in CPU
    cpu.registers.p.set_flag::<'I', true>();
    cpu.registers.x = key_code;
    enable_keyboard_autoscan(&mut mem);
    interrogate_keyboard(&mut cpu, &mut mem);
    assert!(cpu.registers.p.has::<'N'>());  // Pressed
    assert_eq!(cpu.registers.x, PRESSED);   // Pressed

    cpu.registers.p.set_flag::<'I', true>();
    cpu.registers.x = 0; // SHIFT keycode
    enable_keyboard_autoscan(&mut mem);
    interrogate_keyboard(&mut cpu, &mut mem);
    assert!(cpu.registers.p.has::<'N'>());
    assert_eq!(cpu.registers.x, PRESSED);

    keyboard.borrow_mut().release_key_ascii('A' as u8);

    // all keys released; this should work without IRQ masked
    cpu.registers.p.set_flag::<'I', false>();
    cpu.registers.x = key_code;
    enable_keyboard_autoscan(&mut mem);
    interrogate_keyboard(&mut cpu, &mut mem);
    assert!(!cpu.registers.p.has::<'N'>()); // Released
    assert_eq!(cpu.registers.x, 0x00);      // Released

    // Also check shift release
    cpu.registers.x = 0;
    interrogate_keyboard(&mut cpu, &mut mem);
    assert_eq!(cpu.registers.x, 0x00);
  }
}



