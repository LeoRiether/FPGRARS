///
/// If a type implements IntoRegister, then we can store its bit regresentation
/// in a 32-bit register as a u32
///
pub trait IntoRegister {
    fn into(self) -> u32;
}

macro_rules! impl_into_reg {
    ($type:ident, $conv:ident) => {
        impl IntoRegister for $type {
            fn into(self) -> u32 {
                self as $conv as u32
            }
        }
    };
}

impl_into_reg!(u32, u32);
impl_into_reg!(i32, u32);
impl_into_reg!(u16, u16);
impl_into_reg!(i16, u16);
impl_into_reg!(u8, u8);
impl_into_reg!(i8, u8);

pub trait FromRegister {
    fn from(x: u32) -> Self;
}

macro_rules! impl_from_reg {
    ($type:ident) => {
        impl FromRegister for $type {
            fn from(x: u32) -> Self {
                x as $type
            }
        }
    };
}

impl_from_reg!(u32);
impl_from_reg!(i32);
impl_from_reg!(u16);
impl_from_reg!(i16);
impl_from_reg!(u8);
impl_from_reg!(i8);