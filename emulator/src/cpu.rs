use std::path::{Path, PathBuf};

use crossbeam_channel::Receiver;

use crate::{keyboard::Keyboard, renderer::Renderer, speaker::Speaker};

const SPRITES: [u8; 5 * 0x10] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Cpu {
    memory: [u8; 0x1000],
    registers: [u8; 0x10],
    pointer: u16,
    program_counter: u16,
    delay_timer: u8,
    sound_timer: u8,
    stack: Vec<u16>,
    paused: bool,
    speed: usize,
    keyboard: Option<(u8, Receiver<u8>)>,
}

impl Cpu {
    pub fn new() -> Self {
        let mut s = Self {
            memory: [0; 0x1000],
            registers: [0; 0x10],
            pointer: 0,
            program_counter: 0x200,
            delay_timer: 0,
            sound_timer: 0,
            stack: Vec::new(),
            paused: false,
            speed: 10,
            keyboard: None,
        };
        s.load_sprites();
        s
    }

    fn load_sprites(&mut self) {
        for (s, mem) in SPRITES.iter().zip(self.memory.iter_mut()) {
            *mem = *s;
        }
    }

    pub fn load_program(&mut self, program: &[u8]) {
        for (s, mem) in program.iter().zip(self.memory.iter_mut().skip(0x200)) {
            *mem = *s;
        }
    }

    pub fn load_rom<P: AsRef<Path>>(&mut self, p: P) -> std::io::Result<()> {
        let program = std::fs::read(p)?;
        self.load_program(&program);
        Ok(())
    }

    pub fn load_rom_with_name<P: AsRef<Path>>(&mut self, name: P) -> std::io::Result<()> {
        self.load_rom(PathBuf::from("roms").join(name))
    }

    pub fn cycle(&mut self, speaker: &Speaker, renderer: &mut Renderer, keyboard: &mut Keyboard) {
        for _ in 0..self.speed {
            if !self.paused {
                let opcode = ((self.memory[self.program_counter as usize] as u16) << 8)
                    | self.memory[self.program_counter as usize + 1] as u16;
                self.execute_instruction(opcode, renderer, keyboard);
            }
        }
        if !self.paused {
            self.update_timers()
        }
        self.play_sound(speaker)
    }

    fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }
    fn play_sound(&self, speaker: &Speaker) {
        if self.sound_timer > 0 {
            speaker.play()
        } else {
            speaker.stop()
        }
    }

    fn execute_instruction(
        &mut self,
        opcode: u16,
        renderer: &mut Renderer,
        keyboard: &mut Keyboard,
    ) {
        if let Some((x, key)) = self
            .keyboard
            .as_ref()
            .and_then(|(x, rx)| rx.try_recv().ok().map(|k| (*x, k)))
        {
            self.registers[x as usize] = key;
            self.keyboard = None;
        } else {
            self.program_counter += 2;
            let instr = ((opcode & 0xF000) >> 12) as u8;
            let addr = opcode & 0x0FFF;
            let x = ((opcode & 0x0F00) >> 8) as u8;
            let y = ((opcode & 0x00F0) >> 4) as u8;
            let n = (opcode & 0x000F) as u8;
            let kk = (opcode & 0x00FF) as u8;
            match (instr, x, y, n) {
                (0, 0, 0xE, 0) => renderer.clear(), // CLS
                (0, 0, 0xE, 0xE) => {
                    // RET
                    self.program_counter = self.stack.pop().expect("Address to return to");
                }
                (0, _, _, _) => (), // SYS addr
                (1, _, _, _) => {
                    // JP addr
                    self.program_counter = addr;
                }
                (2, _, _, _) => {
                    // CALL addr
                    self.stack.push(self.program_counter);
                    self.program_counter = addr;
                }
                (3, _, _, _) => {
                    // SE Vx, byte
                    if self.registers[x as usize] == kk {
                        self.program_counter += 2;
                    }
                }
                (4, _, _, _) => {
                    // SNE Vx, byte
                    if self.registers[x as usize] != kk {
                        self.program_counter += 2;
                    }
                }
                (5, _, _, 0) => {
                    // SE Vx, Vy
                    if self.registers[x as usize] == self.registers[y as usize] {
                        self.program_counter += 2;
                    }
                }
                (6, _, _, _) => {
                    // LD Vx, byte
                    self.registers[x as usize] = kk
                }
                (7, _, _, _) => {
                    // ADD Vx, byte
                    self.registers[x as usize] = self.registers[x as usize].wrapping_add(kk)
                }
                (8, _, _, 0) => {
                    // LD Vx, Vy
                    self.registers[x as usize] = self.registers[y as usize]
                }
                (8, _, _, 1) => {
                    // OR Vx, Vy
                    self.registers[x as usize] |= self.registers[y as usize]
                }
                (8, _, _, 2) => {
                    // AND Vx, Vy
                    self.registers[x as usize] &= self.registers[y as usize]
                }
                (8, _, _, 3) => {
                    // XOR Vx, Vy
                    self.registers[x as usize] ^= self.registers[y as usize]
                }
                (8, _, _, 4) => {
                    // ADD Vx, Vy
                    let (r, overflowed) =
                        self.registers[x as usize].overflowing_add(self.registers[y as usize]);
                    self.registers[0xF] = if overflowed { 1 } else { 0 };
                    self.registers[x as usize] = r;
                }
                (8, _, _, 5) => {
                    // SUB Vx, Vy
                    let (r, overflowed) =
                        self.registers[x as usize].overflowing_sub(self.registers[y as usize]);
                    self.registers[0xF] = if !overflowed { 1 } else { 0 };
                    self.registers[x as usize] = r;
                }
                (8, _, _, 6) => {
                    // SHR Vx{, Vy}
                    self.registers[0xF] = self.registers[x as usize] & 1;
                    self.registers[x as usize] >>= 1;
                }
                (8, _, _, 7) => {
                    // SUBN Vx, Vy
                    let (r, overflowed) =
                        self.registers[y as usize].overflowing_sub(self.registers[x as usize]);
                    self.registers[0xF] = if !overflowed { 1 } else { 0 };
                    self.registers[x as usize] = r;
                }
                (8, _, _, 0xE) => {
                    // SHL Vx{, Vy}
                    let (r, overflowed) = self.registers[x as usize].overflowing_shl(1);
                    self.registers[0xF] = if overflowed { 1 } else { 0 };
                    self.registers[x as usize] = r;
                }
                (9, _, _, 0) => {
                    // SNE Vx, Vy
                    if self.registers[x as usize] != self.registers[y as usize] {
                        self.program_counter += 2;
                    }
                }
                (0xA, _, _, _) => {
                    // LD I, addr
                    self.pointer = addr;
                }
                (0xB, _, _, _) => {
                    // JP V0, addr
                    self.program_counter = addr + self.registers[0] as u16;
                }
                (0xC, _, _, _) => {
                    // RND Vx, byte
                    self.registers[x as usize] = rand::random::<u8>() & kk;
                }
                (0xD, _, _, _) => {
                    // DRW Vx, Vy, nibble
                    let mut coll = false;
                    let (x, y) = (
                        self.registers[x as usize] as usize,
                        self.registers[y as usize] as usize,
                    );
                    log::info!("Drawing sprite at {} with {} bytes at {} {}", self.pointer, n, x, y);
                    for y_diff in 0..n {
                        for (b, x_diff) in (0..8).rev().enumerate() {
                            // log::info!("{} {}  => {}", x + x_diff, y + y_diff as usize, (self.memory[self.pointer as usize] & (1 << b)) >> b);
                            if self.memory[self.pointer as usize + y_diff as usize] & (1 << b) != 0 {
                                coll |= renderer.set_pixel(x + x_diff, y + y_diff as usize);
                            }
                        }
                    }
                    self.registers[0xF] = if coll { 1 } else { 0 };
                }
                (0xE, _, 0x9, 0xE) => {
                    // SKP Vx
                    if keyboard.is_pressed(self.registers[x as usize]) {
                        self.program_counter += 2
                    }
                }
                (0xE, _, 0xA, 0x1) => {
                    // SKNP Vx
                    if !keyboard.is_pressed(self.registers[x as usize]) {
                        self.program_counter += 2
                    }
                }
                (0xF, _, 0x0, 0x7) => {
                    // LD Vx, DT
                    self.registers[x as usize] = self.delay_timer;
                }
                (0xF, _, 0x0, 0xA) => {
                    // LD Vx, K
                    self.keyboard = Some((x, keyboard.set_callback()))
                }
                (0xF, _, 0x1, 0x5) => {
                    // LD DT, Vx
                    self.delay_timer = self.registers[x as usize]
                }
                (0xF, _, 0x1, 0x8) => {
                    // LD ST, Vx
                    self.sound_timer = self.registers[x as usize]
                }
                (0xF, _, 0x1, 0xE) => {
                    // ADD I, Vx
                    self.pointer = self.pointer.wrapping_add(self.registers[x as usize] as u16)
                }
                (0xF, _, 0x2, 0x9) => {
                    // LD F, Vx
                    self.pointer = self.registers[x as usize] as u16 * 5
                }
                (0xF, _, 0x3, 0x3) => {
                    // LD B, Vx
                    let vx = self.registers[x as usize];
                    self.memory[self.pointer as usize] = vx / 100;
                    self.memory[self.pointer as usize + 1] = (vx % 100) / 10;
                    self.memory[self.pointer as usize + 2] = vx % 10;
                }
                (0xF, _, 0x5, 0x5) => {
                    // LD [I], Vx
                    for (v, mem) in self
                        .registers
                        .iter()
                        .take(x as usize)
                        .zip(self.memory.iter_mut().skip(self.pointer as usize))
                    {
                        *mem = *v
                    }
                }
                (0xF, _, 0x6, 0x5) => {
                    // LD Vx, [I]
                    for (v, mem) in self
                        .registers
                        .iter_mut()
                        .take(x as usize)
                        .zip(self.memory.iter().skip(self.pointer as usize))
                    {
                        *v = *mem
                    }
                }
                _ => panic!("Unknown instruction: {:04X}", opcode),
            }
        }
    }

    // pub fn reset(&mut self) {
    //     *self = Self::new()
    // }
}
