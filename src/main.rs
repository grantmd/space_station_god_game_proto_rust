// https://github.com/ggez/ggez/blob/master/docs/FAQ.md#i-get-a-console-window-when-i-launch-my-executable-on-windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::graphics::{Color, DrawMode, DrawParam};
use ggez::mint;
use ggez::nalgebra as na;
use ggez::{conf, graphics, timer, Context, ContextBuilder, GameResult};
use std::env;
use std::path;

// Main game state object. Holds positions, scores, etc
struct SpaceStationGodGame {
    dt: std::time::Duration, // Time between updates
    circle_pos_x: f32,       // Position of a circle on the screen
    frames: usize,           // Total number of frames drawn
    text: graphics::Text,    // Some text to draw on the screen
    dragon: graphics::Image, // Next three are images to draw
    shot_linear: graphics::Image,
    shot_nearest: graphics::Image,
    rotation: f32,               // Rotation of some of the images
    meshes: Vec<graphics::Mesh>, // Collection of meshes to draw
}

impl SpaceStationGodGame {
    pub fn new(ctx: &mut Context) -> GameResult<SpaceStationGodGame> {
        // The ttf file will be in your resources directory. Later, we
        // will mount that directory so we can omit it in the path here.
        let font = graphics::Font::new(ctx, "/DejaVuSerif.ttf")?;
        let text = graphics::Text::new(("Hello world!", font, 48.0));

        let dragon = graphics::Image::new(ctx, "/dragon1.png")?;
        let shot_linear = graphics::Image::new(ctx, "/shot.png")?;
        let mut shot_nearest = graphics::Image::new(ctx, "/shot.png")?;
        shot_nearest.set_filter(graphics::FilterMode::Nearest);
        let meshes = vec![build_mesh(ctx)?, build_textured_triangle(ctx)?];

        // Load/create resources such as images here and otherwise initialize state
        let s = SpaceStationGodGame {
            dt: std::time::Duration::new(0, 0),
            circle_pos_x: 0.0,
            frames: 0,
            text: text,
            dragon: dragon,
            shot_linear: shot_linear,
            shot_nearest: shot_nearest,
            rotation: 1.0,
            meshes: meshes,
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
        self.circle_pos_x = self.circle_pos_x % graphics::drawable_size(&ctx).0 + 1.0; // Move the circle to the right and wrap around when the end is reached

        // Rotate one of the images at 60fps
        const DESIRED_FPS: u32 = 60;
        while timer::check_update_time(ctx, DESIRED_FPS) {
            self.rotation += 0.01;
        }
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
            mint::Point2 { x: 0.0, y: 0.0 },
            100.0,
            0.1,
            graphics::WHITE,
        )?;
        // Draw the circle at the position
        graphics::draw(ctx, &circle, (na::Point2::new(self.circle_pos_x, 380.0),))?;

        // Draw some text moving from top-left to bottom-right
        // Drawables are drawn from their top-left corner.
        let offset = self.frames as f32 / 10.0;
        let dest_point = mint::Point2 {
            x: (offset),
            y: (offset),
        };
        graphics::draw(ctx, &self.text, (dest_point,))?;

        // Draw an image.
        let dst = cgmath::Point2::new(20.0, 20.0);
        graphics::draw(ctx, &self.dragon, (dst,))?;

        // Draw an image with some options, and different filter modes.
        let dst = cgmath::Point2::new(200.0, 100.0);
        let dst2 = cgmath::Point2::new(400.0, 400.0);
        let scale = cgmath::Vector2::new(10.0, 10.0);

        graphics::draw(
            ctx,
            &self.shot_linear,
            graphics::DrawParam::new()
                .dest(dst)
                .rotation(self.rotation)
                .scale(scale),
        )?;
        graphics::draw(
            ctx,
            &self.shot_nearest,
            graphics::DrawParam::new()
                .dest(dst2)
                .rotation(self.rotation)
                .offset(na::Point2::new(0.5, 0.5))
                .scale(scale),
        )?;

        // Create and draw a filled rectangle mesh.
        let rect = graphics::Rect::new(450.0, 450.0, 50.0, 50.0);
        let r1 =
            graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), rect, graphics::WHITE)?;
        graphics::draw(ctx, &r1, DrawParam::default())?;

        // Create and draw a stroked rectangle mesh.
        let rect = graphics::Rect::new(450.0, 450.0, 50.0, 50.0);
        let r2 = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::stroke(1.0),
            rect,
            graphics::Color::new(1.0, 0.0, 0.0, 1.0),
        )?;
        graphics::draw(ctx, &r2, DrawParam::default())?;

        // Draw some pre-made meshes
        for m in &self.meshes {
            graphics::draw(ctx, m, DrawParam::new())?;
        }

        // Actually draw everything to the screen
        graphics::present(ctx)?;

        // Increment the number of frames drawn
        self.frames += 1;
        if (self.frames % 100) == 0 {
            // Every 100 frames print some stats to the console
            println!(
                "FPS: {}, dt: {}ns, ticks: {}",
                ggez::timer::fps(ctx),
                self.dt.subsec_nanos(),
                ggez::timer::ticks(ctx)
            );
        }

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
            KeyCode::Space => {
                graphics::set_fullscreen(ctx, conf::FullscreenType::True).unwrap();
            }
            _ => (), // Do nothing
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
        .window_setup(conf::WindowSetup::default().title("Space Station God Game"))
        .window_mode(conf::WindowMode::default().dimensions(1280.0, 960.0))
        .build()?;
    println!("{}", graphics::renderer_info(&ctx)?);
    println!("Game resource path: {:?}", ctx.filesystem);
    println!("{:#?}", graphics::drawable_size(&ctx));

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let mut state = SpaceStationGodGame::new(&mut ctx)?;

    // Run!
    event::run(&mut ctx, &mut event_loop, &mut state)
}

fn build_mesh(ctx: &mut Context) -> GameResult<graphics::Mesh> {
    let mb = &mut graphics::MeshBuilder::new();

    mb.line(
        &[
            na::Point2::new(200.0, 200.0),
            na::Point2::new(400.0, 200.0),
            na::Point2::new(400.0, 400.0),
            na::Point2::new(200.0, 400.0),
            na::Point2::new(200.0, 300.0),
        ],
        4.0,
        Color::new(1.0, 0.0, 0.0, 1.0),
    )?;

    mb.ellipse(
        DrawMode::fill(),
        na::Point2::new(600.0, 200.0),
        50.0,
        120.0,
        1.0,
        Color::new(1.0, 1.0, 0.0, 1.0),
    );

    mb.circle(
        DrawMode::fill(),
        na::Point2::new(600.0, 380.0),
        40.0,
        1.0,
        Color::new(1.0, 0.0, 1.0, 1.0),
    );

    mb.build(ctx)
}

fn build_textured_triangle(ctx: &mut Context) -> GameResult<graphics::Mesh> {
    let mb = &mut graphics::MeshBuilder::new();
    let triangle_verts = vec![
        graphics::Vertex {
            pos: [100.0, 100.0],
            uv: [1.0, 1.0],
            color: [1.0, 0.0, 0.0, 1.0],
        },
        graphics::Vertex {
            pos: [0.0, 100.0],
            uv: [0.0, 1.0],
            color: [0.0, 1.0, 0.0, 1.0],
        },
        graphics::Vertex {
            pos: [0.0, 0.0],
            uv: [0.0, 0.0],
            color: [0.0, 0.0, 1.0, 1.0],
        },
    ];

    let triangle_indices = vec![0, 1, 2];

    let i = graphics::Image::new(ctx, "/rock.png")?;
    mb.raw(&triangle_verts, &triangle_indices, Some(i));
    mb.build(ctx)
}
