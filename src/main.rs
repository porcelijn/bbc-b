mod mos6502;
use mos6502::CPU;
fn main() {
  println!("My first BBC-B emulator");
  let mut cpu = CPU::new();
  println!("- {:?}", cpu);
  cpu.step(3);
  println!("- {:?}", cpu);
}
