use ggez::audio::{self, SoundSource};
use ggez::{filesystem, Context, GameResult};

use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rand::seq::SliceRandom;

pub struct Music {
    sound: Option<audio::Source>,
    paths: Vec<PathBuf>,
}

impl Music {
    pub fn new(ctx: &mut Context) -> Music {
        let dir_contents: Vec<PathBuf> = filesystem::read_dir(ctx, "/music").unwrap().collect();

        Music {
            sound: None,
            paths: dir_contents,
        }
    }

    pub fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        match &mut self.sound {
            Some(sound) => {
                if sound.stopped() {
                    let path = choose_music(&self.paths);
                    println!("Choosing new music: {}", path.display());
                    let mut sound = audio::Source::new(ctx, path).unwrap();
                    sound.set_fade_in(Duration::from_millis(1000));
                    sound.play(ctx)?;

                    self.sound = Some(sound);
                }
            }
            None => {
                let path = choose_music(&self.paths);
                println!("Starting music: {}", path.display());
                let mut sound = audio::Source::new(ctx, path).unwrap();
                sound.set_fade_in(Duration::from_millis(1000));
                sound.play(ctx)?;

                self.sound = Some(sound);
            }
        }

        Ok(())
    }
}

fn choose_music(paths: &Vec<PathBuf>) -> &Path {
    let mut rng = rand::thread_rng();
    paths.choose(&mut rng).unwrap()
}

impl fmt::Display for Music {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.sound {
            Some(sound) => {
                write!(f, "{0:.1}s", sound.elapsed().as_secs_f32())
            }
            None => {
                write!(f, "None")
            }
        }
    }
}
