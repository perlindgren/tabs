#[derive(Debug)]
pub struct Note {
    pub string: u8,       // typically 0..3 for base, 0..5 for guitar
    pub fret: u8,         // the fret fretition for the note
    pub start: f32,       // start time in beats, 3.0 denotes a note struct at beat 3
    pub ext: Option<f32>, // off time
}

pub struct Notes(pub Vec<Note>);

impl Default for Notes {
    fn default() -> Self {
        Notes(vec![
            Note {
                string: 0,
                fret: 3,
                start: 0.0,
                ext: None,
            },
            Note {
                string: 1,
                fret: 1,
                start: 1.0,
                ext: None,
            },
            Note {
                string: 2,
                fret: 0,
                start: 2.0,
                ext: None,
            },
            Note {
                string: 3,
                fret: 5,
                start: 3.0,
                ext: None,
            },
            Note {
                string: 4,
                fret: 2,
                start: 4.0,
                ext: None,
            },
            Note {
                string: 5,
                fret: 2,
                start: 4.0,
                ext: Some(4.5),
            },
            Note {
                string: 1,
                fret: 2,
                start: 5.0,
                ext: None,
            },
            Note {
                string: 1,
                fret: 3,
                start: 5.25,
                ext: None,
            },
            Note {
                string: 2,
                fret: 3,
                start: 6.0,
                ext: None,
            },
            Note {
                string: 2,
                fret: 10,
                start: 10.0,
                ext: Some(11.0),
            },
        ])
    }
}
