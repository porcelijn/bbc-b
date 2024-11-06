pub struct CPU {
  cycles: u64,
}

impl CPU {
  pub fn new() -> Self {
    CPU { cycles: 0 }
  }

  pub fn step(&mut self, ticks: u64) {
    while self.cycles < ticks {
      self.cycles += 1;
    }
  }
}
