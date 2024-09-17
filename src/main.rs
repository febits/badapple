use badapple::badframes::{ANSICodes, Frames};
use badapple::bar::ProgressBar;
use badapple::cliargs::Cli;

use std::thread;
use std::time;

use clap::Parser;
use term_size;

fn main() {
    let cli = Cli::parse();

    let (width, height) = term_size::dimensions().unwrap();
    let mut frames_handler = Frames::new(width as u32, height as u32);

    frames_handler
        .extract_frames(&cli)
        .expect("Problem while extracting frames");

    let frame_duration = time::Duration::from_secs_f64(1.0 / cli.fps as f64);

    let mut bar = ProgressBar::new(100.0, width - 7, '-', '#');
    let update_rate = bar.pmax / frames_handler.max_frame_id as f64;

    frames_handler.ansi_action(&[ANSICodes::HideCursor]);

    for id in 1..=frames_handler.max_frame_id {
        let now = time::Instant::now();

        frames_handler
            .load_frame_by_id(id, &cli)
            .expect("Problem while loading frame");

        frames_handler.ansi_action(&[ANSICodes::MoveCursorToStart]);

        for line in &frames_handler.curr_frame {
            print!("{line}");
        }

        bar.update_bar(update_rate);
        println!("{bar}");

        let elapsed = now.elapsed();
        if elapsed < frame_duration {
            thread::sleep(frame_duration - elapsed);
        }
    }

    frames_handler.ansi_action(&[ANSICodes::ShowCursor, ANSICodes::ClearScreen]);
    println!("[-] Rendered {} frames", frames_handler.max_frame_id);
}
