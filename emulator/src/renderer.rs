use crate::{HEIGHT, WIDTH};

pub struct Renderer {
    pixels: [[u8; HEIGHT as usize]; WIDTH as usize],
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
            pixels: [[0; HEIGHT as usize]; WIDTH as usize],
        }
    }
}

impl Renderer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.pixels = [[0; HEIGHT as usize]; WIDTH as usize];
    }

    pub fn set_pixel(&mut self, x: usize, y: usize) -> bool {
        let x = x % self.pixels.len();
        let y = y % self.pixels[0].len();
        self.pixels[x][y] ^= 1;
        self.pixels[x][y] == 0
    }

    pub fn draw(&self, frame: &mut [u8]) {
        // let (mut lx, mut ly) = (0,0);
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = i % self.pixels.len();
            let y = i / self.pixels.len();

            // let inside_the_box = x >= self.box_x
            //     && x < self.box_x + BOX_SIZE
            //     && y >= self.box_y
            //     && y < self.box_y + BOX_SIZE;

            let rgba = if self.pixels[x][y] == 0 {
                [0, 0, 0, 0xff]
            } else {
                [0xff, 0xff, 0xff, 0xff]
            }; // [0x48, 0xb2, 0xe8, 0xff];
            pixel.copy_from_slice(&rgba);
            // lx = x;
            // ly = y;
        }
        // log::info!("{} {} : W {} H {}", lx, ly, WIDTH, HEIGHT);
    }
}
