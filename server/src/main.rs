use std::env;
use std::io;

fn main() -> Result<(), Box<dyn Error>> {
    let path_arg = env::args().nth(1).ok_or("first argument: file to play")?;
    play(path_arg)?;

    Ok(())
}

use std::fs::File;
use std::io::BufReader;
use rodio::Sink;
use std::error::Error;
use std::io::Read;
use std::io::Cursor;

fn play(path: String) -> Result<(), Box<dyn Error>> {
    let device = rodio::default_output_device().ok_or("no output device")?;
    let sink = Sink::new(&device);
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let source = rodio::Decoder::new(Cursor::new(buffer))?;
    // rodio::play_raw(&device, source.convert_samples());
    sink.append(source);
    sink.play();
    io::stdin().read_line(&mut String::new())?;
    Ok(())
}
