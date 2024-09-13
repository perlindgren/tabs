mod note;
use note::*;

pub trait Tuning {
    fn tuning(&self) -> &[Note];
}

#[derive(Debug)]
pub struct EADGBE {}

impl EADGBE {
    const ROOT_NOTES: [Note; 6] = [
        Note::new(SemiTone::E, 2),
        Note::new(SemiTone::A, 2),
        Note::new(SemiTone::D, 2),
        Note::new(SemiTone::G, 2),
        Note::new(SemiTone::B, 2),
        Note::new(SemiTone::E, 3),
    ];
}

impl Tuning for EADGBE {
    fn tuning(&self) -> &[Note] {
        &EADGBE::ROOT_NOTES
    }
}

#[derive(Debug)]
pub struct EADG {}

impl EADG {
    const ROOT_NOTES: [Note; 4] = [
        Note::new(SemiTone::E, 2),
        Note::new(SemiTone::A, 2),
        Note::new(SemiTone::D, 2),
        Note::new(SemiTone::G, 2),
    ];
}

impl Tuning for EADG {
    fn tuning(&self) -> &[Note] {
        &EADG::ROOT_NOTES
    }
}

#[derive(Debug, Copy, Clone)]
pub struct FretNote<T>
where
    T: Tuning,
{
    // string index typically 0..3 for base, 0..5 for guitar,
    // 0 is the lowest string for now
    pub string: u8,
    pub fret: u8,         // the fret index for the note, 0 for open string
    pub start: f32,       // start time in beats, 3.0 denotes a note struct at beat 3
    pub ext: Option<f32>, // off time
    pub tuning: T,
}

impl<T> From<&FretNote<T>> for Note
where
    T: Tuning,
{
    fn from(note: &FretNote<T>) -> Self {
        note.tuning.tuning()[note.string as usize] + note.fret.into()
    }
}

impl<T> From<FretNote<T>> for Note
where
    T: Tuning,
{
    fn from(note: FretNote<T>) -> Self {
        note.tuning.tuning()[note.string as usize] + note.fret.into()
    }
}

pub struct FretNotes<T>(pub Vec<FretNote<T>>)
where
    T: Tuning;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hz() {
        let fret_note: FretNote<EADGBE> = FretNote {
            string: 1,
            fret: 0,
            start: 10.0,
            ext: Some(11.0),
            tuning: EADGBE {},
        };

        let note: Note = (&fret_note).into();

        let hz: Hz = note.into();

        println!("note {:?}, freq {:?}", fret_note, hz)
    }

    #[test]
    fn test_from() {
        let n = FretNote::<EADGBE> {
            string: 0,
            fret: 3,
            start: 10.0,
            ext: Some(11.0),
            tuning: EADGBE {},
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

// impl Default for Notes<6, EADGBE> {
//     fn default() -> Self {
//         Notes(vec![
//             FretNote {
//                 string: 0,
//                 fret: 3,
//                 start: 0.0,
//                 ext: None,
//                 _marker: PhantomData,
//             },
//             FretNote {
//                 string: 1,
//                 fret: 1,
//                 start: 1.0,
//                 ext: None,
//                 _marker: PhantomData,
//             },
//             FretNote {
//                 string: 2,
//                 fret: 0,
//                 start: 2.0,
//                 ext: None,
//                 _marker: PhantomData,
//             },
//             FretNote {
//                 string: 3,
//                 fret: 5,
//                 start: 3.0,
//                 ext: None,
//                 _marker: PhantomData,
//             },
//             FretNote {
//                 string: 4,
//                 fret: 2,
//                 start: 4.0,
//                 ext: None,
//                 _marker: PhantomData,
//             },
//             FretNote {
//                 string: 5,
//                 fret: 2,
//                 start: 4.0,
//                 ext: Some(4.5),
//                 _marker: PhantomData,
//             },
//             FretNote {
//                 string: 1,
//                 fret: 2,
//                 start: 5.0,
//                 ext: None,
//                 _marker: PhantomData,
//             },
//             FretNote {
//                 string: 1,
//                 fret: 3,
//                 start: 5.25,
//                 ext: None,
//                 _marker: PhantomData,
//             },
//             FretNote {
//                 string: 2,
//                 fret: 3,
//                 start: 6.0,
//                 ext: None,
//                 _marker: PhantomData,
//             },
//             FretNote {
//                 string: 2,
//                 fret: 10,
//                 start: 10.0,
//                 ext: Some(11.0),
//                 _marker: PhantomData,
//             },
//         ])
//     }
// }
