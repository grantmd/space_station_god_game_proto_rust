// https://github.com/ggez/ggez/blob/master/docs/FAQ.md#i-get-a-console-window-when-i-launch-my-executable-on-windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use ggez;
use glam;

use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::graphics::{Color, DrawParam, Text};
use ggez::{conf, graphics, timer, Context, ContextBuilder, GameResult};
use std::collections::HashMap;
use std::env;
use std::path;

// Alias some types to making reading/writing code easier and also in case math libraries change again
type Point2 = glam::Vec2;

// A Tile object, which the Station is made of
const TILE_WIDTH: f32 = 30.0;
#[derive(Debug)]
struct Tile {
    pos: Point2,    // x,y position of the tile within the station
    kind: TileType, // what type of square the tile is
}
#[derive(Debug)]
enum TileType {
    Floor,
    Wall,
    Door,
}

impl Tile {
    fn new(pos: Point2, kind: TileType) -> Tile {
        Tile {
            pos: pos,
            kind: kind,
        }
    }
}

// A type for the Station itself
#[derive(Debug)]
struct Station {
    pos: Point2, // The position of the station (upper-left, basically), in world coordinates
    tiles: HashMap<(i32, i32), Tile>, // All the Tiles that make up the station
    inhabitants: Vec<Inhabitant>, // All the Inhabitants in the station
}

impl Station {
    // Creates a new station from scratch.
    // Will eventually be randomly-generated
    fn new(pos: Point2, width: u32, height: u32) -> Station {
        let mut s = Station {
            pos: pos,
            tiles: HashMap::new(),
            inhabitants: Vec::new(),
        };

        for x in 0..width {
            for y in 0..height {
                // Figure out what type of tile
                let mut tile_type = TileType::Floor;
                if x == 0 || y == 0 {
                    tile_type = TileType::Wall;
                }
                if x == width - 1 || y == height - 1 {
                    tile_type = TileType::Wall;
                }

                // Place the tile
                let tile = Tile::new(Point2::new(x as f32, y as f32), tile_type);
                s.add_tile(tile);
            }
        }

        s
    }

    // Adds a tile to the station. Trusts the tile's position
    fn add_tile(&mut self, tile: Tile) {
        self.tiles
            .insert((tile.pos.x as i32, tile.pos.y as i32), tile);
    }

    // How many tiles do we have?
    fn num_tiles(&mut self) -> usize {
        self.tiles.len()
    }

    // Do we have a tile at a spot?
    fn has_tile(&mut self, pos: (i32, i32)) -> bool {
        self.tiles.contains_key(&pos)
    }

    // Get tile at a spot, if any
    fn get_tile(&mut self, pos: (i32, i32)) -> Option<&Tile> {
        self.tiles.get(&pos)
    }

    // Removes a tile
    fn remove_tile(&mut self, pos: (i32, i32)) {
        self.tiles.remove(&pos);
    }

    // Add an inhabitant
    fn add_inhabitant(&mut self, inhabitant: Inhabitant) {
        self.inhabitants.push(inhabitant)
    }
}

// An Inhabitant of the Station
#[derive(Debug)]
struct Inhabitant {
    pos: Point2,
    dest: Option<Point2>,
    kind: InhabitantType,
    health: i8,
    hunger: i8,
    thirst: i8,
}

#[derive(Debug)]
enum InhabitantType {
    Pilot,
    Engineer,
    Medic,
    Soldier,
    Miner,
    Ghost,
}

impl Inhabitant {
    fn new(pos: Point2, kind: InhabitantType) -> Inhabitant {
        Inhabitant {
            pos: pos,
            dest: None,
            kind: kind,
            health: 100,
            hunger: 0,
            thirst: 0,
        }
    }

    // Whether we can move to a type of tile
    // Doesn't check whether we can _get_ there, but only if we can be there
    fn can_move_to(&mut self, tile: Tile) -> bool {
        match self.kind {
            // Ghosts can go anywhere, lol
            InhabitantType::Ghost => true,

            // Everyone else needs to test the type of tile
            _ => match tile.kind {
                TileType::Wall => false,
                TileType::Door => true, // TODO: Check if we can open it?
                TileType::Floor => true,
            },
        }
    }
}

// Main game state object. Holds positions, scores, etc
struct SpaceStationGodGame {
    dt: std::time::Duration, // Time between updates
    is_fullscreen: bool,
    station: Station,
}

impl SpaceStationGodGame {
    // Load/create resources such as images here and otherwise initialize state
    pub fn new(ctx: &mut Context) -> GameResult<SpaceStationGodGame> {
        // Make a new station
        let (screen_width, screen_height) = graphics::drawable_size(ctx);

        let station_width = 15;
        let station_height = 11;

        let mut station_pos = Point2::new(screen_width / 2.0, screen_height / 2.0);
        station_pos -= Point2::new(
            station_width as f32 * TILE_WIDTH / 2.0,
            station_height as f32 * TILE_WIDTH / 2.0,
        );
        let mut station = Station::new(station_pos, station_width, station_height);

        // Put some people in it
        for _ in 0..1 {
            station.add_inhabitant(Inhabitant::new(
                Point2::new(station_width as f32 / 2.0, station_height as f32 / 2.0),
                InhabitantType::Engineer, // TODO: Random
            ));
        }

        // Create game state and return it
        let s = SpaceStationGodGame {
            dt: std::time::Duration::new(0, 0),
            is_fullscreen: false,
            station: station,
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
        self.dt = timer::delta(ctx);

        // Update at 60fps
        const DESIRED_FPS: u32 = 60;
        while timer::check_update_time(ctx, DESIRED_FPS) {
            let seconds = 1.0 / (DESIRED_FPS as f32);
            println!("{}: {:#?}", seconds, self.dt)
        }
        Ok(())
    }

    // Draw updates this loop
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Draw a black background
        graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());

        // TODO: Starfield

        // Draw the station
        // TODO: MeshBatch
        for (index, tile) in &self.station.tiles {
            let rect = graphics::Rect::new(
                self.station.pos.x + (TILE_WIDTH * index.0 as f32) - (TILE_WIDTH / 2.0),
                self.station.pos.y + (TILE_WIDTH * index.1 as f32) - (TILE_WIDTH / 2.0),
                TILE_WIDTH,
                TILE_WIDTH,
            );

            let mesh = match tile.kind {
                TileType::Floor => graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::stroke(1.0),
                    rect,
                    Color::new(0.3, 0.3, 0.3, 1.0),
                )?,
                TileType::Wall => graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::fill(),
                    rect,
                    Color::new(0.3, 0.3, 0.3, 1.0),
                )?,
                TileType::Door => graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::fill(),
                    rect,
                    graphics::WHITE,
                )?,
            };
            graphics::draw(ctx, &mesh, DrawParam::default())?;
        }

        // Draw the inhabitants
        for inhabitant in &self.station.inhabitants {
            let pos = Point2::new(
                self.station.pos.x + (TILE_WIDTH * inhabitant.pos.x) - (TILE_WIDTH / 2.0),
                self.station.pos.y + (TILE_WIDTH * inhabitant.pos.y) - (TILE_WIDTH / 2.0),
            );
            let mesh = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                pos,
                TILE_WIDTH / 2.0 - 5.0,
                0.1,
                graphics::WHITE,
            )?;
            graphics::draw(ctx, &mesh, DrawParam::default())?;
        }

        // Put our current FPS on top
        let fps = timer::fps(ctx);
        let fps_display = Text::new(format!("FPS: {0:.1}", fps));
        graphics::draw(ctx, &fps_display, (Point2::new(10.0, 0.0), graphics::WHITE))?;

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
