use super::Type;
use byteorder::{ByteOrder, LittleEndian};

pub fn unlabel(data: &mut [u8], i: usize, dt: super::Type, value: u32) {
    match dt {
        Type::Byte | Type::Asciz => {
            data[i] = value as u8;
        }
        Type::Half => LittleEndian::write_u16(&mut data[i..], value as u16),

        Type::Word => LittleEndian::write_u32(&mut data[i..], value),
        Type::Float => LittleEndian::write_f32(&mut data[i..], f32::from_bits(value)),
        Type::Align => panic!("'.align LABEL' is not supported!"),
    }
}
