use std::ops::Range;
use bitfield::bitfield;

const fn mask(range: Range<u32>) -> u32 {
    u32::MAX >> (32 - range.end) << range.start
}

bitfield! {
    #[derive(Default, Clone, Copy)]
    pub struct Instruction(u32);
    impl Debug;

    pub opcode, set_opcode: 6, 0;
    pub rd, set_rd: 11, 7;
    pub funct3, set_funct3: 14, 12;
    pub rs1, set_rs1: 19, 15;
    pub rs2, set_rs2: 24, 20;
    pub funct7, set_funct7: 31, 25;
    // pub imm_i, set_imm_i: 31, 20;
    // pub imm_s, set_imm_s: 31, 25 | 11, 7;
    // pub imm_b, set_imm_b: 31, 31 | 7, 7 | 30, 25 | 11, 8;
    // pub imm_u, set_imm_u: 31, 12;
}

impl Instruction {
    #[inline]
    pub fn funct10(self) -> u32 {
        (self.funct7() << 3) | self.funct3()
    }

    // TODO: check these for correctness
    pub fn imm_i(self) -> i32 {
        let imm = self.0 >> 20;
        if self.0 & (1 << 31) == 0 {
            imm as i32
        } else {
            (imm | mask(12..32)) as i32
        }
    }

    pub fn set_imm_i(&mut self, imm: i32) {
        self.0 = (self.0 & !(0xfff << 20)) | ((imm as u32) << 20);
    }

    pub fn imm_s(self) -> i32 {
        let imm = ((self.0 & mask(25..32)) >> 20) | ((self.0 >> 7) & mask(0..5));
        if self.0 & (1 << 31) == 0 {
            imm as i32
        } else {
            (imm | mask(12..32)) as i32
        }
    }

    pub fn set_imm_s(&mut self, imm: i32) {
        let imm = imm as u32;

        // clear
        self.0 = self.0 & !(0x1f << 7) & !(0x7f << 25);

        // set self[11:7] = imm[4:0]
        self.0 |= (imm & 0x1f) << 7;

        // set self[31:25] = imm[11:5]
        self.0 |= (imm & (0x7f << 5)) << 20;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_imm_i() {
        assert_eq!(Instruction(0x07b14093).imm_i(), 123); 
        assert_eq!(Instruction(0xffc4c413).imm_i(), -4); 
        assert_eq!(Instruction(0x000f8f13).imm_i(), 0); 

        let tests = [0, -4, 123, -123, 0x7f, 0b11111111111];
        for imm in tests {
            let mut i = Instruction(0x12345678);
            i.set_imm_i(imm);
            assert_eq!(i.imm_i(), imm); 
        }
    }

    #[test]
    fn test_imm_s() {
        assert_eq!(Instruction(0x0684ada3).imm_s(), 123); 
        assert_eq!(Instruction(0xfe84ae23).imm_s(), -4); 
        assert_eq!(Instruction(0x0000a023).imm_s(), 0); 

        let tests = [0, -4, 123, -123, 0x7f, 0b11111111111];
        for imm in tests {
            let mut i = Instruction(0x12345678);
            i.set_imm_s(imm);
            assert_eq!(i.imm_s(), imm); 
        }
    }
}
