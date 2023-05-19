use super::bitops::{mask, BitOps};
use bitfield::bitfield;

bitfield! {
    #[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
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
        let imm = (self.0.mask(25..32) >> 20) | ((self.0 >> 7).mask(0..5));
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

    pub fn imm_b(self) -> i32 {
        let imm = (self.0.mask(31..32) >> 19)
            | (self.0.mask(25..31) >> 20)
            | (self.0.mask(8..12) >> 7)
            | (self.0.mask(7..8) << 4);
        if self.0 & (1 << 31) == 0 {
            imm as i32
        } else {
            (imm | mask(12..32)) as i32
        }
    }

    pub fn set_imm_b(&mut self, imm: i32) {
        let imm = imm as u32;

        // clear
        self.0 = self.0 & !(0x1 << 7) & !(0xf << 8) & !(0x3f << 25) & !(0x1 << 31);

        // set self[7] = imm[11]
        self.0 |= (imm & (0x1 << 11)) >> 4;

        // set self[8:11] = imm[1:4]
        self.0 |= (imm & (0xf << 1)) << 7;

        // set self[25:30] = imm[5:10]
        self.0 |= (imm & (0x3f << 5)) << 20;

        // set self[31] = imm[12]
        self.0 |= (imm & (0x1 << 12)) << 19;
    }

    pub fn imm_j(self) -> i32 {
        let imm = (self.0.mask(31..32) >> 11)
            | (self.0.mask(21..31) >> 20)
            | (self.0.mask(20..21) >> 9)
            | self.0.mask(12..20);
        if self.0 & (1 << 31) == 0 {
            imm as i32
        } else {
            (imm | mask(12..32)) as i32
        }
    }

    pub fn set_imm_j(&mut self, imm: i32) {
        let imm = imm as u32;

        // clear
        self.0 &= !mask(12..32);

        // set self[31] = imm[20]
        self.0 |= (imm & mask(20..21)) << 11;

        // set self[21:30] = imm[1:10]
        self.0 |= (imm & mask(1..11)) << 20;

        // set self[20] = imm[11]
        self.0 |= (imm & mask(11..12)) << 9;

        // set self[12:19] = imm[12:19]
        self.0 |= imm & mask(12..20);
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

    #[test]
    fn test_imm_b() {
        assert_eq!(Instruction(0x1e105663).imm_b(), 492);
        assert_eq!(Instruction(0xfe208ee3).imm_b(), -4);
        assert_eq!(Instruction(0xfe0008e3).imm_b(), -16);
        assert_eq!(Instruction(0x015a7063).imm_b(), 0);

        let tests = [0, -4, 122, -122, 0x7e, 0b11111111110];
        for imm in tests {
            let mut i = Instruction(0x12345678);
            i.set_imm_b(imm);
            assert_eq!(i.imm_b(), imm);
        }
    }

    #[test]
    fn test_imm_j() {
        assert_eq!(Instruction(0x052000ef).imm_j(), 82);
        assert_eq!(Instruction(0xffdff2ef).imm_j(), -4);
        assert_eq!(Instruction(0x0000046f).imm_j(), 0);

        for b in 1..20 {
            let imm = 1 << b;
            let mut i = Instruction(0x12345678);
            i.set_imm_j(imm);
            assert_eq!(i.imm_j(), imm);
        }

        let imm = 1 << 20;
        let mut i = Instruction(0x12345678);
        i.set_imm_j(imm);
        assert_eq!(i.imm_j(), imm | mask(12..32) as i32);
    }
}
