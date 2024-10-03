use scorelib::gp;
use std::collections::HashMap;
use std::{io::Read, path::Path};
#[derive()]
pub struct DrumMapping(pub HashMap<i16, Drum>);

impl DrumMapping {
    pub fn new() -> Self {
        DrumMapping(HashMap::new())
    }
}
#[derive(Debug)]
pub struct DrumNotes {
    pub notes: Vec<DrumNote>,
    pub tempo: i16,
}

impl DrumNotes {
    pub fn from_path(path: &Path, track_id: &usize) -> Self {
        let mut song: gp::Song = gp::Song::default();
        let ext = path.extension().unwrap().to_str().unwrap();
        let mut f = std::fs::File::open(path).unwrap();
        let mut data: Vec<u8> = vec![];
        f.read_to_end(&mut data).ok();
        match ext {
            "gp3" => song.read_gp3(&data),
            "gp4" => song.read_gp4(&data),
            "gp5" => song.read_gp5(&data),
            _ => panic!("Invalid file extension (currently only .gp3, .gp4, .gp5 are supported)"),
        }
        let track = song
            .tracks
            .get(*track_id)
            .expect("The selected track does not exist");

        let tempo = song.tempo;

        let mut current_time = 0.0;

        let mut drum_notes = vec![];
        for measure in &track.measures {
            let voice = measure.voices.first().unwrap();
            for beat in &voice.beats {
                for note in &beat.notes {
                    let drum_note = DrumNote::new(current_time as f32, (note.value).into());
                    drum_notes.push(drum_note);
                    current_time += 1.0 / beat.duration.value as f32;
                }
            }
        }

        DrumNotes {
            notes: drum_notes,
            tempo,
        }
    }
}
#[derive(Debug)]
pub struct DrumNote {
    pub start: f32, // time in beats
    pub drum: Drum, // which drum to hit
    pub hit: bool,  // has this note been hit correctly
}

impl DrumNote {
    pub fn new(start: f32, drum: Drum) -> Self {
        DrumNote {
            start,
            drum,
            hit: false,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Drum {
    Snare,
    HihatClosed,
    HihatOpen,
    Crash,
    Tom1,
    Tom2,
    Ride,
    FloorTom,
    Kick,
}

// Standard MIDI Drum mapping
// https://usermanuals.finalemusic.com/SongWriter2012Win/Content/PercussionMaps.htm
impl From<i16> for Drum {
    fn from(value: i16) -> Self {
        match value {
            27 => todo!(),           //27 	Laser
            60 => todo!(),           //60 	High Bongo
            28 => todo!(),           //28 	Whip
            61 => todo!(),           //61 	Low Bongo
            29 => todo!(),           //29 	Scratch Push
            62 => todo!(),           //62 	Conga Dead Stroke
            30 => todo!(),           //30 	Scratch Pull
            63 => todo!(),           //63 	Conga
            31 => todo!(),           //31 	Stick Click
            64 => todo!(),           //64 	Tumba
            32 => todo!(),           //32 	Metronome Click
            65 => todo!(),           //65 	High Timbale
            34 => todo!(),           //34 	Metronome Bell
            66 => todo!(),           //66 	Low Timbale
            35 => Drum::Kick,        //35 	Bass Drum
            67 => todo!(),           //67 	High Agogo
            36 => Drum::Kick,        //36 	Kick Drum
            68 => todo!(),           //68 	Low Agogo
            37 => Drum::Snare,       //37 	Snare Cross Stick
            69 => todo!(),           //69 	Cabasa
            38 => Drum::Snare,       //38 	Snare Drum
            70 => todo!(),           //70 	Maracas
            39 => Drum::Snare,       //39 	Hand Clap
            71 => todo!(),           //71 	Whistle Short
            40 => Drum::Snare,       //40 	Electric Snare Drum
            72 => todo!(),           //72 	Whistle Long
            41 => Drum::FloorTom,    //41 	Floor Tom 2
            73 => todo!(),           //73 	Guiro Short
            42 => Drum::HihatClosed, //42 	Hi-Hat Closed
            74 => todo!(),           //74 	Guiro Long
            43 => Drum::FloorTom,    //43 	Floor Tom 1
            75 => todo!(),           //75 	Claves
            44 => Drum::HihatClosed, //44 	Hi-Hat Foot
            76 => todo!(),           //76 	High Woodblock
            45 => Drum::Tom2,        //45 	Low Tom
            77 => todo!(),           //77 	Low Woodblock
            46 => Drum::HihatOpen,   //46 	Hi-Hat Open
            78 => todo!(),           //78 	Cuica High
            47 => Drum::Tom2,        //47 	Low-Mid Tom
            79 => todo!(),           //79 	Cuica Low
            48 => Drum::Tom1,        //48 	High-Mid Tom
            80 => todo!(),           //80 	Triangle Mute
            49 => Drum::Crash,       //49 	Crash Cymbal
            81 => todo!(),           //81 	Triangle Open
            50 => Drum::Tom1,        //50 	High Tom
            82 => todo!(),           //82 	Shaker
            51 => Drum::Ride,        //51 	Ride Cymbal
            83 => todo!(),           //83 	Sleigh Bell
            52 => Drum::Crash,       //52 	China Cymbal
            84 => todo!(),           //84 	Bell Tree
            53 => Drum::Ride,        //53 	Ride Bell
            85 => todo!(),           //85 	Castanets
            54 => todo!(),           //54 	Tambourine
            86 => todo!(),           //86 	Surdu Dead Stroke
            55 => Drum::Crash,       //55 	Splash cymbal
            87 => todo!(),           //87 	Surdu
            56 => Drum::Ride,        //56 	Cowbell
            91 => todo!(),           //91 	Snare Drum Rod
            57 => Drum::Crash,       //57 	Crash Cymbal 2
            92 => todo!(),           //92 	Ocean Drum
            58 => todo!(),           //58 	Vibraslap
            93 => Drum::Snare,       //93 	Snare Drum Brush
            59 => Drum::Ride,        //59 	Ride Cymbal 2
            _ => todo!(),
        }
    }
}
