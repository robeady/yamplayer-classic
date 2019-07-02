use std::env;
use std::error::Error;
use std::io;

mod server;

type Try<T> = std::result::Result<T, Box<dyn Error>>;

fn main_cmd() -> Try<()> {
    let path_arg = env::args().nth(1).ok_or("first argument: file to play")?;
    let mut app = server::PlayerApp::new()?;
    app.play_file(&path_arg)?;
    wait_for_enter()?;
    Ok(())
}

fn main() -> Try<()> {
    env_logger::init();
    server::run_server()
}

fn wait_for_enter() -> Try<()> {
    io::stdin().read_line(&mut String::new())?;
    Ok(())
}
