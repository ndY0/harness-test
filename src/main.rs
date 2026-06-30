mod app;
mod entities;
mod ghosts;
mod input;
mod levels;
mod maze;
mod render;
mod scoreboard;
mod scoring;

fn main() {
    match app::run_app() {
        Ok(()) => {
            std::process::exit(0);
        }
        Err(e) => {
            if e == "terminal_size" {
                std::process::exit(1);
            }
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
