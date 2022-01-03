pub enum Ast {
    Clear,
    Return,
    System(u16),
    Jump(u16),
    JumpOffset(u16),
    Call(u16),
    SkipEqByte(u8, u8),
    SkipNotEqByte(u8, u8),
    SkipEqReg(u8, u8),
    SkipNotEqReg(u8, u8),
    LoadByte(u8, u8),
    LoadReg(u8, u8),
    LoadPointer(u16),
    LoadFromDT(u8),
    LoadKeyboard(u8),
    LoadIntoDT(u8),
    LoadIntoST(u8),
    LoadFont(u8),
    LoadDigits(u8),
    LoadIntoRegs(u8),
    LoadFromRegs(u8),
    AddByte(u8, u8),
    AddReg(u8, u8),
    AddToPointer(u8),
    Random(u8, u8),
    Draw(u8, u8, u8),
    SkipPressed(u8),
    SkipNotPressed(u8),
    Or(u8, u8),
    And(u8, u8),
    Xor(u8, u8),
    ShiftRight(u8),
    ShiftLeft(u8),
    Sub(u8, u8),
    SubNeg(u8, u8),
}

impl Ast {
    pub fn parse(opcode: u16) -> Self {
        let instr = ((opcode & 0xF000) >> 12) as u8;
        let addr = opcode & 0x0FFF;
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let y = ((opcode & 0x00F0) >> 4) as u8;
        let n = (opcode & 0x000F) as u8;
        let kk = (opcode & 0x00FF) as u8;
        use Ast::*;
        match (instr, x, y, n) {
            (0x0, 0, 0xE, 0) => Clear,                 // CLS
            (0x0, 0, 0xE, 0xE) => Return,              //RET
            (0x0, _, _, _) => System(addr),            // SYS addr
            (0x1, _, _, _) => Jump(addr),              // JP addr
            (0x2, _, _, _) => Call(addr),              // CALL addr
            (0x3, _, _, _) => SkipEqByte(x, kk),       // SE Vx byte
            (0x4, _, _, _) => SkipNotEqByte(x, kk),    // SNE Vx byte
            (0x5, _, _, 0) => SkipEqReg(x, y),         // SE Vx, Vy
            (0x6, _, _, _) => LoadByte(x, kk),         // LD Vx, byte
            (0x7, _, _, _) => AddByte(x, kk),          // ADD Vx, byte
            (0x8, _, _, 0) => LoadReg(x, y),           // LD Vx, Vy
            (0x8, _, _, 1) => Or(x, y),                // OR Vx, Vy
            (0x8, _, _, 2) => And(x, y),               // AND Vx, Vy
            (0x8, _, _, 3) => Xor(x, y),               // XOR Vx, Vy
            (0x8, _, _, 4) => AddReg(x, y),            // ADD Vx, Vy
            (0x8, _, _, 5) => Sub(x, y),               // SUB Vx, Vy
            (0x8, _, _, 6) => ShiftRight(x),           // SHR Vx{, Vy}
            (0x8, _, _, 7) => SubNeg(x, y),            // SUBN Vx, Vy
            (0x8, _, _, 0xE) => ShiftLeft(x),          // SHL Vx{, Vy}
            (0x9, _, _, 0) => SkipNotEqReg(x, y),      // SNE Vx, Vy
            (0xA, _, _, _) => LoadPointer(addr),     // LD I, addr
            (0xB, _, _, _) => JumpOffset(addr),      // JP V0, addr
            (0xC, _, _, _) => Random(x, kk),         // RND Vx, byte
            (0xD, _, _, _) => Draw(x, y, n),         // DRW Vx, Vy, nibble
            (0xE, _, 0x9, 0xE) => SkipPressed(x),    // SKP Vx
            (0xE, _, 0xA, 0x1) => SkipNotPressed(x), // SKNP Vx
            (0xF, _, 0x0, 0x7) => LoadFromDT(x),     // LD Vx, DT
            (0xF, _, 0x0, 0xA) => LoadKeyboard(x),   // LD Vx, K
            (0xF, _, 0x1, 0x5) => LoadIntoDT(x),     // LD DT, Vx
            (0xF, _, 0x1, 0x8) => LoadIntoST(x),     // LD ST, Vx
            (0xF, _, 0x1, 0xE) => AddToPointer(x),   // ADD I, Vx
            (0xF, _, 0x2, 0x9) => LoadFont(x),       // LD F, Vx
            (0xF, _, 0x3, 0x3) => LoadDigits(x),     // LD B, Vx
            (0xF, _, 0x5, 0x5) => LoadFromRegs(x),   // LD [I], Vx
            (0xF, _, 0x6, 0x5) => LoadIntoRegs(x),   // LD Vx, [I]
            _ => panic!("Unknown instruction: {:04X}", opcode),
        }
    }
}

impl From<Ast> for u16 {
    fn from(val: Ast) -> Self {
        match val {
            Ast::Clear => 0x00E0,
            Ast::Return => 0x00EE,
            Ast::System(addr) => addr,
            Ast::Jump(addr) => 0x1000 | addr,
            Ast::JumpOffset(addr) => 0xB000 | addr,
            Ast::Call(addr) => 0x2000 | addr,
            Ast::SkipEqByte(x, kk) => 0x3000 | ((x as u16) << 8) | kk as u16,
            Ast::SkipNotEqByte(x, kk) => 0x4000 | ((x as u16) << 8) | kk as u16,
            Ast::SkipEqReg(x, y) => 0x5000 | ((x as u16) << 8) | ((y as u16) << 4),
            Ast::SkipNotEqReg(x, y) => 0x9000 | ((x as u16) << 8) | ((y as u16) << 4),
            Ast::LoadByte(x, kk) => 0x6000 | ((x as u16) << 8) | kk as u16,
            Ast::LoadReg(x, y) => 0x8000 | ((x as u16) << 8) | ((y as u16) << 4),
            Ast::LoadPointer(addr) => 0xA000 | addr,
            Ast::LoadFromDT(_) => todo!(),
            Ast::LoadKeyboard(_) => todo!(),
            Ast::LoadIntoDT(_) => todo!(),
            Ast::LoadIntoST(_) => todo!(),
            Ast::LoadFont(_) => todo!(),
            Ast::LoadDigits(_) => todo!(),
            Ast::LoadIntoRegs(_) => todo!(),
            Ast::LoadFromRegs(_) => todo!(),
            Ast::AddByte(_, _) => todo!(),
            Ast::AddReg(_, _) => todo!(),
            Ast::AddToPointer(_) => todo!(),
            Ast::Random(_, _) => todo!(),
            Ast::Draw(_, _, _) => todo!(),
            Ast::SkipPressed(_) => todo!(),
            Ast::SkipNotPressed(_) => todo!(),
            Ast::Or(_, _) => todo!(),
            Ast::And(_, _) => todo!(),
            Ast::Xor(_, _) => todo!(),
            Ast::ShiftRight(_) => todo!(),
            Ast::ShiftLeft(_) => todo!(),
            Ast::Sub(_, _) => todo!(),
            Ast::SubNeg(_, _) => todo!(),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         let result = 2 + 2;
//         assert_eq!(result, 4);
//     }
// }
