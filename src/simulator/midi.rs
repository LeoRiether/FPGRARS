//!
//! Defines MIDI ecalls
//!

use midir::{MidiOutput, MidiOutputConnection};
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use parking_lot::Mutex;
use std::thread;
use std::time::Duration;

const NOTE_ON: u8 = 0x90;
const NOTE_OFF: u8 = 0x80;
const PROGRAM_CHANGE: u8 = 0xC0;

const DEFAULT_PORT: usize = 0;

/// When [MidiPlayer::get_connection](struct.MidiPlayer.html#method.get_connection) can't connect
/// successfully, it will return an Err(ConnectionError)
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
            CouldntCreateOutput => {
                write!(f, "Couldn't create MidiOutput, MIDI ecalls will not work")
            }
            NoPorts => write!(f, "No MIDI output port found. MIDI ecalls will not work"),
            CouldntConnect => write!(
                f,
                "Couldn't connect the MIDI output to port, MIDI ecalls will not work"
            ),
        }
    }
}

impl Error for ConnectionError {}

struct MidiPlayerData {
    conn: MidiOutputConnection,

    /// `channels[x]` is the channel that will be used the next time instrument `x`
    /// is played. We alternate between channels 0..=15 for each instrument, so overlapping
    /// notes can be played. Note that channel 9 isn't used because it's only for percussion and
    /// makes different sounds
    channels: [u8; 128],
}

/// A MidiPlayer connects to a MidiOutputConnection and plays notes.
/// `play_note()` blocks the thread for the duration of the note.
#[derive(Clone)]
pub struct MidiPlayer(Option<Arc<Mutex<MidiPlayerData>>>);

impl MidiPlayer {
    fn get_connection(port: Option<usize>) -> Result<MidiOutputConnection, ConnectionError> {
        let midi_out = match MidiOutput::new("FPGRARS_MIDI_Out") {
            Ok(x) => x,
            Err(_) => return Err(ConnectionError::CouldntCreateOutput),
        };

        let ports = midi_out.ports();
        if port.is_some() && port.unwrap() >= ports.len() {
            panic!(
                "Provided MIDI port ({}) isn't valid (should be in range 0-{} inclusive)",
                port.unwrap(),
                ports.len() as isize - 1
            );
        }
        let port = match ports.len() {
            0 => return Err(ConnectionError::NoPorts),
            1 => &ports[port.unwrap_or(DEFAULT_PORT)],
            _ => {
                if port.is_none() {
                    eprintln!("Warning: more than one MIDI output port found. Port 0 will be used, unless you provide a --port flag");
                }
                &ports[port.unwrap_or(DEFAULT_PORT)]
            }
        };

        match midi_out.connect(port, "FPGRARS_MIDI_conn") {
            Ok(conn) => Ok(conn),
            Err(_) => return Err(ConnectionError::CouldntConnect),
        }
    }

    pub fn new(port: Option<usize>) -> Self {
        match Self::get_connection(port) {
            Ok(c) => {
                Self(Some(Arc::new(Mutex::new(MidiPlayerData{
                    conn: c,
                    channels: [0; 128]
                }))))
            }
            Err(e) => {
                eprintln!("Warning: {}", e);
                Self(None)
            }
        }
    }

    /// Plays a blocking MIDI note (unless there's no connection)
    fn play_note(&self, pitch: u8, duration: u32, instrument: u8, velocity: u8) {
        if self.0.is_none() { return; }

        let ch;
        {
            let mut d = self.0.as_ref().unwrap().lock();
            ch = get_channel(&mut d, instrument);

            d.conn.send(&[PROGRAM_CHANGE | ch, instrument])
                .expect("Failed to send PROGRAM_CHANGE message to MIDI output");
            d.conn.send(&[NOTE_ON | ch, pitch, velocity])
                .expect("Failed to send NOTE_ON message to MIDI output");
        }
        thread::sleep(Duration::from_millis(duration as u64));
        {
            let mut d = self.0.as_ref().unwrap().lock();
            d.conn.send(&[NOTE_OFF | ch, pitch, velocity])
                .expect("Failed to send NOTE_OFF message to MIDI output");
        }
    }

    /// Tries to handle a MIDI ecall and returns whether we could handle it
    pub fn handle_ecall(&self, ecall: u32, registers: &mut [u32; 32]) -> bool {
        if (ecall != 31 && ecall != 33) || self.0.is_none() {
            return false;
        }

        let pitch = registers[10]; // a0
        let duration = registers[11] as i32; // a1
        let instrument = registers[12]; // a2
        let velocity = registers[13]; // a3

        let player = self.clone();
        let play = move || {
            player.play_note(
                pitch as u8,
                if duration < 0 { 1000 } else { duration as u32 },
                instrument as u8,
                if (0..128).contains(&velocity) { velocity as u8 } else { 100 },
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
}

/// Get and increment channel for a given instrument
fn get_channel(d: &mut MidiPlayerData, instrument: u8) -> u8 {
    let ch = &mut d.channels[instrument as usize];
    *ch = (*ch + 1) & 0xF; // channels 0x0 through 0xF are used
    if *ch == 9 { *ch = 10; } // ch 9 is only for percussion...
    *ch
}
