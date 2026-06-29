mod maze;
mod levels;
mod entities;
mod ghosts;
mod scoring;
mod scoreboard;
mod input;
mod render;
mod app;

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
