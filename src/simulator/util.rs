/// Returns a bitmask of a floating point classification, according to the
/// [RISC-V spec](https://riscv.org//wp-content/uploads/2019/06/riscv-spec.pdf).
/// The definition can be found at the "Single Precision Floating-Point Classify Instruction",
/// but I'll copy the table here anyway:
///
/// | _rd_ bit | Meaning |
/// |---------:|---------|
/// 0| _rs1_ is −∞.
/// 1| _rs1_ is a negative normal number.
/// 2| _rs1_ is a negative subnormal number.
/// 3| _rs1_ is −0.
/// 4| _rs1_ is +0.
/// 5| _rs1_ is a positive subnormal number.
/// 6| _rs1_ is a positive normal number.
/// 7| _rs1_ is +∞.
/// 8| _rs1_ is a signaling NaN.
/// 9| _rs1_ is a quiet NaN.
///
/// The last 2 bits may or may not be wrong in some (hopefully older) architectures
/// because of encoding shenanigans I don't know how to deal with.
/// See [https://en.wikipedia.org/wiki/NaN#Encoding](https://en.wikipedia.org/wiki/NaN#Encoding)
pub fn class_mask(f: f32) -> u32 {
    use std::num::FpCategory::*;
    let neg = f.is_sign_negative();
    let bit = match f.classify() {
        Infinite if neg => 0,
        Normal if neg => 1,
        Subnormal if neg => 2,
        Zero if neg => 3,
        Zero => 4,
        Subnormal => 5,
        Normal => 6,
        Infinite => 7,
        Nan if neg => 9,
        Nan => 8,
    };

    1_u32 << bit
}
