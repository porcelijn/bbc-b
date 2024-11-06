
pub struct Address(u16);

impl Address {
  pub const fn from(address: u16) -> Self {
    Address(address)
  }

  #[allow(unused)]
  pub const fn to_u16(&self) -> u16 {
    self.0
  }

  pub fn inc_by(&mut self, plus: u8) {
    self.0 = self.0.wrapping_add(plus.into());
  }

  #[allow(unused)]
  pub fn dec_by(&mut self, plus: u8) {
    self.0 = self.0.wrapping_sub(plus.into());
  }
}

impl std::fmt::Debug for Address {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "&{:#06x}", self.0)
  }
}

