// Motorola 6845 video controller
// Just provide 50 Hz VSync signal to System VIA

use crate::devices::Clocked;

#[derive(Debug)]
pub struct CRTC {
  pub vsync: bool,
}

impl Clocked for CRTC {
  fn step(&mut self, ms: u64) {
    self.vsync = ms % 20_000 == 0;
  }
}
