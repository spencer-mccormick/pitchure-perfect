extern crate clap;
use clap::{Arg, App, SubCommand};
use std::fs::File;

const SAMPLE_RATE: u64 = 48000;
const SAMPLE_DEPTH: u16 = 24;

fn main() {
    let matches = App::new("Pitchure Perfect")
                          .version("0.1")
                          .author("Spencer McCormick")
                          .about("Music, but images. Yeah, its as stupid as it sounds...")
                          .arg(Arg::with_name("INPUT")
                               .short("i")
                               .long("input")
                               .value_name("FILE")
                               .help("PNG file to read")
                               .required(true)
                               .takes_value(true))
                          .arg(Arg::with_name("OUTPUT")
                                .short("o")
                                .long("output")
                                .value_name("FILE")
                                .help("Opus file to write")
                                .required(true)
                                .takes_value(true))
                          .arg(Arg::with_name("LENGTH")
                               .short("l")
                               .long("length")
                               .takes_value(true)
                               .help("Length (in seconds) of each triad"))
                          .arg(Arg::with_name("NUMBER")
                               .short("n")
                               .long("number")
                               .takes_value(true)
                               .help("Number of pitches in the chord"))
                          .arg(Arg::with_name("PITCH")
                               .short("p")
                               .long("pitch")
                               .takes_value(true)
                               .help("Pitch to scale against"))
                          .get_matches();

    let input_file_name = matches.value_of("INPUT").unwrap();
    let output_file_name = matches.value_of("OUTPUT").unwrap();
    let length_str = matches.value_of("LENGTH").unwrap_or("0.5");
    let number_str = matches.value_of("NUMBER").unwrap_or("3");
    let pitch_str = matches.value_of("PITCH").unwrap_or("440.0");

    let length: f64 = length_str.parse().unwrap();
    let number: u64 = number_str.parse().unwrap();
    let pitch: f64 = pitch_str.parse().unwrap();

    let decoder = png::Decoder::new(File::open(input_file_name).unwrap());
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();
    let bytes = &buf[..info.buffer_size()];

    let mut chords: Vec<Vec<f64>> =  Vec::new();
    let mut chord: Vec<f64> = Vec::new();
    let mut modulus: u64 = 0;

    for byte in bytes {
        if (modulus % number) == 0 {
            chords.push(chord.clone());
            chord = Vec::new();
        }

        chord.push((*byte as f64).ln() * pitch);

        modulus += 1;
    }

    let mut track: Vec<f32> = Vec::new();

    let wav_spec = hound::WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE as u32,
        bits_per_sample: SAMPLE_DEPTH,
        sample_format: hound::SampleFormat::Int,
    };

    let mut wav_writer = hound::WavWriter::create(output_file_name, wav_spec).unwrap();
    
    for c in chords {
        let cd = render_chord(c, length);

        for c in cd {
            wav_writer.write_sample((c * 16777000 as f64) as i32);
        }
    }

    wav_writer.finalize().unwrap();
}

fn render_chord(pitches: Vec<f64>, length: f64) -> Vec<f64> {
    let mut amps: Vec<f64> = Vec::new();
    let samples: u64 = (SAMPLE_RATE as f64 * length).ceil() as u64;
    
    for i in 0..samples {
        let mut val: f64 = 0.0;

        for pitch in &pitches {
            val += (1.0 + ((i as f64 / SAMPLE_RATE as f64) * pitch * std::f64::consts::TAU).cos()) / (2.0 * pitches.len() as f64);
        }

        amps.push(val as f64);
    }

    amps
}
