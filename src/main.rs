// https://github.com/ggez/ggez/blob/master/docs/FAQ.md#i-get-a-console-window-when-i-launch-my-executable-on-windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod inhabitant;
mod item;
mod music;
mod starfield;
mod station;

use inhabitant::{Inhabitant, InhabitantType};
use music::Music;
use starfield::Starfield;
use station::{GridPosition, Station, TileType};

use chrono::{DateTime, Local};
use oorandom::Rand32;
use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};

use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::graphics::{Color, DrawMode, DrawParam, Font, PxScale, Text, TextFragment};
use ggez::input::mouse;
use ggez::{conf, filesystem, graphics, timer, Context, ContextBuilder, GameResult};

use std::io::{Read, Write};
use std::{env, path};

// Alias some types to making reading/writing code easier and also in case math libraries change again
type Point2 = glam::Vec2;

const TILE_WIDTH: f32 = 30.0;

// Main game state object. Holds positions, scores, etc
struct SpaceStationGodGame {
    rng: oorandom::Rand32,
    is_fullscreen: bool,
    is_paused: bool,
    camera: Camera,
    starfield: Starfield,
    station: Station,
    inhabitants: Vec<Inhabitant>,
    music: Music,
}

#[derive(Serialize, Deserialize)]
pub struct Camera {
    pos: Point2,
    zoom: Point2,
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

        let station_width = 21;
        let station_height = 13;

        let mut station_pos = Point2::new(screen_width / 2.0, screen_height / 2.0);
        station_pos -= Point2::new(
            station_width as f32 * TILE_WIDTH / 2.0,
            station_height as f32 * TILE_WIDTH / 2.0,
        );
        let station = Station::new(ctx, station_pos, station_width, station_height, &mut rng);

        // Create game state and return it
        let mut game = SpaceStationGodGame {
            rng,
            is_fullscreen: false, // TODO: Is it possible to know this on startup from context?
            is_paused: true,
            camera: Camera {
                pos: Point2::zero(),
                zoom: Point2::one(),
            },
            starfield: Starfield::new(ctx, &mut rng),
            station,
            inhabitants: Vec::with_capacity(1),
            music: Music::new(ctx),
        };

        // Do we have any saved games?
        let saves = game.list_saves(ctx)?;
        println!("Saves: {:#?}", saves);

        // Put some people in it
        let tile = game
            .station
            .get_random_tile(TileType::Floor, &mut game.rng)
            .unwrap();
        let pos = tile.to_world_position(&game.station);
        let num_crew = 3;
        for _ in 0..num_crew {
            // TODO: Don't repeat
            // TODO: Got to be a better way to do this
            let inhabitant_type = match rng.rand_range(0..6) {
                0 => InhabitantType::Pilot,
                1 => InhabitantType::Engineer,
                2 => InhabitantType::Scientist,
                3 => InhabitantType::Medic,
                4 => InhabitantType::Soldier,
                5 => InhabitantType::Miner,
                6 => InhabitantType::Cook,
                _ => panic!("Invalid inhabitant type chosen"),
            };
            game.add_inhabitant(pos, inhabitant_type);
        }

        // Return the initial game state
        Ok(game)
    }

    // Add an inhabitant to the game
    fn add_inhabitant(&mut self, pos: Point2, kind: InhabitantType) {
        println!("Putting {:?} inhabitant at {}", kind, pos);
        self.inhabitants.push(Inhabitant::new(pos, kind));
    }

    // Save the game state to a file, overwriting if it exists
    fn save(&self, ctx: &mut Context, name: String) -> GameResult<()> {
        // Make sure the directory exists
        filesystem::create_dir(ctx, path::Path::new("/saves")).unwrap();

        // Write the game state out
        let filename = format!("/saves/{}.cbor", name);
        println!("Saving game to {}", filename);
        let test_file = path::Path::new(&filename);
        let file = filesystem::create(ctx, test_file)?;
        serde_cbor::to_writer(file, &self).unwrap();

        // Guess it worked
        Ok(())
    }

    // Load the game state from a file
    fn load(&self, ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    // List saved games
    fn list_saves(&self, ctx: &mut Context) -> GameResult<Vec<path::PathBuf>> {
        let dir_contents: Vec<path::PathBuf> = filesystem::read_dir(ctx, "/saves")?.collect();
        Ok(dir_contents)
    }
}

// Main event loop
impl EventHandler for SpaceStationGodGame {
    // Update game state.
    // `self` is state, `ctx` provides access to hardware (input, graphics, sound, etc)
    // Returns GameResult so ggez can handle any errors
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Check music
        self.music.update(ctx)?;

        // Update at 60fps
        const DESIRED_FPS: u32 = 60;
        while timer::check_update_time(ctx, DESIRED_FPS) {
            // Always update the starfield
            self.starfield.update(ctx)?;

            // Are we paused?
            if self.is_paused {
                continue;
            }

            // Update the station
            self.station.update(ctx)?;

            // Update and move the inhabitants
            for inhabitant in &mut self.inhabitants {
                inhabitant.update(ctx, &self.station, &mut self.rng)?;
            }
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
        self.station.draw(ctx, &self.camera)?;

        // Draw the inhabitants
        for inhabitant in &mut self.inhabitants {
            inhabitant.draw(ctx, &self.camera)?;
        }

        // Draw where the mouse is
        let mut mouse_pos = mouse::position(ctx);
        let mut mouse_display = Text::new(format!("Mouse: ({}, {})", mouse_pos.x, mouse_pos.y));
        if let Some(selected_tile) = self
            .station
            .get_tile_from_screen(Point2::new(mouse_pos.x, mouse_pos.y), &self.camera)
        {
            let world_pos = selected_tile.to_world_position(&self.station);
            mouse_display.add(format!(
                "\nTile: Grid ({}, {}), World ({},{}), {:?}",
                selected_tile.pos.x,
                selected_tile.pos.y,
                world_pos.x,
                world_pos.y,
                selected_tile.kind
            ));

            if selected_tile.items.len() > 0 {
                mouse_display.add(format!("\n{:?}", selected_tile.items));
            }

            let tile_rect = graphics::Rect::new(
                (crate::TILE_WIDTH * selected_tile.pos.x as f32) - (crate::TILE_WIDTH / 2.0),
                (crate::TILE_WIDTH * selected_tile.pos.y as f32) - (crate::TILE_WIDTH / 2.0),
                crate::TILE_WIDTH,
                crate::TILE_WIDTH,
            );
            let mesh = graphics::Mesh::new_rectangle(
                ctx,
                DrawMode::stroke(1.0),
                tile_rect,
                Color::new(1.0, 1.0, 0.0, 1.0),
            )?;
            graphics::draw(ctx, &mesh, DrawParam::default().dest(self.station.pos))?;
        }
        mouse_pos.y -= mouse_display.height(ctx);
        graphics::queue_text(ctx, &mouse_display, mouse_pos, Some(Color::WHITE));

        // If paused, grey out the screen and show that that's the case
        if self.is_paused {
            let (screen_width, screen_height) = graphics::drawable_size(ctx);
            let screen_rect = graphics::Rect::new(0.0, 0.0, screen_width, screen_height);
            let mesh = graphics::Mesh::new_rectangle(
                ctx,
                DrawMode::fill(),
                screen_rect,
                Color::new(1.0, 1.0, 1.0, 0.1),
            )?;
            graphics::draw(ctx, &mesh, DrawParam::default())?;

            let paused_font = Font::new(ctx, "/fonts/Moonhouse-yE5M.ttf")?;
            let paused_display = Text::new(
                TextFragment::new("PAUSED")
                    .font(paused_font)
                    .scale(PxScale::from(100.0)),
            );
            let dims = paused_display.dimensions(ctx);
            graphics::queue_text(
                ctx,
                &paused_display,
                Point2::new(
                    screen_width / 2.0 - dims.w / 2.0,
                    screen_height / 2.0 - dims.h / 2.0,
                ),
                Some(Color::WHITE),
            );
        }

        // Put our current FPS on top along with other info
        let fps = timer::fps(ctx);
        let mut height = 0.0;
        let fps_display = Text::new(format!("FPS: {0:.1}", fps));
        graphics::queue_text(
            ctx,
            &fps_display,
            Point2::new(10.0, 0.0 + height),
            Some(Color::WHITE),
        );
        height += 5.0 + fps_display.height(ctx) as f32;
        let uptime_display = Text::new(format!("Uptime: {:?}", timer::time_since_start(ctx)));
        graphics::queue_text(
            ctx,
            &uptime_display,
            Point2::new(10.0, 0.0 + height),
            Some(Color::WHITE),
        );
        height += 5.0 + uptime_display.height(ctx) as f32;
        let station_display = Text::new(format!(
            "Station Tiles: {} at {}, Selected: None",
            self.station.num_tiles(),
            self.station.pos
        ));
        graphics::queue_text(
            ctx,
            &station_display,
            Point2::new(10.0, 0.0 + height),
            Some(Color::WHITE),
        );
        height += 5.0 + station_display.height(ctx) as f32;
        let inhabitant_display = Text::new(format!("Inhabitants: {}", self.inhabitants.len()));
        graphics::queue_text(
            ctx,
            &inhabitant_display,
            Point2::new(10.0, 0.0 + height),
            Some(Color::WHITE),
        );
        height += 5.0 + inhabitant_display.height(ctx) as f32;
        let camera_display = Text::new(format!(
            "Camera: {} ({1:.1}x)",
            self.camera.pos, self.camera.zoom.x
        ));
        graphics::queue_text(
            ctx,
            &camera_display,
            Point2::new(10.0, 0.0 + height),
            Some(Color::WHITE),
        );
        height += 5.0 + camera_display.height(ctx) as f32;
        let music_display = Text::new(format!("Music: {}", self.music));
        graphics::queue_text(
            ctx,
            &music_display,
            Point2::new(10.0, 0.0 + height),
            Some(Color::WHITE),
        );

        // Render all queued text
        graphics::draw_queued_text(
            ctx,
            DrawParam::default(),
            None,
            graphics::FilterMode::Linear,
        )?;

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

            // Toggle paused
            KeyCode::Space => {
                self.is_paused = !self.is_paused;
                if self.is_paused {
                    println!("Pausing");
                } else {
                    println!("Unpausing");
                }
            }

            // Add a new inhabitant
            KeyCode::N => {
                if self.is_paused {
                    return;
                }

                let tile = self
                    .station
                    .get_random_tile(TileType::Floor, &mut self.rng)
                    .unwrap();
                let pos = tile.to_world_position(&self.station);
                self.add_inhabitant(
                    pos,
                    InhabitantType::Engineer, // TODO: Random
                );
            }

            // Camera movement from arrow keys
            KeyCode::Up => {
                self.camera.pos += Point2::unit_y() * 10.0;
            }
            KeyCode::Down => {
                self.camera.pos -= Point2::unit_y() * 10.0;
            }
            KeyCode::Left => {
                self.camera.pos += Point2::unit_x() * 10.0;
            }
            KeyCode::Right => {
                self.camera.pos -= Point2::unit_x() * 10.0;
            }
            KeyCode::C => {
                self.camera.pos = Point2::zero();
                self.camera.zoom = Point2::one();
            }

            // Save the game
            KeyCode::S => {
                let now: DateTime<Local> = Local::now();
                self.save(ctx, now.format("%Y-%m-%d %H-%M-%S.%f").to_string())
                    .unwrap();
            }

            // Everything else does nothing
            _ => (),
        }
    }

    // The mousewheel/trackpad was moved
    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, y: f32) {
        self.camera.zoom += Point2::one() * y * 2.0; // TODO: Tweak this multiple
        if self.camera.zoom < Point2::one() {
            self.camera.zoom = Point2::one();
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

impl Serialize for SpaceStationGodGame {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 4 is the number of fields from the struct we care about
        let mut state = serializer.serialize_struct("SpaceStationGodGame", 4)?;
        state.serialize_field("camera", &self.camera)?;
        state.serialize_field("station", &self.station)?;
        state.serialize_field("inhabitants", &self.inhabitants)?;
        state.serialize_field("rng", &self.rng.state())?;
        state.end()
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
        .window_setup(
            conf::WindowSetup::default()
                .title("Space Station God Game")
                .vsync(true),
        )
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
