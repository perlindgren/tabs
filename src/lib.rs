use core::marker::PhantomData;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::convert::TryFrom;
use std::ops::Add;

trait StringInstrument<const N: usize, T>
where
    T: Tuning<N>,
{
}

impl<T> StringInstrument<6, T> for Guitar6String<T> where T: Tuning<6> {}
impl<T> StringInstrument<4, T> for Base4String<T> where T: Tuning<4> {}

trait Tuning<const N: usize> {
    const BASE_NOTES: [Note; N];
}

#[derive(Debug)]
struct EADGBE {}

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

#[derive(Debug)]
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
        let b = T::BASE_NOTES[note.string as usize] + note.fret.into();
        b
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
struct Note {
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
        let n: Note = (s1 + s2).into();

        n
    }
}

pub struct Notes<const N: usize, T>(pub Vec<FretNote<N, T>>)
where
    T: Tuning<N>;

#[cfg(test)]
mod test {
    use crate::*;

    #[test]

    fn test_from() {
        let n = FretNote::<6, EADGBE> {
            string: 2,
            fret: 10,
            start: 10.0,
            ext: Some(11.0),
            _marker: PhantomData,
        };

        let n: Note = n.into();
        println!("n {:?}", n);
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
