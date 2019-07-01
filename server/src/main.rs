use std::env;
use std::fs::File;
use rodio::{Source, Sink};
use std::error::Error;
use std::io::{self, Read, Cursor};

type Try<T> = Result<T, Box<dyn Error>>;

fn main() -> Try<()> {
    let path_arg = env::args().nth(1).ok_or("first argument: file to play")?;
    play(path_arg)?;
    Ok(())
}

fn play(path: String) -> Try<()> {
    let device = rodio::default_output_device().ok_or("no output device")?;
    let sink = Sink::new(&device);
    let buffer = load_file(path)?;
    let source = rodio::Decoder::new(Cursor::new(buffer))?;
    let duration_secs = source.total_duration().expect("unknown duration").as_secs();
    println!("track length: {}:{}", duration_secs / 60, duration_secs % 60);
    sink.append(source);
    sink.play();
    wait_for_enter()?;
    Ok(())
}

fn wait_for_enter() -> Try<()> {
    io::stdin().read_line(&mut String::new())?;
    Ok(())
}

fn load_file(path: String) -> Try<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}
