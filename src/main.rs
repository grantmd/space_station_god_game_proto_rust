// https://github.com/ggez/ggez/blob/master/docs/FAQ.md#i-get-a-console-window-when-i-launch-my-executable-on-windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::mint;
use ggez::nalgebra as na;
use ggez::{graphics, timer, Context, ContextBuilder, GameResult};
use std::env;
use std::path;

// Main game state object. Holds positions, scores, etc
struct SpaceStationGodGame {
    dt: std::time::Duration, // Time between updates
    circle_pos_x: f32,       // Position of a circle on the screen
    frames: usize,           // Total number of frames drawn
    text: graphics::Text,    // Some text to draw on the screen
}

impl SpaceStationGodGame {
    pub fn new(ctx: &mut Context) -> GameResult<SpaceStationGodGame> {
        // The ttf file will be in your resources directory. Later, we
        // will mount that directory so we can omit it in the path here.
        let font = graphics::Font::new(ctx, "/DejaVuSerif.ttf")?;
        let text = graphics::Text::new(("Hello world!", font, 48.0));

        // Load/create resources such as images here and otherwise initialize state
        let s = SpaceStationGodGame {
            dt: std::time::Duration::new(0, 0),
            circle_pos_x: 0.0,
            frames: 0,
            text: text,
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
        self.circle_pos_x = self.circle_pos_x % 800.0 + 1.0; // Move the circle to the right and wrap around when the end is reached
        Ok(())
    }

    // Draw updates this loop
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Draw a blue background
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        // Create a white filled circle
        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            na::Point2::new(0.0, 0.0),
            100.0,
            2.0,
            graphics::WHITE,
        )?;
        // Draw the circle at the position
        graphics::draw(ctx, &circle, (na::Point2::new(self.circle_pos_x, 380.0),))?;

        // Drawables are drawn from their top-left corner.
        // Draw some text moving from top-left to bottom-right
        let offset = self.frames as f32 / 10.0;
        let dest_point = mint::Point2 {
            x: (offset),
            y: (offset),
        };
        graphics::draw(ctx, &self.text, (dest_point,))?;

        // Actually draw everything to the screen
        graphics::present(ctx)?;

        // Increment the number of frames drawn
        self.frames += 1;
        if (self.frames % 100) == 0 {
            // Every 100 frames print some stats to the console
            println!(
                "FPS: {}, dt: {}ns",
                ggez::timer::fps(ctx),
                self.dt.subsec_nanos()
            );
        }

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
        if keycode == KeyCode::Escape || keycode == KeyCode::Q {
            event::quit(ctx);
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
    let (mut ctx, mut event_loop) = ContextBuilder::new("space_station_god_game", "Myles Grant")
        .add_resource_path(resource_dir)
        .build()
        .expect("aieee, could not create ggez context!");

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let mut state = SpaceStationGodGame::new(&mut ctx)?;

    // Run!
    event::run(&mut ctx, &mut event_loop, &mut state)
}
