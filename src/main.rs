// https://github.com/ggez/ggez/blob/master/docs/FAQ.md#i-get-a-console-window-when-i-launch-my-executable-on-windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod inhabitant;
mod item;
mod music;
mod scenes;
mod starfield;
mod station;

use crate::scenes::*;
use crate::station::gridposition::*;
use music::Music;
use starfield::Starfield;

use serde::{Deserialize, Serialize};

use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::{conf, graphics, timer, Context, ContextBuilder, GameResult};

use std::{env, path};

// Alias some types to making reading/writing code easier and also in case math libraries change again
type Point2 = glam::Vec2;

const TILE_WIDTH: f32 = 30.0;

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Camera {
    pos: Point2,
    zoom: Point2,
}

// Main game state object. Holds positions, scores, etc
struct GameState {
    is_fullscreen: bool,
    starfield: Starfield,
    music: Music,
    scenes: Vec<Box<dyn scene::Scene>>,
}

impl GameState {
    // Load/create resources such as images here and otherwise initialize state
    pub fn new(ctx: &mut Context) -> GameResult<GameState> {
        // Create game state and return it
        let mut state = GameState {
            is_fullscreen: false, // TODO: Is it possible to know this on startup from context?
            starfield: Starfield::new(ctx),
            music: Music::new(ctx),
            scenes: Vec::with_capacity(5),
        };

        // Add the initial title scene
        state.push_scene(Box::new(scenes::title::Title {}));

        // Return the initial game state
        Ok(state)
    }

    // Return the currently-active scene
    pub fn get_current_scene(&mut self) -> Option<&mut Box<dyn scene::Scene>> {
        self.scenes.last_mut()
    }

    // Push a new scene, unless it's the currently-active one
    pub fn push_scene(&mut self, scene: Box<dyn scene::Scene>) {
        match self.scenes.last() {
            Some(current_scene) => {
                if current_scene.get_type() != scene.get_type() {
                    self.scenes.push(scene);
                }
            }
            None => {
                self.scenes.push(scene);
            }
        }
    }

    // Pop the top of the scene stack
    pub fn pop_scene(&mut self) {
        self.scenes.pop();

        // Push the default state back on
        if self.scenes.len() == 0 {
            self.push_scene(Box::new(scenes::title::Title {}));
        }
    }
}

// Main event loop
impl EventHandler for GameState {
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

            // Update all scenes
            for scene in self.scenes.iter_mut() {
                scene.update(ctx)?;
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

        // Draw all scenes
        for scene in self.scenes.iter() {
            scene.draw(ctx)?;
        }

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
        keymods: KeyMods,
        repeat: bool,
    ) {
        // Global overrides/shortcuts
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

            // Everything else does nothing
            _ => (),
        }

        // Inform current scene
        if let Some(scene) = self.get_current_scene() {
            scene.key_down_event(ctx, keycode, keymods, repeat);
        }
    }

    // The mousewheel/trackpad was moved
    fn mouse_wheel_event(&mut self, ctx: &mut Context, x: f32, y: f32) {
        // Inform current scene
        if let Some(scene) = self.get_current_scene() {
            scene.mouse_wheel_event(ctx, x, y);
        }
    }

    // The window was resized
    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        let new_rect = graphics::Rect::new(0.0, 0.0, width, height);
        graphics::set_screen_coordinates(ctx, new_rect).unwrap();
        self.starfield.resize_event(ctx, width, height);
        println!("Resized screen to {}, {}", width, height);

        // Inform current scene
        if let Some(scene) = self.get_current_scene() {
            scene.resize_event(ctx, width, height);
        }
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
    let state = GameState::new(&mut ctx)?;

    // Run!
    event::run(ctx, event_loop, state)
}
