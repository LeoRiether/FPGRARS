
macro_rules! define_inner {
    () => {};
    (F7: $value:expr $(, $($tts:tt)*)?) => {
        #[allow(dead_code)]
        pub const F7: u32 = $value;
        #[allow(dead_code)]
        pub const F10: u32 = ($value << 3) | F3;
        define_inner! { $($($tts)*)? }
    };
    ($key:ident: $value:expr $(, $($tts:tt)*)?) => {
        #[allow(dead_code)]
        pub const $key: u32 = $value;
        define_inner! { $($($tts)*)? }
    };
}

macro_rules! define {
    ($name:ident, $($props:tt)*) => {
        pub mod $name {
            define_inner! { $($props)* }
        }
    };
}

pub const OPCODE_TYPE_R: u32 = 0b0110011;
pub const OPCODE_TYPE_I_IMM: u32 = 0b0010011;
pub const OPCODE_TYPE_I_LOAD: u32 = 0b0000011;
pub const OPCODE_TYPE_I_JALR: u32 = 0b1100111;
pub const OPCODE_TYPE_I_BRANCH: u32 = 0b1100011;
pub const OPCODE_TYPE_I_MISC_MEM: u32 = 0b0001111;
pub const OPCODE_TYPE_I_SYSTEM: u32 = 0b1110011;
pub const OPCODE_TYPE_S: u32 = 0b0100011;
pub const OPCODE_TYPE_U: u32 = 0b0110111;
pub const OPCODE_TYPE_B: u32 = 0b1100011;

// Type R
define! { add,    F3: 0b000, F7: 0b0000000 }
define! { sub,    F3: 0b000, F7: 0b0100000 }
define! { sll,    F3: 0b001, F7: 0b0000000 }
define! { slt,    F3: 0b010, F7: 0b0000000 }
define! { sltu,   F3: 0b011, F7: 0b0000000 }
define! { xor,    F3: 0b100, F7: 0b0000000 }
define! { srl,    F3: 0b101, F7: 0b0000000 }
define! { sra,    F3: 0b101, F7: 0b0100000 }
define! { or,     F3: 0b110, F7: 0b0000000 }
define! { and,    F3: 0b111, F7: 0b0000000 }
define! { mul,    F3: 0b000, F7: 0b0000001 }
define! { mulh,   F3: 0b001, F7: 0b0000001 }
define! { mulhsu, F3: 0b010, F7: 0b0000001 }
define! { mulhu,  F3: 0b011, F7: 0b0000001 }
define! { div,    F3: 0b100, F7: 0b0000001 }
define! { divu,   F3: 0b101, F7: 0b0000001 }
define! { rem,    F3: 0b110, F7: 0b0000001 }
define! { remu,   F3: 0b111, F7: 0b0000001 }

// Type I -- Load
define! { lb,     F3: 0b000 }
define! { lh,     F3: 0b001 }
define! { lw,     F3: 0b010 }
define! { lbu,    F3: 0b100 }
define! { lhu,    F3: 0b101 }

// Type I -- Immediate
define! { addi,   F3: 0b000 }
define! { slti,   F3: 0b010 }
define! { sltiu,  F3: 0b011 }
define! { xori,   F3: 0b100 }
define! { ori,    F3: 0b110 }
define! { andi,   F3: 0b111 }
define! { slli,   F3: 0b001, F7: 0b0000000 }
define! { srli,   F3: 0b101, F7: 0b0000000 }
define! { srai,   F3: 0b101, F7: 0b0100000 }

// Type I -- Jalr
define! { jalr,   F3: 0b000 }

// Type I -- Misc Mem
define! { fence,  F3: 0b000, F7: 0b0000000 }

// Type I -- System
define! { ecall,  F3: 0b000, F7: 0b0000000 }
define! { ebreak, F3: 0b000, F7: 0b0000001 }
define! { csrrw,  F3: 0b001, F7: 0b0000000 }
define! { csrrs,  F3: 0b010, F7: 0b0000000 }
define! { csrrc,  F3: 0b011, F7: 0b0000000 }
define! { csrrwi, F3: 0b101, F7: 0b0000000 }
define! { csrrsi, F3: 0b110, F7: 0b0000000 }
define! { csrrci, F3: 0b111, F7: 0b0000000 }

// Type S 
define! { sb, F3: 0b000 }
define! { sh, F3: 0b001 }
define! { sw, F3: 0b010 }

// Type B 
define! { beq,  F3: 0b000 }
define! { bne,  F3: 0b001 }
define! { blt,  F3: 0b100 }
define! { bge,  F3: 0b101 }
define! { bltu, F3: 0b110 }
define! { bgeu, F3: 0b111 }

