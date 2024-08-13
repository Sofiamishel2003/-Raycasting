use rodio::{OutputStream, Sink, Decoder};
use std::fs::File;
use std::io::BufReader;
use std::thread;

pub fn play_background_music(file_path: &'static str) {
    let file_path_clone = file_path.to_string(); // Clonamos el path
    thread::spawn(move || {
        let (_stream, handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&handle).unwrap();
        let file = BufReader::new(File::open(file_path_clone).unwrap());
        let source = Decoder::new(file).unwrap();
        sink.append(source);
        sink.sleep_until_end();
    });
}

pub fn play_victory_sound(file_path: &str) {
    let (_stream, handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&handle).unwrap();
    let file = BufReader::new(File::open(file_path).unwrap());
    let source = Decoder::new(file).unwrap();
    sink.append(source);
    thread::spawn(move || {
        sink.sleep_until_end();
    });
}

