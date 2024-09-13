use core::marker::PhantomData;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::convert::TryFrom;
use std::ops::{Add, Sub};

// trait StringInstrument<const N: usize, T>
// where
//     T: Tuning<N>,
// {
// }

// impl<T> StringInstrument<6, T> for Guitar6String<T> where T: Tuning<6> {}
// impl<T> StringInstrument<4, T> for Base4String<T> where T: Tuning<4> {}

pub trait Tuning<const N: usize> {
    const BASE_NOTES: [Note; N];
}

#[derive(Debug)]
pub struct EADGBE {}

impl Tuning<6> for EADGBE {
    const BASE_NOTES: [Note; 6] = [
        Note {
            semi_tone: SemiTone::E,
            octave: 2,
        },
        Note {
            semi_tone: SemiTone::A,
            octave: 2,
        },
        Note {
            semi_tone: SemiTone::D,
            octave: 2,
        },
        Note {
            semi_tone: SemiTone::G,
            octave: 2,
        },
        Note {
            semi_tone: SemiTone::B,
            octave: 2,
        },
        Note {
            semi_tone: SemiTone::E,
            octave: 2,
        },
    ];
}

#[derive(Debug)]
struct EADG {}

impl Tuning<4> for EADG {
    const BASE_NOTES: [Note; 4] = [
        Note {
            semi_tone: SemiTone::E,
            octave: 2,
        },
        Note {
            semi_tone: SemiTone::A,
            octave: 2,
        },
        Note {
            semi_tone: SemiTone::D,
            octave: 2,
        },
        Note {
            semi_tone: SemiTone::G,
            octave: 2,
        },
    ];
}

#[derive(Debug)]
struct Guitar6String<T: Tuning<6>> {
    tuning: T,
}

#[derive(Debug)]
struct Base4String<T: Tuning<4>> {
    tuning: T,
}

#[derive(Debug, Copy, Clone)]
pub struct FretNote<const N: usize, T>
where
    T: Tuning<N>,
{
    // string index typically 0..3 for base, 0..5 for guitar,
    // 0 is the lowest string for now
    pub string: u8,
    pub fret: u8,         // the fret index for the note, 0 for open string
    pub start: f32,       // start time in beats, 3.0 denotes a note struct at beat 3
    pub ext: Option<f32>, // off time
    _marker: PhantomData<T>,
}

impl<const N: usize, T> From<FretNote<N, T>> for Note
where
    T: Tuning<N>,
{
    fn from(note: FretNote<N, T>) -> Self {
        T::BASE_NOTES[note.string as usize] + note.fret.into()
    }
}

impl<const N: usize, T> From<&FretNote<N, T>> for Note
where
    T: Tuning<N>,
{
    fn from(note: &FretNote<N, T>) -> Self {
        T::BASE_NOTES[note.string as usize] + note.fret.into()
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]

enum SemiTone {
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

#[derive(Debug, Clone, Copy)]
pub struct Note {
    semi_tone: SemiTone,
    octave: u8,
}

impl From<u8> for Note {
    fn from(v: u8) -> Self {
        Self {
            semi_tone: SemiTone::try_from(v % 12).unwrap(),
            octave: v / 12,
        }
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
struct Hz(f32);

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

pub struct Notes<const N: usize, T>(pub Vec<FretNote<N, T>>)
where
    T: Tuning<N>;

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn test_hz() {
        let fret_note: FretNote<6, EADGBE> = FretNote {
            string: 1,
            fret: 0,
            start: 10.0,
            ext: Some(11.0),
            _marker: PhantomData,
        };

        let note: Note = (&fret_note).into();

        let hz: Hz = note.into();

        println!("note {:?}, freq {:?}", fret_note, hz)
    }

    #[test]
    fn test_from() {
        let n = FretNote::<6, EADGBE> {
            string: 0,
            fret: 3,
            start: 10.0,
            ext: Some(11.0),
            _marker: PhantomData,
        };

        let oct: Note = 12.into();
        let one: Note = 8.into();
        let n: Note = n.into();
        println!("n {:?}", n);
        let m: Note = n + oct;
        println!("m {:?}", m);
        let s: Note = m - one;
        println!("s {:?}", s);
        let max: Note = 255.into();
        println!("max {:?}", max);
    }
}

impl Default for Notes<6, EADGBE> {
    fn default() -> Self {
        Notes(vec![
            FretNote {
                string: 0,
                fret: 3,
                start: 0.0,
                ext: None,
                _marker: PhantomData,
            },
            FretNote {
                string: 1,
                fret: 1,
                start: 1.0,
                ext: None,
                _marker: PhantomData,
            },
            FretNote {
                string: 2,
                fret: 0,
                start: 2.0,
                ext: None,
                _marker: PhantomData,
            },
            FretNote {
                string: 3,
                fret: 5,
                start: 3.0,
                ext: None,
                _marker: PhantomData,
            },
            FretNote {
                string: 4,
                fret: 2,
                start: 4.0,
                ext: None,
                _marker: PhantomData,
            },
            FretNote {
                string: 5,
                fret: 2,
                start: 4.0,
                ext: Some(4.5),
                _marker: PhantomData,
            },
            FretNote {
                string: 1,
                fret: 2,
                start: 5.0,
                ext: None,
                _marker: PhantomData,
            },
            FretNote {
                string: 1,
                fret: 3,
                start: 5.25,
                ext: None,
                _marker: PhantomData,
            },
            FretNote {
                string: 2,
                fret: 3,
                start: 6.0,
                ext: None,
                _marker: PhantomData,
            },
            FretNote {
                string: 2,
                fret: 10,
                start: 10.0,
                ext: Some(11.0),
                _marker: PhantomData,
            },
        ])
    }
}
