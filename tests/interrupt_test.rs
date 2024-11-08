
use bbc_b::mos6502::{CPU, stop_after, stop_when};
use bbc_b::memory::{Address, MemoryBus, ram::RAM};

fn load_program() -> (RAM, Address, Address, Address) {
  // Shortly after CPU enters START, it will spin in a loop waiting for mem[0]
  // to become 0. This never happens unless the INTERRUPT routine is executed.
  // The spin loop either exits after 255 tries with accumulator = 0 (failure)
  // or after the semaphore is freed with accumulator = 1 (success).  Execution
  // must stop when reaching BRK and this must be the fine BRK. So, on exit, pc
  // should point at end of program

  const PROGRAM: [u8; 40] = [
                      //  .ORG $FF00
    0xa2, 0xff,       // START:    ldx #$FF
    0x86, 0x00,       //           stx $00     ; initialize semafore
    0x20, 0x0d, 0xff, //           jsr loop
    0x4c, 0x27, 0xff, //           jmp END     ; we're done
    0x00,             //           brk
    0x00,             //           brk
    0x00,             //           brk
    0xca,             // loop:     dex
    0xf0, 0x07,       //           beq timeout ; stop, failed
    0xa5, 0x00,       //           lda $00     ; read semafore
    0xd0, 0xf9,       //           bne loop    ; spin back
    0xa9, 0x01,       //           lda #1      ; OK
    0x60,             //           rts
    0xa9, 0x00,       // timeout:  lda #0      ; Failed
    0x60,             //           rts
    0x00,             //           brk
    0x00,             //           brk
    0x00,             //           brk
    0xe6, 0x00,       // INTERRUPT:inc $00     ; 0xFF1D <- release semafore
    0x40,             //           rti
    0x00,             //           brk
    0x00,             //           brk
    0x00,             //           brk
    0x78,             // dummy:    sei         ; don't use this jump
    0x20, 0x1d, 0xff, // jsr INTERRUPT
    0x00,             // END:      brk         ; 0xFF27
  ];

  let start = Address::from(0xFF00);
  let end = Address::from(0xFF27);
  let irq_entry = Address::from(0xFF1D);
  let mut ram = RAM::new();
  let _size = ram.load_at(&PROGRAM, start);
  // setup IRQ vector
  const IRQ_VECTOR: Address = Address::from(0xFFFE);
  ram.write(IRQ_VECTOR,        irq_entry.lo_u8());
  ram.write(IRQ_VECTOR.next(), irq_entry.hi_u8());

  (ram, start, end, irq_entry)
}

#[test]
fn test_interrupt() {
  let (mut ram, start, end, _) = load_program();

  // Run without interruption
  // - stopped at END label (pc = 0xFF27)
  // - accumulator == 0 on exit -> failure
  // - X == 0 -> watchdog depleted
  let mut cpu = CPU::new();
  cpu.registers.pc = start;

  const BRK: u8 = 0x0;
  cpu.run(&mut ram, &stop_when::<BRK>);
  let regs = &mut cpu.registers;
  assert_eq!(regs.pc, end);
  assert_eq!(regs.a, 0); // FAIL
  assert_eq!(regs.x, 0);
  assert_eq!(regs.s.to_u8(), 0xFF); // Initial value
  assert!(!regs.p.has::<'I'>());
}

#[test]
fn test_no_interrupt() {
  let (mut ram, start, end, irq_entry) = load_program();

  // Interrupt after 100 "cycles"
  // - stopped at END label (pc = 0xFF27)
  // - accumulator == 1 on exit -> success
  // - X != 0 -> watchdog not depleted
  let mut cpu = CPU::new();
  cpu.registers.pc = start;

  const BRK: u8 = 0x0;
  cpu.run(&mut ram, &stop_after::<100>);

  // stopped arbitrarily aftes 100 "cycles", we're somewhere in spin loop
  assert_eq!(cpu.registers.pc, Address::from(0xFF0E));
  assert_eq!(cpu.registers.a, 0xFF);

//println!("handle interrupt");
  cpu.handle_irq(&mut ram);
  // We're servicing the interrupt, going step-by-step
  assert_eq!(cpu.registers.pc, irq_entry);
  assert!(cpu.registers.p.has::<'I'>());
  assert!(!cpu.registers.p.has::<'B'>());

  cpu.step(&mut ram); // 1
  assert_eq!(cpu.registers.pc, irq_entry.next().next()); // @ RTI, now
  assert!(cpu.registers.p.has::<'I'>());
  assert!(!cpu.registers.p.has::<'B'>());
  cpu.step(&mut ram); // 2

  assert_eq!(cpu.registers.pc, Address::from(0xFF0E)); // back spin loop 
  assert!(!cpu.registers.p.has::<'I'>());

  // run till END
  cpu.run(&mut ram, &stop_when::<BRK>);

  assert_eq!(cpu.registers.pc, end);
  assert_eq!(cpu.registers.a, 1); // SUCCESS
  assert_eq!(cpu.registers.x, 230); // not depleted
  assert_eq!(cpu.registers.s.to_u8(), 0xFF); // Initial value
  assert!(!cpu.registers.p.has::<'I'>());
}

 
