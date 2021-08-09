use super::paused::*;
use super::scene::*;
use super::quit::*;
use crate::inhabitant::{Inhabitant, InhabitantType};
use crate::station::station::*;
use crate::station::tile::*;
use crate::Camera;

use ggez::event::{KeyCode, KeyMods};
use ggez::graphics::{Color, DrawMode, DrawParam, Text};
use ggez::input::mouse;
use ggez::{filesystem, graphics, timer, Context, GameResult};

use chrono::{DateTime, Local};
use oorandom::Rand32;
use serde::{Deserialize, Serialize};

use std::path;

type Point2 = glam::Vec2;

pub struct Game {
    rng: oorandom::Rand32,
    is_paused: bool,
    camera: Camera,
    station: Station,
    inhabitants: Vec<Inhabitant>,

    show_stats: bool,
}

impl Game {
    pub fn new(ctx: &mut Context) -> Game {
        // Create a seeded random-number generator
        // TODO: Accept seed as input for deterministic replayability
        let mut seed: [u8; 8] = [0; 8];
        getrandom::getrandom(&mut seed[..]).expect("Could not create RNG seed");
        let mut rng = Rand32::new(u64::from_ne_bytes(seed));

        // Make a new station
        let (screen_width, screen_height) = graphics::drawable_size(ctx);

        let station_width = 21;
        let station_height = 13;

        let mut station_pos = Point2::new(screen_width / 2.0, screen_height / 2.0);
        station_pos -= Point2::new(
            station_width as f32 * crate::TILE_WIDTH / 2.0,
            station_height as f32 * crate::TILE_WIDTH / 2.0,
        );
        let station = Station::new(ctx, station_pos, station_width, station_height, &mut rng);

        // Create game state and return it
        let mut game = Game {
            rng,
            is_paused: false,
            camera: Camera {
                pos: Point2::zero(),
                zoom: Point2::one(),
            },
            station,
            inhabitants: Vec::with_capacity(1),

            show_stats: true,
        };

        // Put some people in it
        let tile = game
            .station
            .get_random_tile(TileType::Floor, &mut game.rng)
            .unwrap();
        let pos = tile.to_world_position(&game.station);
        let num_crew = 3;
        for _ in 0..num_crew {
            // TODO: Don't repeat inhabitant types
            let inhabitant_type = game.get_random_inhabitant_type();
            game.add_inhabitant(pos, inhabitant_type);
        }

        // Do we have any saved games?
        let saves = game.list_saves(ctx).unwrap();
        println!("Saves: {:#?}", saves);

        // Return the initial game state
        game
    }

    // Add an inhabitant to the game
    fn add_inhabitant(&mut self, pos: Point2, kind: InhabitantType) {
        println!("Putting {:?} inhabitant at {}", kind, pos);
        self.inhabitants.push(Inhabitant::new(pos, kind));
    }

    fn get_random_inhabitant_type(&mut self) -> InhabitantType {
        // TODO: Got to be a better way to do this
        match self.rng.rand_range(0..6) {
            0 => InhabitantType::Pilot,
            1 => InhabitantType::Engineer,
            2 => InhabitantType::Scientist,
            3 => InhabitantType::Medic,
            4 => InhabitantType::Soldier,
            5 => InhabitantType::Miner,
            6 => InhabitantType::Cook,
            _ => panic!("Invalid inhabitant type chosen"),
        }
    }

    // Save the game state to a file, overwriting if it exists
    fn save(&self, ctx: &mut Context, name: String) -> GameResult<()> {
        // Make sure the directory exists
        filesystem::create_dir(ctx, path::Path::new("/saves")).unwrap();

        // Create the save game object
        let state = SavedGame {
            rng_state: self.rng.state(),
            camera: self.camera,
            inhabitants: self.inhabitants.clone(),
            station: self.station.clone(),
        };

        // Write the game state out
        let filename = format!("/saves/{}.cbor", name);
        println!("Saving game to {}", filename);
        let test_file = path::Path::new(&filename);
        let file = filesystem::create(ctx, test_file)?;
        serde_cbor::to_writer(file, &state).unwrap();

        // Guess it worked
        Ok(())
    }

    // Load the game state from a file
    fn load(&mut self, ctx: &mut Context, filename: &path::PathBuf) -> GameResult<()> {
        // Load the file
        let file = filesystem::open(ctx, path::Path::new(filename)).unwrap();
        let save: SavedGame = serde_cbor::from_reader(file).unwrap();

        // Copy the data over
        self.rng = oorandom::Rand32::from_state(save.rng_state);
        self.camera = save.camera;
        self.inhabitants = save.inhabitants;
        self.station = save.station;

        // Rebuild all the meshes
        self.station.build_mesh(ctx)?;

        // Guess it worked
        Ok(())
    }

    // List saved games
    fn list_saves(&self, ctx: &mut Context) -> GameResult<Vec<path::PathBuf>> {
        let dir_contents: Vec<path::PathBuf> = filesystem::read_dir(ctx, "/saves")?.collect();
        Ok(dir_contents)
    }
}

impl Scene for Game {
    fn get_type(&self) -> SceneType {
        SceneType::Game
    }

    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        // Draw the station
        self.station.draw(ctx, &self.camera)?;

        // Draw the inhabitants
        for inhabitant in &self.inhabitants {
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

        if self.show_stats {
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
            height += 5.0 + fps_display.height(ctx);
            let uptime_display = Text::new(format!("Uptime: {:?}", timer::time_since_start(ctx)));
            graphics::queue_text(
                ctx,
                &uptime_display,
                Point2::new(10.0, 0.0 + height),
                Some(Color::WHITE),
            );
            height += 5.0 + uptime_display.height(ctx);
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
            height += 5.0 + station_display.height(ctx);
            let inhabitant_display = Text::new(format!("Inhabitants: {}", self.inhabitants.len()));
            graphics::queue_text(
                ctx,
                &inhabitant_display,
                Point2::new(10.0, 0.0 + height),
                Some(Color::WHITE),
            );
            height += 5.0 + inhabitant_display.height(ctx);
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
            /*height += 5.0 + camera_display.height(ctx);
            let music_display = Text::new(format!("Music: {}", state.music));
            graphics::queue_text(
                ctx,
                &music_display,
                Point2::new(10.0, 0.0 + height),
                Some(Color::WHITE),
            );*/
        }

        // Render all queued text
        graphics::draw_queued_text(
            ctx,
            DrawParam::default(),
            None,
            graphics::FilterMode::Linear,
        )?;

        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Are we paused?
        if self.is_paused {
            return Ok(());
        }

        // Update the station
        self.station.update(ctx)?;

        // Update and move the inhabitants
        for inhabitant in &mut self.inhabitants {
            inhabitant.update(ctx, &self.station, &mut self.rng)?;
        }
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        repeat: bool,
    ) -> SceneAction {
        let mut action = SceneAction::None; // The action we will end up returning

        match keycode {
            // Quit
            KeyCode::Escape | KeyCode::Q if !repeat => {
                action = SceneAction::Push(Box::new(Quit {}))
            }

            // Toggle paused
            KeyCode::Space if !repeat => {
                println!("Pausing");
                action = SceneAction::Push(Box::new(Paused {}))
            }

            // Add a new inhabitant
            KeyCode::N if !repeat && !self.is_paused => {
                let tile = self
                    .station
                    .get_random_tile(TileType::Floor, &mut self.rng)
                    .unwrap();
                let pos = tile.to_world_position(&self.station);
                let inhabitant_type = self.get_random_inhabitant_type();
                self.add_inhabitant(pos, inhabitant_type);
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
            KeyCode::S if !repeat => {
                let now: DateTime<Local> = Local::now();
                self.save(ctx, now.format("%Y-%m-%d %H-%M-%S.%f").to_string())
                    .unwrap();
            }

            // Load a save
            KeyCode::L if !repeat => {
                let saves = self.list_saves(ctx).unwrap();
                if let Some(filename) = saves.last() {
                    self.load(ctx, &filename).unwrap();
                }
            }

            // Toggle stats
            KeyCode::F1 if !repeat => self.show_stats = !self.show_stats,

            // Everything else does nothing
            _ => (),
        }

        action
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, y: f32) -> SceneAction {
        self.camera.zoom += Point2::one() * y * 2.0; // TODO: Tweak this multiple
        if self.camera.zoom < Point2::one() {
            self.camera.zoom = Point2::one();
        }

        SceneAction::None
    }

    fn resize_event(&mut self, _ctx: &mut Context, _width: f32, _height: f32) -> SceneAction {
        SceneAction::None
    }

    fn from_scene(&mut self, kind: SceneType) {
        match kind {
            SceneType::Paused => self.is_paused = false,
            SceneType::Quit => self.is_paused = false,
            _ => (),
        }
    }

    fn to_scene(&mut self, kind: SceneType) {
        match kind {
            SceneType::Paused => self.is_paused = true,
            SceneType::Quit => self.is_paused = true,
            _ => (),
        }
    }
}

// Save game serialize/deserialize object
#[derive(Serialize, Deserialize)]
struct SavedGame {
    rng_state: (u64, u64),
    camera: Camera,
    station: Station,
    inhabitants: Vec<Inhabitant>,
}
