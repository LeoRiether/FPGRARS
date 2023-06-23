use super::error::ParserError as Error;
use hashbrown::HashMap;

pub const TIME_INDEX: u8 = 0;
pub const MISA_INDEX: u8 = 1;
pub const UEPC_INDEX: u8 = 2;
pub const USTATUS_INDEX: u8 = 3;
pub const UTVEC_INDEX: u8 = 4;
pub const UCAUSE_INDEX: u8 = 5;

pub type RegMap = HashMap<String, u8>;

#[derive(Debug)]
pub struct RegNames {
    pub regs: RegMap,
    pub floats: RegMap,
    pub status: RegMap,
}

impl Default for RegNames {
    fn default() -> Self {
        Self {
            regs: regs(),
            floats: floats(),
            status: status(),
        }
    }
}

fn insert_names(map: &mut RegMap, names: &[&'static str]) {
    for (i, name) in names.iter().enumerate() {
        map.insert(name.to_string(), i as u8);
    }
}

pub const REGVEC: [&str; 32] = [
    "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", "s0", "s1", "a0", "a1", "a2", "a3", "a4",
    "a5", "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11", "t3", "t4",
    "t5", "t6",
];

pub fn regs() -> RegMap {
    let mut map = RegMap::with_capacity(64);

    // Insert x-prefixed registers
    for i in 0..32 {
        map.insert(format!("x{}", i), i);
    }

    // Insert named registers
    insert_names(&mut map, &REGVEC);

    map
}

pub const FLOATVEC: [&str; 32] = [
    "ft0", "ft1", "ft2", "ft3", "ft4", "ft5", "ft6", "ft7", "fs0", "fs1", "fa0", "fa1", "fa2",
    "fa3", "fa4", "fa5", "fa6", "fa7", "fs2", "fs3", "fs4", "fs5", "fs6", "fs7", "fs8", "fs9",
    "fs10", "fs11", "ft8", "ft9", "ft10", "ft11",
];

pub fn floats() -> RegMap {
    let mut map = RegMap::with_capacity(64);

    // Insert f-prefixed registers
    for i in 0..32 {
        map.insert(format!("f{}", i), i);
    }

    // Insert named registers
    insert_names(&mut map, &FLOATVEC);

    map
}

pub fn status() -> RegMap {
    let mut map = RegMap::default();

    map.insert("time".to_owned(), TIME_INDEX);
    map.insert("misa".to_owned(), MISA_INDEX);
    map.insert("uepc".to_owned(), UEPC_INDEX);
    map.insert("ustatus".to_owned(), USTATUS_INDEX);
    map.insert("utvec".to_owned(), UTVEC_INDEX);
    map.insert("ucause".to_owned(), UCAUSE_INDEX);

    let names = vec!["uscratch", "utval", "instret", "instreth", "cycle", "timeh"];
    for name in names {
        map.insert(name.to_string(), map.len() as u8);
    }

    map.insert("0".to_owned(), USTATUS_INDEX);
    map.insert("3073".to_owned(), TIME_INDEX);
    map.insert("769".to_owned(), MISA_INDEX);
    map.insert("65".to_owned(), UEPC_INDEX);
    map.insert("0".to_owned(), USTATUS_INDEX);
    map.insert("5".to_owned(), UTVEC_INDEX);
    map.insert("66".to_owned(), UCAUSE_INDEX);

    map
}

pub trait TryGetRegister {
    fn try_get(&self, name: &str) -> Result<u8, Error>;
}

impl TryGetRegister for RegMap {
    fn try_get(&self, name: &str) -> Result<u8, Error> {
        self.get(name)
            .copied()
            .ok_or_else(|| Error::RegisterNotFound(name.to_owned()))
    }
}
