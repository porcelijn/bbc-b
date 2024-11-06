mod mos6502;
use mos6502::CPU;
fn main() {
  let mut cpu = CPU::new();
  cpu.step(3);
  println!("My first BBC-B emulator");
}
