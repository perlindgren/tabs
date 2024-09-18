use core::marker::PhantomData;
use std::{cell::RefCell, fmt::Debug, rc::Rc, sync::Arc};
mod note;
pub use note::*;

pub trait Tuning {
    fn tuning(&self) -> &'static [Note];
}

impl Debug for dyn Tuning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tuning:{:?}", self.tuning());
        Ok(())
    }
}

pub enum Tunings {
    EADGBE,
    EADG,
}

#[derive(Debug, Clone, Copy)]
pub struct EADGBE {}

impl EADGBE {
    const ROOT_NOTES: [Note; 6] = [
        Note::new(SemiTone::E, 2),
        Note::new(SemiTone::A, 2),
        Note::new(SemiTone::D, 3),
        Note::new(SemiTone::G, 3),
        Note::new(SemiTone::B, 3),
        Note::new(SemiTone::E, 4),
    ];
}

impl Tuning for EADGBE {
    fn tuning(&self) -> &'static [Note] {
        &EADGBE::ROOT_NOTES
    }
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone)]
pub struct FretNote {
    // string index typically 0..3 for base, 0..5 for guitar,
    // 0 is the lowest string for now
    pub string: u8,
    pub fret: u8,         // the fret index for the note, 0 for open string
    pub start: f32,       // start time in beats, 3.0 denotes a note struct at beat 3
    pub ext: Option<f32>, // off time
    pub tuning: Rc<RefCell<dyn Tuning>>,
}

impl FretNote {
    pub fn new(
        string: u8,
        fret: u8,
        start: f32,
        ext: Option<f32>,
        tuning: Rc<RefCell<dyn Tuning>>,
    ) -> Self {
        FretNote {
            string,
            fret,
            start,
            ext,
            tuning,
        }
    }
}

impl<'a> From<&FretNote> for Note {
    fn from(note: &FretNote) -> Self {
        note.tuning.borrow().tuning()[note.string as usize] + note.fret.into()
    }
}

impl<'a> From<FretNote> for Note {
    fn from(note: FretNote) -> Self {
        note.tuning.borrow().tuning()[note.string as usize] + note.fret.into()
    }
}
#[derive(Debug)]
pub struct FretNotes(pub Vec<FretNote>);
/*
impl<'a> Default for FretNotes<'a, EADGBE> {
    fn default() -> Self {
        static TUNING: EADGBE = EADGBE {};
        let tuning = &Box::new(TUNING);
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
}*/

#[cfg(test)]
mod test {
    use std::{cell::RefCell, rc::Rc};

    use super::*;

    #[test]
    fn test_hz() {
        let eadgbe = EADGBE {};
        let tuning: Rc<RefCell<dyn Tuning>> = Rc::new(RefCell::new(eadgbe));
        let fret_note: FretNote = FretNote::new(1, 0, 0.0, None, tuning);

        let note: Note = (&fret_note).into();

        let hz: Hz = note.into();

        println!("note {:?}, freq {:?}", fret_note, hz)
    }

    #[test]
    fn test_from() {
        let tuning: Rc<RefCell<dyn Tuning>> = Rc::new(RefCell::new(EADGBE {}));
        let n = FretNote::new(0, 3, 0.0, None, tuning);

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
