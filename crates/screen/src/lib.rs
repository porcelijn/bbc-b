use minifb::{Key, Key::*, Window, WindowOptions};

// MODE 4 Physical size in pixels
const WIDTH: usize = 320;
const HEIGHT: usize = 256;

//const PIXELS_PER_BYTE: usize = 8; // MODE 4
//const PIXELS_PER_BYTE: usize = 4; // MODE 1, 5
const PIXELS_PER_BYTE: usize = 2;   // MODE 2
const COLUMNS: usize = 80; //WIDTH / PIXELS_PER_BYTE;
const _ROWS: usize = WIDTH / 8; // 32

// 3 bit RGB color
#[allow(non_camel_case_types)]
type u3 = u8;

// 4 bit colour address or F-R-G-B
#[allow(non_camel_case_types)]
type u4 = u8;

type Buffer =[u32; WIDTH * HEIGHT]; // 24 bits RGB

pub struct Screen {
  buffer: Buffer,
  window: Window,
}

impl Screen {
  const fn color_from_u3(rgb3: u3) -> u32 {
    assert!(rgb3 < 8);
    let mut rgb24 = 0u32;
    let mut mask = 0b100;
    while mask != 0 {
      rgb24 <<= 8;
      if rgb3 & mask != 0 {
        rgb24 |= 0xFF;
      }
      mask >>= 1;
    }
    rgb24
  }

  const BLACK:   u32 = Self::color_from_u3(0b000);
  const BLUE:    u32 = Self::color_from_u3(0b001);
  const GREEN:   u32 = Self::color_from_u3(0b010);
  const CYAN:    u32 = Self::color_from_u3(0b011);
  const RED:     u32 = Self::color_from_u3(0b100);
  const MAGENTA: u32 = Self::color_from_u3(0b101);
  const YELLOW:  u32 = Self::color_from_u3(0b110);
  const WHITE:   u32 = Self::color_from_u3(0b111);

  const COLORS: [u32; 8] = [
    Self::BLACK,
    Self::BLUE,
    Self::GREEN,
    Self::CYAN,
    Self::RED,
    Self::MAGENTA,
    Self::YELLOW,
    Self::WHITE,
  ];

  const MONOCHROME: [u32; 2] = [Screen::BLACK, Screen::WHITE];
  const FOURCOLORS: [u32; 4] =
    [Screen::BLACK, Screen::RED, Screen::YELLOW, Screen::WHITE];
  const ALL_COLORS: [u32; 8] = [
    Screen::BLACK, Screen::RED, Screen::GREEN, Screen::YELLOW,
    Screen::BLUE, Screen::MAGENTA, Screen::CYAN, Screen::WHITE,
  ];
  const PALETTE: Palette = make_palette(&Self::ALL_COLORS);

  pub fn new(title: &str) -> Self {
    let mut window_options = WindowOptions::default();
    window_options.scale = minifb::Scale::X2;
    let mut window = Window::new(title, WIDTH, HEIGHT, window_options)
      .unwrap_or_else(|e| { panic!("failed to open Window {}", e); });

    // Limit to max ~50 fps update rate
    window.set_target_fps(50);

    Screen { buffer: [0u32; WIDTH * HEIGHT], window }
  }

  pub fn done(&self) -> bool {
    !self.window.is_open() || self.window.is_key_down(Key::Escape)
  }

  pub fn get_keys(&self) -> Vec<u8> {
    let mut result = Vec::new();
    let mut keys: Vec<Key> = self.window.get_keys();
    let mut shift = false;
    let pred = |key: &Key| {
      shift |= *key == Key::LeftShift || *key == RightShift;
      *key != Key::LeftShift && *key != RightShift
    };
    keys = keys.into_iter().filter(pred).collect::<Vec<Key>>();
    for key in keys.iter() {
      result.push(Self::key_to_ascii(*key, shift));
    }
    result
  }

  pub fn show(&mut self) {
    // We unwrap here as we want this code to exit if it fails. Real
    // applications may want to handle this in a different way
    self.window
      .update_with_buffer(&self.buffer, WIDTH, HEIGHT)
      .unwrap();
  }

  pub fn blit(&mut self, video_ram: &[u8]) {
    for y in 0..HEIGHT {
      let mut target = y * WIDTH;
      let mut source = (y % 8) + (y / 8) * COLUMNS * 8;
      for x in 0..(COLUMNS) {
        assert!(y < HEIGHT);
        assert!(x * PIXELS_PER_BYTE < WIDTH);
        assert!((x * 8 + (y % 8) + (y / 8) * WIDTH) < WIDTH * HEIGHT); 
        assert_eq!(source, x * 8 + (y % 8) + (y / 8) * COLUMNS * 8); 
        assert_eq!(target, x * 2 * PIXELS_PER_BYTE + y * WIDTH);
        let byte = video_ram[source];
        for color in PixelIter::new(byte, PIXELS_PER_BYTE as u8) {
          let color = Self::PALETTE[color as usize];
          self.buffer[target] = color;
          target += 1;
          self.buffer[target] = color; // MODE 5
          target += 1;
        }
        source += 8;
      }
    }
  }

  fn key_to_ascii(key: Key, shift: bool) -> u8 {
    if Key::A <= key && key <= Key::Z {
      if shift {
        'A' as u8 + key as u8 - Key::A as u8
      } else {
        'a' as u8 + key as u8 - Key::A as u8
      }
    } else if Key::Key0 <= key && key <= Key9 {
      if shift {
        match key {
          Key::Key0 => ')' as u8,
          Key::Key1 => '!' as u8,
          Key::Key2 => '@' as u8,
          Key::Key3 => '#' as u8,
          Key::Key4 => '$' as u8,
          Key::Key5 => '%' as u8,
          Key::Key6 => '^' as u8,
          Key::Key7 => '&' as u8,
          Key::Key8 => '*' as u8,
          Key::Key9 => '(' as u8,
          _ => unreachable!(),
        }
      } else {
        '0' as u8 + key as u8 - Key::Key0 as u8
      }
    } else {
      match key {
        Key::Apostrophe => if shift { b'"' } else { b'\'' },
        Key::Backquote => if shift { b'~' } else { b'`' },
        Key::Backslash => '\\' as u8,
        Key::Backspace => 127,
        Key::Comma => if shift { b'<' } else { b',' },
        Key::Delete => 127,
        Key::Enter => b'\r',
        Key::Equal => if shift { b'+' } else { b'=' },
        Key::Escape => 27,
        Key::Minus => if shift { b'_' } else { b'-' },
        Key::Period => if shift { b'>' } else { b'.' },
        Key::Semicolon => if shift { b':' } else { b';' },
        Key::Slash => if shift { b'?' } else { b'/' },
        Key::Space => ' ' as u8,
        Key::Tab =>   '\t' as u8,
        // TODO: a lot
        _ => unimplemented!("Unknown key: {key:?}"),
      }
    }
  }
}

type Palette = [u32; 16];
const fn make_palette(colors: &[u32]) -> Palette {
  let len = colors.len();
  assert!(len == 2 || len == 4 || len == 8);
  let mut palette = [colors[0]; 16];
  let mut index = 1;
  while index != 16 {
    let color = match len {
      2 =>  (0b1000 & index) >> 3,
      4 => ((0b1000 & index) >> 2) | ((0b0010 & index)>>1),
      8 =>  (0b0111 & index) >> 0,
      _ => panic!("Invalid colors, len must be 2, 4, or 8"),
    };
    palette[index] = colors[color as usize];
    index += 1;
  }
  palette
}

struct PixelIter {
  byte: u8,
  count: u8, // shift count down: 8 (monochrome), 4 (MODE 1, 5) or 2 (16 colors)
}

impl PixelIter {
  const fn new(byte: u8, count: u8) -> Self {
    assert!(count == 2 || count == 4 || count == 8);
    PixelIter { byte, count }
  }

  // the shifted byte mask maps:
  //   b7 b6 b5 b4 b3 b2 b1 b0
  //   a3    a2    a1    a0
  // lines a0-3 are address lines to 64 bit (4x4) palette register
  fn u8_to_u4(mask: u8) -> u4 {
    let a3 = 0b1000_0000 & mask != 0;
    let a2 = 0b0010_0000 & mask != 0;
    let a1 = 0b0000_1000 & mask != 0;
    let a0 = 0b0000_0010 & mask != 0;
    let mut address: u8 = 0;
    if a0 { address |= 0b0001 }
    if a1 { address |= 0b0010 }
    if a2 { address |= 0b0100 }
    if a3 { address |= 0b1000 }
    address
  }
}

impl Iterator for PixelIter {
  type Item = u4;

  fn next(&mut self) -> Option<Self::Item> {
    if self.count != 0 {
      let address = Self::u8_to_u4(self.byte);
      self.byte <<= 1;
      self.byte |= 1;
      self.count -= 1;
      Some(address)
    } else {
      None
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn update(buffer: &mut Buffer, time: u32) {
    let mut seed = time as usize;
    for iter in buffer.iter_mut() {
      *iter = Screen::COLORS[(seed / 10) % 6]; // write something more funny here!
      seed += 1;
    }
  }

  fn draw_line(buffer: &mut Buffer) {
    for x in 0..WIDTH {
      let y = (HEIGHT - 1) - HEIGHT * x / WIDTH;
      buffer[x + WIDTH * y] = 0xFFFFFF;
    }
  }


#[test]
  fn it_works() {
    let mut buffer = Screen::new("Test (ESC to exit, time out after 2s)");
    let mut time = 0;
    while !buffer.done() && time < 2 * 50 {
      update(&mut buffer.buffer, time);
      draw_line(&mut buffer.buffer);
      buffer.show();
      time += 1;
    }
  }
}

