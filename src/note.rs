// use core::marker::PhantomData;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::convert::TryFrom;
use std::ops::{Add, Sub};

#[derive(Debug, Eq, PartialEq, Copy, Clone, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]

pub enum SemiTone {
    C,
    CSharpDFlat,
    D,
    DSharpEFlat,
    E,
    F,
    FSharpGFlat,
    G,
    GSharpAFlat,
    A,
    ASharpBFlat,
    B,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Note {
    semi_tone: SemiTone,
    octave: u8,
}

impl Note {
    pub const fn new(semi_tone: SemiTone, octave: u8) -> Self {
        Note { semi_tone, octave }
    }
}

impl From<u8> for Note {
    fn from(v: u8) -> Self {
        Note::new(SemiTone::try_from(v % 12).unwrap(), v / 12)
    }
}

impl From<Note> for u8 {
    fn from(n: Note) -> Self {
        let s: u8 = n.semi_tone.into();
        s + 12 * n.octave
    }
}

impl Add for Note {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let s1: u8 = self.into();
        let s2: u8 = other.into();
        (s1 + s2).into()
    }
}

impl Sub for Note {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let s1: u8 = self.into();
        let s2: u8 = other.into();
        (s1 - s2).into()
    }
}

#[derive(Debug)]
pub struct Hz(pub f32);

impl From<Note> for Hz {
    fn from(n: Note) -> Self {
        let a0 = Note {
            semi_tone: SemiTone::A,
            octave: 0,
        };

        let diff_semitones: u8 = (n - a0).into();

        let exp = diff_semitones as f64 / 12.0;
        let freq_diff = exp.exp2();

        Hz(freq_diff as f32 * 27.5)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct MidiNote(pub u32);

impl From<MidiNote> for Note {
    fn from(value: MidiNote) -> Self {
        let id = value.0;
        let semitone = id % 12;
        let semitone = match semitone {
            0 => SemiTone::C,
            1 => SemiTone::CSharpDFlat,
            2 => SemiTone::D,
            3 => SemiTone::DSharpEFlat,
            4 => SemiTone::E,
            5 => SemiTone::F,
            6 => SemiTone::FSharpGFlat,
            7 => SemiTone::G,
            8 => SemiTone::GSharpAFlat,
            9 => SemiTone::A,
            10 => SemiTone::ASharpBFlat,
            11 => SemiTone::B,
            12 => SemiTone::C,
            _ => unreachable!(),
        };
        // 0th is C,,,
        let octave = ((id - id % 12) / 12) - 1;

        Note {
            semi_tone: semitone,
            octave: octave as u8,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hz() {
        let note = Note::new(SemiTone::E, 1);
        let hz: Hz = note.into();
        println!("note {:?}, freq {:?}", note, hz);
        let note = note + 1.into();
        let hz: Hz = note.into();
        println!("note {:?}, freq {:?}", note, hz);
    }
}
