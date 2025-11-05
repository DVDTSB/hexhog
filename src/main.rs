mod byte;
mod config;
mod app;

use clap::Parser;
use color_eyre::Result;
use config::Config;
use app::{App, Args};

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let config_file_path = dirs::config_dir()
        .unwrap()
        .join("hexhog")
        .join("config.toml");

    let config = Config::read_config(config_file_path.to_str().unwrap()).unwrap_or_else(|e| {
        eprintln!("Error reading config: {e}");
        eprintln!("Using default config");
        Config::default()
    });

    let app = App::new(args, config)?;
    let terminal = ratatui::init();
    let result = app.run(terminal);
    ratatui::restore();
    result
}
