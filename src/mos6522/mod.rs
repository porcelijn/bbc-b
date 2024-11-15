use crate::memory::{Address, devices::Device, MemoryBus};

pub trait Port {
  fn read(&self, ddr_mask: u8) -> u8;
  fn write(&mut self, value: u8, ddr_mask: u8);
}

pub struct BogusPort<const ID: char, const VALUE: u8>;
impl<const ID: char, const VALUE: u8> Port for BogusPort<ID, VALUE> {
  fn read(&self, ddr_mask: u8) -> u8 {
    let result = VALUE & ddr_mask; // fixme
    println!("reading from port {ID} -> {result}");
    result
  }
  fn write(&mut self, value: u8, ddr_mask: u8) {
    let result = value & ddr_mask ^ VALUE; // fixme
    println!("writing to port {ID} -> {result}");
  }
}

pub type SystemPortA = BogusPort<'A', 42>;

//  &40–&5F 6522 VIA SYSTEM VIA 23
pub type SystemVIA = VIA<SystemPortA>;
impl Device for SystemVIA {
  fn name(&self) -> &'static str { "6522 System VIA" }
}

pub type UserPortA = BogusPort<'a', 0xFF>;

//  &60–&7F 6522 VIA USER VIA 24
pub type UserVIA = VIA<UserPortA>;
impl Device for UserVIA {
  fn name(&self) -> &'static str { "6522 User VIA" }
}

#[derive(Debug)]
pub struct VIA<P: Port> {
  iora: u8,
  iorb: u8,
  ddra: u8,
  ddrb: u8,
  // todo ...
  ier: u8,
  port_a: P,
}

const BIT7: u8 = 1u8 << 7;

impl<P: Port> VIA<P> {
  pub const fn new(port_a: P) -> Self {
    VIA::<P>{ iora: 0, iorb: 0,
            ddra: 0, ddrb: 0,
            ier: 0,
            port_a
    }
  }
}

impl<P: Port> MemoryBus for VIA<P> {
  fn read(&self, address: Address) -> u8 {
    match address.to_u16() & 0x000F {
      0b0000 => println!("read {address:?} IORB"),
      0b0001 => {
        println!("read {address:?} IORA");
        // self.iora = self.port_a.read(self.ddra);
      },
      0b0010 => println!("read {address:?} DDRB"),
      0b0011 => println!("read {address:?} DDRA"),
      0b0100 => println!("read {address:?} T1C-L"),
      0b0101 => println!("read {address:?} T1C-H"),
      0b0110 => println!("read {address:?} T1L-L"),
      0b0111 => println!("read {address:?} T1L-H"),
      0b1000 => println!("read {address:?} T2C-L"),
      0b1001 => println!("read {address:?} T2C-H"),
      0b1010 => println!("read {address:?} SR"),
      0b1011 => println!("read {address:?} ACR"),
      0b1100 => println!("read {address:?} PCR"),
      0b1101 => println!("read {address:?} IFR"),
      0b1110 => { 
        println!("read {address:?} IER");
        return self.ier | BIT7; // When read, bit 7 is *always* a logic 1
      },
      0b1111 => {
        println!("read {address:?} IORAnh");
        self.port_a.read(self.ddra);
      },
      _      => unreachable!(),
    };

    0xFF // bogus
  }
  fn write(&mut self, address: Address, value: u8) {
    match address.to_u16() & 0x000F {
      0b0000 => {
        println!("write {value:#04x} -> {address:?} IORB");
        self.iorb = value;
        return;
      },
      0b0001 => {
        println!("write {value:#04x} -> {address:?} IORA");
        self.iora = value;
        self.port_a.write(value, self.ddra);
        return;
      },
      0b0010 => {
        println!("write {value:#04x} -> {address:?} DDRB");
        self.ddrb = value; //todo
        return;
      },
      0b0011 => {
        println!("write {value:#04x} -> {address:?} DDRA");
        self.ddra = value; // todo
        return;
      },
      0b0100 => println!("write {value:#04x} -> {address:?} T1C-L"),
      0b0101 => println!("write {value:#04x} -> {address:?} T1C-H"),
      0b0110 => println!("write {value:#04x} -> {address:?} T1L-L"),
      0b0111 => println!("write {value:#04x} -> {address:?} T1L-H"),
      0b1000 => println!("write {value:#04x} -> {address:?} T2C-L"),
      0b1001 => println!("write {value:#04x} -> {address:?} T2C-H"),
      0b1010 => println!("write {value:#04x} -> {address:?} SR"),
      0b1011 => println!("write {value:#04x} -> {address:?} ACR"),
      0b1100 => println!("write {value:#04x} -> {address:?} PCR"),
      0b1101 => println!("write {value:#04x} -> {address:?} IFR"),
      0b1110 => println!("write {value:#04x} -> {address:?} IER"),
      0b1111 => {
        println!("write {value:#04x} -> {address:?} IORAnh");
        self.port_a.write(value, self.ddra);
      },
      _      => unreachable!(),
    };
  }
}


