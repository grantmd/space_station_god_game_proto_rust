// https://github.com/ggez/ggez/blob/master/docs/FAQ.md#i-get-a-console-window-when-i-launch-my-executable-on-windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use ggez::event::{self, EventHandler};
use ggez::{graphics, timer, Context, ContextBuilder, GameResult};

fn main() {
    // Make a Context. This is passed to the game loop
    let (mut ctx, mut event_loop) = ContextBuilder::new("space_station_god_game", "Myles Grant")
        .build()
        .expect("aieee, could not create ggez context!");

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let mut my_game = SpaceStationGodGame::new(&mut ctx);

    // Run!
    match event::run(&mut ctx, &mut event_loop, &mut my_game) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e),
    }
}

// Main game state object. Holds positions, scores, etc
struct SpaceStationGodGame {
    dt: std::time::Duration,
}

impl SpaceStationGodGame {
    pub fn new(_ctx: &mut Context) -> SpaceStationGodGame {
        // Load/create resources such as images here and otherwise initialize state
        SpaceStationGodGame {
            dt: std::time::Duration::new(0, 0),
        }
    }
}

// Main event loop
impl EventHandler for SpaceStationGodGame {
    // Update game state.
    // `self` is state, `ctx` provides access to hardware (input, graphics, sound, etc)
    // Returns GameResult so ggez can handle any errors
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.dt = timer::delta(ctx);
        Ok(())
    }

    // Draw updates this loop
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        println!("Hello ggez! dt = {}ns", self.dt.subsec_nanos());
        graphics::clear(ctx, graphics::WHITE);
        graphics::present(ctx)
    }
}
