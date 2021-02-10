// https://github.com/ggez/ggez/blob/master/docs/FAQ.md#i-get-a-console-window-when-i-launch-my-executable-on-windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod inhabitant;
mod starfield;
mod station;

use inhabitant::{Inhabitant, InhabitantType};
use starfield::Starfield;
use station::{Station, Tile, TileType};

use ggez;
use glam;
use oorandom::Rand32;

use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::graphics::{Color, DrawParam, Text};
use ggez::{conf, graphics, timer, Context, ContextBuilder, GameResult};

use keyframe::{ease, functions::EaseInOut};

use std::env;
use std::path;

// Alias some types to making reading/writing code easier and also in case math libraries change again
type Point2 = glam::Vec2;

const TILE_WIDTH: f32 = 30.0;

// Main game state object. Holds positions, scores, etc
struct SpaceStationGodGame {
    dt: std::time::Duration, // Time between updates
    rng: oorandom::Rand32,
    is_fullscreen: bool,
    starfield: Starfield,
    station: Station,
    inhabitants: Vec<Inhabitant>,
}

impl SpaceStationGodGame {
    // Load/create resources such as images here and otherwise initialize state
    pub fn new(ctx: &mut Context) -> GameResult<SpaceStationGodGame> {
        // Create a seeded random-number generator
        let mut seed: [u8; 8] = [0; 8];
        getrandom::getrandom(&mut seed[..]).expect("Could not create RNG seed");
        let mut rng = Rand32::new(u64::from_ne_bytes(seed));

        // Make a new station
        let (screen_width, screen_height) = graphics::drawable_size(ctx);

        let station_width = 15;
        let station_height = 11;

        let mut station_pos = Point2::new(screen_width / 2.0, screen_height / 2.0);
        station_pos -= Point2::new(
            station_width as f32 * TILE_WIDTH / 2.0,
            station_height as f32 * TILE_WIDTH / 2.0,
        );
        let station = Station::new(station_pos, station_width, station_height);

        // Put some people in it
        let mut inhabitants = Vec::new();
        for _ in 0..1 {
            inhabitants.push(Inhabitant::new(
                Point2::new(station_width as f32 / 2.0, station_height as f32 / 2.0),
                InhabitantType::Engineer, // TODO: Random
            ));
        }

        // Create game state and return it
        let s = SpaceStationGodGame {
            dt: std::time::Duration::new(0, 0),
            rng: rng,
            is_fullscreen: false,
            starfield: Starfield::new(ctx, &mut rng),
            station: station,
            inhabitants: inhabitants,
        };

        Ok(s)
    }
}

// Main event loop
impl EventHandler for SpaceStationGodGame {
    // Update game state.
    // `self` is state, `ctx` provides access to hardware (input, graphics, sound, etc)
    // Returns GameResult so ggez can handle any errors
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.dt += timer::delta(ctx);

        // Update at 60fps
        const DESIRED_FPS: u32 = 60;
        while timer::check_update_time(ctx, DESIRED_FPS) {
            // Move the inhabitants
            for inhabitant in &mut self.inhabitants {
                match inhabitant.dest {
                    Some(dest) => {
                        // Keep going until we get there
                        //let pos = ease(EaseInOut, inhabitant.pos, inhabitant.dest.unwrap(), self.dt.as_secs_f64());
                        println!("Continuing from {} to {}", inhabitant.pos, dest);
                        inhabitant.pos = dest;
                        inhabitant.dest = None;
                    }
                    None => {
                        // Only move once per second
                        if self.dt.as_secs() >= 1 {
                            // Pick a random valid destination
                            loop {
                                let x = self.rng.rand_range(0..3) as i32 - 1;
                                let y = self.rng.rand_range(0..3) as i32 - 1;
                                let tile = self.station.get_tile((
                                    inhabitant.pos.x as i32 + x,
                                    inhabitant.pos.y as i32 + y,
                                ));
                                if inhabitant.can_move_to(tile) {
                                    let dest = Point2::new(
                                        inhabitant.pos.x + x as f32,
                                        inhabitant.pos.y + y as f32,
                                    );
                                    if dest != inhabitant.pos {
                                        println!("Moving to {}", dest);
                                        inhabitant.dest = Some(dest);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Only count the seconds
        if self.dt.as_secs() >= 1 {
            self.dt -= std::time::Duration::new(1, 0);
        }

        // Done processing
        Ok(())
    }

    // Draw updates this loop
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Draw a black background
        graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());

        // Starfield
        self.starfield.draw(ctx)?;

        // Draw the station
        // TODO: MeshBatch
        self.station.draw(ctx)?;

        // Draw the inhabitants
        for inhabitant in &mut self.inhabitants {
            inhabitant.draw(ctx, self.station.pos)?;
        }

        // Put our current FPS on top
        let fps = timer::fps(ctx);
        let fps_display = Text::new(format!("FPS: {0:.1}", fps));
        graphics::draw(ctx, &fps_display, (Point2::new(10.0, 0.0), Color::WHITE))?;

        // Actually draw everything to the screen
        graphics::present(ctx)?;

        // We yield the current thread until the next update
        ggez::timer::yield_now();

        Ok(())
    }

    // Handle keypresses
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            // Quit
            KeyCode::Escape | KeyCode::Q => {
                event::quit(ctx);
            }
            // Toggle fullscreen
            KeyCode::F10 => {
                self.is_fullscreen = !self.is_fullscreen;

                let fullscreen_type = if self.is_fullscreen {
                    println!("Switching to fullscreen");
                    conf::FullscreenType::Desktop
                } else {
                    println!("Switching to windowed");
                    conf::FullscreenType::Windowed
                };

                graphics::set_fullscreen(ctx, fullscreen_type).unwrap();
            }
            _ => (), // Do nothing
        }
    }

    // The window was resized
    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        let new_rect = graphics::Rect::new(0.0, 0.0, width, height);
        graphics::set_screen_coordinates(ctx, new_rect).unwrap();
        self.starfield
            .resize_event(ctx, &mut self.rng, width, height);
        println!("Resized screen to {}, {}", width, height);
    }
}

// Entrypoint
fn main() -> GameResult {
    // We add the CARGO_MANIFEST_DIR/resources to the resource paths
    // so that ggez will look in our cargo project directory for files.
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    // Make a Context. This is passed to the game loop
    let (mut ctx, event_loop) = ContextBuilder::new("space_station_god_game", "Myles Grant")
        .add_resource_path(resource_dir)
        .window_setup(conf::WindowSetup::default().title("Space Station God Game"))
        .window_mode(conf::WindowMode::default().dimensions(1280.0, 720.0))
        .build()?;
    println!("{}", graphics::renderer_info(&ctx)?);
    println!("Game resource path: {:#?}", ctx.filesystem);

    println!("{:#?}", graphics::drawable_size(&ctx));

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let state = SpaceStationGodGame::new(&mut ctx)?;

    // Run!
    event::run(ctx, event_loop, state)
}
