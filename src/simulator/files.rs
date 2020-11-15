//!
//! Contains the definitions and procedures necessary for the file operations the RISC-V
//! code can perform. This includes opening a file, reading from it and writing to it.
//!

use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};

/// Maximum number of simultaneous open files
const MAX_DESCRIPTORS: i32 = 1 << 30;

/// Data structure to add, remove and fetch [Files](struct.File.html)
pub struct FileHolder {
    next: i32,
    items: BTreeMap<i32, fs::File>,
}

impl FileHolder {
    pub fn new() -> Self {
        Self {
            next: 0,
            items: BTreeMap::new(),
        }
    }

    fn gen_next_key(&mut self) {
        if self.items.len() >= MAX_DESCRIPTORS as usize {
            self.next = -1; // too many simultaneous descriptors, can't generate next key
            return;
        }

        self.next = (self.next + 1) % MAX_DESCRIPTORS;
        while self.items.contains_key(&self.next) {
            self.next = (self.next + 1) % MAX_DESCRIPTORS;
        }
    }

    /// Adds a file to the holder and returns its ID/descriptor
    pub fn add(&mut self, f: fs::File) -> i32 {
        let fd = self.next;
        self.items.insert(fd, f);
        self.gen_next_key();
        fd
    }

    /// Removes a file, given an ID/descriptor
    pub fn remove(&mut self, key: i32) -> Option<fs::File> {
        self.items.remove(&key)
    }

    /// Fetches a file, given an ID/descriptor
    pub fn get_mut(&mut self, key: i32) -> Option<&mut fs::File> {
        self.items.get_mut(&key)
    }
}

/// Open a file and return its descriptor
fn open(filepath: &str, flags: u32, holder: &mut FileHolder) -> i32 {
    let file_opt = match flags {
        0 => fs::File::open(&filepath).ok(),
        1 => fs::File::create(&filepath).ok(),
        9 => fs::OpenOptions::new().append(true).open(&filepath).ok(),
        _ => None,
    };

    let descriptor = file_opt.map(|f| holder.add(f)).unwrap_or(-1);
    descriptor
}

/// Close a file
fn close(fd: i32, holder: &mut FileHolder) {
    holder.remove(fd).map(|mut f| f.flush());
}

/// Seek to a position given by offset, starting from the start, end or current cursor.
/// Returns the new position of the cursor from the start of the file
fn seek(fd: i32, offset: u32, from_where: u32, holder: &mut FileHolder) -> i32 {
    let seek_action = match from_where {
        1 => SeekFrom::Current(offset as i32 as i64),
        2 => SeekFrom::End(offset as i32 as i64),
        _ => SeekFrom::Start(offset as u64),
    };

    holder
        .get_mut(fd)
        .and_then(|file| file.seek(seek_action).ok())
        .map(|x| x as i32)
        .unwrap_or(-1)
}

/// Read `len` bytes from a file and put them in `memory[buffer_start..buffer_start + len]`
fn read(
    fd: i32,
    buffer_start: u32,
    len: usize,
    holder: &mut FileHolder,
    memory: &mut super::Memory,
) -> i32 {
    holder
        .get_mut(fd)
        .and_then(|file| {
            memory.set_with(buffer_start as usize, 0, |buf, _| {
                file.take(len as u64).read(buf).ok()
            })
        })
        .map(|x| x as i32)
        .unwrap_or(-1)
}

/// Write `memory[buffer_start..buffer_start + len]` to a file
fn write(
    fd: i32,
    buffer_start: u32,
    len: usize,
    holder: &mut FileHolder,
    memory: &mut super::Memory,
) -> i32 {
    holder
        .get_mut(fd)
        .and_then(|file| memory.get_with(buffer_start as usize, |buf| file.write(&buf[..len]).ok()))
        .map(|x| x as i32)
        .unwrap_or(-1)
}

/// Tries to handle an ecall and returns whether we could handle it
pub fn handle_ecall(
    ecall: u32,
    holder: &mut FileHolder,
    registers: &mut [u32; 32],
    memory: &mut super::Memory,
) -> bool {
    match ecall {
        1024 => {
            // Open file
            let (a0, flags) = (registers[10] as usize, registers[11]);
            let filepath: String = (a0..)
                .map(|i| memory.get_byte(i) as char)
                .take_while(|&c| c != '\0')
                .collect();

            registers[10] = open(&filepath, flags, holder) as u32;

            true
        }

        57 => {
            // Close file
            let fd = registers[10] as i32;
            close(fd, holder);

            true
        }

        62 => {
            // LSeek
            let (fd, offset, from_where) = (registers[10] as i32, registers[11], registers[12]);

            registers[10] = seek(fd, offset, from_where, holder) as u32;

            true
        }

        63 => {
            // Read
            let (fd, buffer_start, len) =
                (registers[10] as i32, registers[11], registers[12] as usize);

            registers[10] = read(fd, buffer_start, len, holder, memory) as u32;

            true
        }

        64 => {
            // Write
            let (fd, buffer_start, len) =
                (registers[10] as i32, registers[11], registers[12] as usize);

            registers[10] = write(fd, buffer_start, len, holder, memory) as u32;

            true
        }

        _ => false,
    }
}
