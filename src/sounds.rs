use rodio::{OutputStream, Sink, Decoder};
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::thread;
use once_cell::sync::Lazy;

// Variable global para el Sink, lo cual nos permite detener la música.
static CURRENT_SINK: Lazy<Arc<Mutex<Option<Arc<Sink>>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

pub fn play_background_music(file_path: &'static str) {
    let file_path_clone = file_path.to_string(); // Clonamos el path
    thread::spawn(move || {
        let (_stream, handle) = OutputStream::try_default().unwrap();
        let sink = Arc::new(Sink::try_new(&handle).unwrap()); // Crear Arc de Sink
        let file = BufReader::new(File::open(file_path_clone).unwrap());
        let source = Decoder::new(file).unwrap();
        
        // Guardamos la referencia del Sink en la variable global
        {
            let mut current_sink = CURRENT_SINK.lock().unwrap();
            *current_sink = Some(Arc::clone(&sink)); // Usamos Arc::clone
        }

        sink.append(source);
        sink.sleep_until_end();
    });
}

pub fn stop_music() {
    let mut current_sink = CURRENT_SINK.lock().unwrap();
    if let Some(sink) = current_sink.take() {
        sink.stop(); // Detenemos la música
    }
}

pub fn play_sound_effect(file_path: &str) {
    let file_path_clone = file_path.to_string(); // Clonar el path para evitar problemas de lifetime
    thread::spawn(move || {
        let (_stream, handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&handle).unwrap();
        let file = BufReader::new(File::open(file_path_clone).unwrap());
        let source = Decoder::new(file).unwrap();
        sink.append(source);
        sink.sleep_until_end();
    });
}

pub fn play_victory_sound() {
    play_sound_effect("src/assets/music/Victory_Music.mp3");
}

pub fn play_screamer_sound() {
    play_sound_effect("src/assets/music/screamer.mp3");
}
