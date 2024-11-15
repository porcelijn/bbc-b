
use crate::memory::Address;
use crate::memory::MemoryBus;

pub struct RAM([u8; 64 * 1024]);

impl RAM {
  pub const fn new() -> RAM {
    RAM([0; 64 * 1024])
  }

  #[allow(unused)]
  pub fn load_at(&mut self, program: &[u8], mut address: Address) -> usize {
    for byte in program {
      self.write(address, *byte);
      address = address.next();
    }

    program.len()
  }

  #[allow(unused)]
  pub fn load_bin_at(&mut self, program: &str, address: Address) -> usize {
    use std::io::Read;
    let mut file = std::fs::File::open(program).expect("failed to open file");
    let mut addr = address;

    loop {
      let mut buf = [0u8; 1];
      if file.read(&mut buf).expect("failed to read from file") == 0 {
        break;
      }
      if addr.to_u16() == 0 {
        panic!("reached end of memory!");
      }
      self.write(addr, buf[0]);
      addr = addr.next();
    }

    let addr = match addr.to_u16() {
      0 => 0x10000,
      _ => addr.to_u16() as usize
    };

    addr - address.to_u16() as usize
  }

  #[allow(unused)]
  pub fn load_hex(&mut self, program: &str) ->  (Address, usize) {
    let mut start: Option<Address> = None;
    let mut current = Address::from(0);
    for line in std::fs::read_to_string(program).unwrap().lines() {
      let mut value: Option<u32> = None;
      for c in line.chars() {
        if let Some(v) = c.to_digit(16) {
          match value {
            Some(ref mut value) => {
              *value <<= 4;
              *value += v;
            },
            None => {
              value = Some(v);
            },
          }
        } else if c.is_whitespace() && value.is_some() {
          let value = {
            let v = value.expect("Some(???)");
            value = None;
            v
          };
          assert!(value < 256);
          let byte = value as u8;
          self.write(current, byte);
          current = current.next();
        } else if c == ':' && value.is_some() {
          let value = {
            let v = value.expect("Some(???)");
            value = None;
            v
          };
          assert!(value < 1<<16);
          let value = Address::from(value as u16);
          if start.is_none() {
            current = value;
            start = Some(value);
          } else {
            assert_eq!(current, value);
          }
        }
      }
    }

    let start = start.expect("missing start address");
    let size = (current.to_u16() - start.to_u16()) as usize;

    (start, size)
  }
}

impl std::cmp::PartialEq for Address {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}
impl std::cmp::Eq for Address {}
impl std::cmp::PartialOrd for Address {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.0.cmp(&other.0))
  }
}
impl std::cmp::Ord for Address {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.0.cmp(&other.0)
  }
}

impl MemoryBus for RAM {
  fn read(&self, address: Address) -> u8 {
    self.0[address.to_u16()as usize]
  }

  fn write(&mut self, address: Address, value: u8) {
    self.0[address.to_u16() as usize] = value;
  }
}

