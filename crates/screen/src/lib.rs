use minifb::{Key, Key::*, Window, WindowOptions};

// MODE 4 Physical size in pixels
const WIDTH: usize = 320;
const HEIGHT: usize = 256;

const COLUMNS: usize = WIDTH / 8; // 40
const _ROWS: usize = WIDTH / 8; // 32

// 3 bit RGB color
#[allow(non_camel_case_types)]
type u3 = u8;
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
      let mut source = (y % 8) + (y / 8) * WIDTH;
      for x in 0..(COLUMNS) {
        assert!(y < HEIGHT);
        assert!(x * 8 < WIDTH);
        assert!((x * 8 + (y % 8) + (y / 8) * WIDTH) < WIDTH * HEIGHT); 
        assert_eq!(source, x * 8 + (y % 8) + (y / 8) * WIDTH); 
        assert_eq!(target, x * 8 + y * WIDTH);
        let byte = video_ram[source];
        for color in PixelIter::new(byte) {
          self.buffer[target] = color;
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

struct PixelIter {
  byte: u8,
  count: u8,
}

impl PixelIter {
  const PALETTE: [ u32; 4] = [
    Screen::BLUE >> 2, // looks better than black
    Screen::BLUE >> 2,
    Screen::WHITE,
    Screen::YELLOW,    // ARTIFACT
  ];

  fn new(byte: u8) -> Self {
    PixelIter { byte, count: 8 }
  }
}

impl Iterator for PixelIter {
  type Item = u32;

  fn next(&mut self) -> Option<Self::Item> {
    if self.count != 0 {
      let a3 = 0b1000_0000 & self.byte != 0;
      let a2 = 0b0010_0000 & self.byte != 0;
      let _a1 = 0b0000_1000 & self.byte != 0;
      let _a0 = 0b0000_0010 & self.byte != 0;
      let color = Self::PALETTE[2*a3 as usize | a2 as usize];
      self.byte <<= 1;
      self.byte |= 1;
      self.count -= 1;
      Some(color)
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

