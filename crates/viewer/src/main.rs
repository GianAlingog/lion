use arena::{
    Cli, GameMode,
    solo::{SoloConfig, run_solo},
    versus::{VersusConfig, run_versus},
};

use crate::tui::{SoloTui, VersusTui};

pub mod render;
pub mod tui;

fn main() {
    let cli = Cli::parse_args();

    match cli.game_mode {
        GameMode::Solo(args) => {
            let cfg = SoloConfig::new(&cli.game_args, &args);
            let mut tui = SoloTui::new(ratatui::init(), cli.game_args.step_mode);
            let session_stats = run_solo(&cfg, &mut tui);
            ratatui::restore();
            println!("{session_stats}");
        }
        GameMode::Versus(args) => {
            let cfg = VersusConfig::new(&cli.game_args, &args);
            let mut tui = VersusTui::new(ratatui::init(), cli.game_args.step_mode);
            let session_stats = run_versus(&cfg, &mut tui);
            ratatui::restore();
            println!("{session_stats}");
        }
    }
}
