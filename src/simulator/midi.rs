//!
//! Defines MIDI ecalls
//!

use midir::{MidiOutput, MidiOutputConnection};
use std::error::Error;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const NOTE_ON: u8 = 0x90;
const NOTE_OFF: u8 = 0x80;
const PROGRAM_CHANGE: u8 = 0xC0;

#[derive(Debug)]
enum ConnectionError {
    CouldntCreateOutput,
    NoPorts,
    CouldntConnect,
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ConnectionError::*;
        match self {
            CouldntCreateOutput => write!(f, "Couldn't create MidiOutput, MIDI ecalls will not work"),
            NoPorts => write!(f, "No MIDI output port found. MIDI ecalls will not work"),
            CouldntConnect => write!(f, "Couldn't connect the MIDI output to port, MIDI ecalls will not work"),
        }
    }
}

impl Error for ConnectionError {  }

pub struct MidiPlayer {
    conn: Option<Mutex<MidiOutputConnection>>,
}

impl MidiPlayer {
    fn get_connection(port: Option<usize>) -> Result<Mutex<MidiOutputConnection>, ConnectionError> {
        let midi_out = match MidiOutput::new("FPGRARS_MIDI_Out") {
            Ok(x) => x,
            Err(_) => return Err(ConnectionError::CouldntCreateOutput),
        };

        let ports = midi_out.ports();
        if port.is_some() && port.unwrap() >= ports.len() {
            panic!("Provided MIDI port ({}) isn't valid (should be in range 0-{} inclusive)", port.unwrap(), ports.len() as isize - 1);
        }
        let port = match ports.len() {
            0 => return Err(ConnectionError::NoPorts),
            1 => &ports[port.unwrap_or(0)],
            _ => {
                if port.is_none() {
                    eprintln!("Warning: more than one MIDI output port found. Port 0 will be used, unless you provide a --port flag");
                }
                &ports[port.unwrap_or(0)]
            },
        };

        match midi_out.connect(port, "FPGRARS_MIDI_conn") {
            Ok(conn) => Ok(Mutex::new(conn)),
            Err(_) => return Err(ConnectionError::CouldntConnect),
        }
    }

    pub fn new(port: Option<usize>) -> Self {
        let conn = match Self::get_connection(port) {
            Ok(c) => Some(c),
            Err(e) => {
                eprintln!("Warning: {}", e);
                None
            }
        };
        Self { conn }
    }

    fn play_note(
        &self,
        pitch: u8,
        duration: u32,
        instrument: u8,
        velocity: u8,
    ) {
        {
            let mut conn = self.conn.as_ref().unwrap().lock().unwrap();
            conn.send(&[PROGRAM_CHANGE, instrument])
                .expect("Failed to send PROGRAM_CHANGE message to MIDI output");
            conn.send(&[NOTE_ON, pitch, velocity])
                .expect("Failed to send NOTE_ON message to MIDI output");
        }
        thread::sleep(Duration::from_millis(duration as u64));
        {
            let mut conn = self.conn.as_ref().unwrap().lock().unwrap();
            conn.send(&[NOTE_OFF, pitch, velocity])
                .expect("Failed to send NOTE_OFF message to MIDI output");
        }
    }
}


/// Tries to handle a MIDI ecall and returns whether we could handle it
pub fn handle_ecall(
    player: &Arc<MidiPlayer>,
    ecall: u32,
    registers: &mut [u32; 32],
) -> bool {
    if (ecall != 31 && ecall != 33) || player.conn.is_none() {
        return false;
    }

    let pitch = registers[10]; // a0
    let duration = registers[11] as i32; // a1
    let instrument = registers[12]; // a2
    let velocity = registers[13]; // a3

    let player = player.clone();
    let play = move || {
        player.play_note(
            pitch as u8,
            if duration < 0 { 1000 } else { duration as u32 },
            instrument as u8,
            if (0..128).contains(&velocity) { velocity as u8 } else { 100 }
        );
    };

    if ecall == 31 {
        // MIDI Out Async
        thread::spawn(play);
    } else {
        // MIDI Out Sync
        play();
    }

    true
}

