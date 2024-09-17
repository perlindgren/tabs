use core::marker::PhantomData;
mod note;
pub use note::*;

pub trait Tuning {
    fn tuning(&self) -> &'static [Note];
}

pub enum Tunings {
    EADGBE,
    EADG,
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
    fn tuning(&self) -> &'static [Note] {
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
    fn tuning(&self) -> &'static [Note] {
        &EADG::ROOT_NOTES
    }
}

#[derive(Debug, Copy, Clone)]
pub struct FretNote<'a, T>
where
    T: Tuning,
{
    // string index typically 0..3 for base, 0..5 for guitar,
    // 0 is the lowest string for now
    pub string: u8,
    pub fret: u8,         // the fret index for the note, 0 for open string
    pub start: f32,       // start time in beats, 3.0 denotes a note struct at beat 3
    pub ext: Option<f32>, // off time
    pub tuning: &'a T,
    pub _marker: PhantomData<T>,
}

impl<'a, T> FretNote<'a, T>
where
    T: Tuning,
{
    pub fn new(string: u8, fret: u8, start: f32, ext: Option<f32>, tuning: &'a T) -> Self {
        FretNote {
            string,
            fret,
            start,
            ext,
            tuning,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> From<&FretNote<'a, T>> for Note
where
    T: Tuning,
{
    fn from(note: &FretNote<T>) -> Self {
        note.tuning.tuning()[note.string as usize] + note.fret.into()
    }
}

impl<'a, T> From<FretNote<'a, T>> for Note
where
    T: Tuning,
{
    fn from(note: FretNote<T>) -> Self {
        note.tuning.tuning()[note.string as usize] + note.fret.into()
    }
}

pub struct FretNotes<'a, T>(pub Vec<FretNote<'a, T>>)
where
    T: Tuning;

impl<'a> Default for FretNotes<'a, EADGBE> {
    fn default() -> Self {
        let tuning = &EADGBE {};
        FretNotes(vec![
            FretNote::new(0, 3, 0.0, None, tuning),
            FretNote::new(1, 1, 1.0, None, tuning),
            FretNote::new(2, 3, 3.0, None, tuning),
            FretNote::new(3, 5, 3.0, None, tuning),
            FretNote::new(4, 2, 4.0, None, tuning),
            FretNote::new(5, 2, 4.5, None, tuning),
            FretNote::new(2, 6, 5.0, None, tuning),
            FretNote::new(1, 3, 5.25, None, tuning),
            FretNote::new(2, 3, 6.0, None, tuning),
            FretNote::new(2, 10, 10.0, Some(11.0), tuning),
        ])
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hz() {
        let tuning = &EADGBE {};
        let fret_note: FretNote<EADGBE> = FretNote::new(1, 0, 0.0, None, tuning);

        let note: Note = (&fret_note).into();

        let hz: Hz = note.into();

        println!("note {:?}, freq {:?}", fret_note, hz)
    }

    #[test]
    fn test_from() {
        let tuning = &EADGBE {};
        let n = FretNote::<EADGBE>::new(0, 3, 0.0, None, tuning);

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
