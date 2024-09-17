pub mod cliargs {
    use clap::Parser;

    #[derive(Parser)]
    #[command(version, about, long_about = None)]
    pub struct Cli {
        /// Path to a video
        pub video_path: String,

        /// Frames output dir
        pub frame_dir: String,

        /// Frames per second
        pub fps: u32,
    }
}

pub mod badframes {
    use super::cliargs::Cli;
    use image::ImageReader;
    use std::error::Error;
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    const ASCII_SHADING: &str = " .:-=+*#%@";
    const SCALE_FACTOR: u8 = std::u8::MAX / (ASCII_SHADING.len() as u8 - 1);

    pub struct Frames {
        pub curr_frame: Vec<String>,
        pub max_frame_id: usize,
        framew: u32,
        frameh: u32,
    }

    pub enum ANSICodes {
        MoveCursorToStart,
        HideCursor,
        ShowCursor,
        ClearScreen,
    }

    impl Frames {
        pub fn new(framew: u32, frameh: u32) -> Self {
            Self {
                curr_frame: Vec::new(),
                max_frame_id: 0,
                framew,
                frameh,
            }
        }

        pub fn extract_frames(&mut self, cli: &Cli) -> Result<(), Box<dyn Error>> {
            let output = Path::new(&cli.frame_dir);

            if !output.exists() {
                fs::create_dir(output)?;
                Command::new("ffmpeg")
                    .args([
                        "-i",
                        cli.video_path.as_str(),
                        "-vf",
                        format!("fps={}", cli.fps).as_str(),
                        format!("{}/frame%d.jpg", cli.frame_dir).as_str(),
                    ])
                    .status()?;
            }

            let mut frames_id = Vec::new();
            output.read_dir()?.for_each(|entry| {
                let fname = entry.unwrap().file_name().into_string().unwrap();
                frames_id.push(
                    fname
                        .split("frame")
                        .nth(1)
                        .unwrap()
                        .split(".")
                        .next()
                        .unwrap()
                        .parse::<usize>()
                        .unwrap(),
                );
            });

            println!(
                "[-] Frames from {} extracted to {}",
                cli.video_path, cli.frame_dir
            );

            self.max_frame_id = *frames_id.iter().max().unwrap();
            Ok(())
        }

        fn map_grayscale_to_ascii(&self, gray: u8) -> char {
            ASCII_SHADING
                .chars()
                .nth((gray / SCALE_FACTOR) as usize)
                .unwrap()
        }

        pub fn load_frame_by_id(&mut self, id: usize, cli: &Cli) -> Result<(), Box<dyn Error>> {
            self.curr_frame.clear();
            let luma = ImageReader::open(format!("{}/frame{}.jpg", cli.frame_dir, id))?
                .decode()?
                .resize_exact(self.framew, self.frameh, image::imageops::Nearest)
                .to_luma8();

            for (_, r) in luma.enumerate_rows() {
                let mut tmp = String::new();
                for (_, (_, _, gray)) in r.enumerate() {
                    tmp.push(self.map_grayscale_to_ascii(gray.0[0]));
                }
                tmp.push('\n');
                self.curr_frame.push(tmp);
            }

            Ok(())
        }

        pub fn ansi_action(&self, actions: &[ANSICodes]) {
            for action in actions {
                match action {
                    ANSICodes::HideCursor => print!("\x1b[?25l"),
                    ANSICodes::ShowCursor => print!("\x1b[?25h"),
                    ANSICodes::MoveCursorToStart => print!("\x1b[H"),
                    ANSICodes::ClearScreen => print!("\x1b[2J"),
                }
            }
        }
    }
}

pub mod bar {
    pub struct ProgressBar {
        progress: f64,
        fillc: char,
        emptyc: char,
        cmax: usize,
        pub pmax: f64,
    }

    impl ProgressBar {
        pub fn new(pmax: f64, cmax: usize, emptyc: char, fillc: char) -> Self {
            Self {
                pmax,
                cmax,
                emptyc,
                fillc,
                progress: 0.00,
            }
        }

        pub fn update_bar(&mut self, inc: f64) {
            if self.progress + inc >= self.pmax {
                self.progress = self.pmax;
                return;
            }

            self.progress += inc;
        }
    }

    impl std::fmt::Display for ProgressBar {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let fill_with = ((self.progress * self.cmax as f64) / self.pmax) as usize;
            let empty_with = self.cmax - fill_with;

            let mut tmp = String::new();
            for _ in 0..fill_with {
                tmp.push(self.fillc);
            }
            for _ in 0..empty_with {
                tmp.push(self.emptyc);
            }

            write!(f, "[{}] {}%", tmp, self.progress as usize)
        }
    }
}
