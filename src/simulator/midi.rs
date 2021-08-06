//!
//! Defines MIDI ecalls
//!

use std::thread;
use std::sync::{Mutex};
use std::time::Duration;
use lazy_static::lazy_static;
use midir::{MidiOutput, MidiOutputConnection};

// leaks a MIDI connection but that's fine
lazy_static! {
    static ref CONN: Option<Mutex<MidiOutputConnection>> = {
        let midi_out = match MidiOutput::new("FPGRARS_MIDI_Out") {
            Ok(x) => x,
            Err(_) => {
                eprintln!("Warning: couldn't create MidiOutput, MIDI ecalls will not work");
                return None;
            }
        };

        let ports = midi_out.ports();
        let port = match ports.len() {
            0 => {
                eprintln!("Warning: no MIDI output port found. MIDI ecalls will not work");
                return None;
            }
            1 => &ports[0],
            _ => {
                eprintln!("Warning: more than one MIDI output port found, choosing first one arbitrarily...");
                &ports[0]
            },
        };

        match midi_out.connect(port, "FPGRARS_MIDI_conn") {
            Ok(conn) => Some(Mutex::new(conn)),
            Err(_) => {
                eprintln!("Warning: couldn't connect the MIDI output to port, MIDI ecalls will not work");
                None
            }
        }
    };
}

fn play_note(
    pitch: u8,
    duration: u32,
    instrument: u8,
    velocity: u8,
) {
    const NOTE_ON: u8 = 0x90;
    const NOTE_OFF: u8 = 0x80;
    const PROGRAM_CHANGE: u8 = 0xC0;

    {
        let mut conn = CONN.as_ref().unwrap().lock().unwrap();
        conn.send(&[PROGRAM_CHANGE, instrument])
            .expect("Failed to send PROGRAM_CHANGE message to MIDI output");
        conn.send(&[NOTE_ON, pitch, velocity])
            .expect("Failed to send NOTE_ON message to MIDI output");
    }
    thread::sleep(Duration::from_millis(duration as u64));
    {
        let mut conn = CONN.as_ref().unwrap().lock().unwrap();
        conn.send(&[NOTE_OFF, pitch, velocity])
            .expect("Failed to send NOTE_OFF message to MIDI output");
    }
}

/// Tries to handle a MIDI ecall and returns whether we could handle it
pub fn handle_ecall(
    ecall: u32,
    registers: &mut [u32; 32],
) -> bool {
    if (ecall != 31 && ecall != 33) || CONN.is_none() {
        return false;
    }

    let pitch = registers[10]; // a0
    let duration = registers[11] as i32; // a1
    let instrument = registers[12]; // a2
    let velocity = registers[13]; // a3

    let play = move || {
        play_note(
            pitch as u8,
            if duration <= 0 { 1000 } else { duration as u32 },
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

