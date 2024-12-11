use minifb::{Key, Window, WindowOptions};

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
    for key in self.window.get_keys().iter() {
      result.push(*key as u8); // FIXME: need proper conversion to ASCII(?)
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
        let mut byte = video_ram[source];
        for _ in 0..8 {
          let color = if 0b1000_0000 & byte == 0 {
            Self::BLUE >> 2 // looks better than black
          } else {
            Self::WHITE
          };

          self.buffer[target] = color;

          byte <<= 1;
          target += 1;
        }
        source += 8;
      }
    }
  }

  pub fn pixel(&mut self, x: usize, y: usize, c: u3) {
    assert!(c < 8);
    self.buffer[x + WIDTH * y] = Self::COLORS[c as usize];
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

