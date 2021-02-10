use ggez::graphics::{Color, DrawParam};
use ggez::{graphics, Context, GameResult};

use oorandom::Rand32;

type Point2 = glam::Vec2;

pub struct Starfield {
    mesh: graphics::Mesh,
}

impl Starfield {
    pub fn new(ctx: &mut Context, rng: &mut Rand32) -> Starfield {
        let (screen_width, screen_height) = graphics::drawable_size(ctx);

        let stars = generate_stars(rng, screen_width, screen_height);
        let mb = generate_mesh(ctx, &stars);
        Starfield { mesh: mb.unwrap() }
    }

    pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::draw(ctx, &self.mesh, DrawParam::default())
    }

    pub fn resize_event(
        &mut self,
        ctx: &mut Context,
        rng: &mut Rand32,
        screen_width: f32,
        screen_height: f32,
    ) {
        let stars = generate_stars(rng, screen_width, screen_height);
        let mb = generate_mesh(ctx, &stars);
        self.mesh = mb.unwrap();
    }
}

pub struct Star {
    pos: Point2,
    size: f32,
    color: Color,
}

fn random_color(rng: &mut Rand32) -> Color {
    let color = rng.rand_range(0..4);
    match color {
        0 => Color::new(0.0, 0.0, 1.0, 1.0), // cyan
        1 => Color::new(1.0, 1.0, 0.0, 1.0), // yellow
        _ => Color::WHITE,
    }
}

// Create stars scaled to screen size
fn generate_stars(rng: &mut Rand32, screen_width: f32, screen_height: f32) -> Vec<Star> {
    let num_stars = (screen_width * screen_height / 2500.0) as usize;
    let mut stars = Vec::with_capacity(num_stars);
    for _ in 0..num_stars {
        let x = rng.rand_range(0..screen_width as u32) as f32;
        let y = rng.rand_range(0..screen_height as u32) as f32;

        let size = (rng.rand_float() + 0.1) * 2.0;

        stars.push(Star {
            pos: Point2::new(x, y),
            size: size,
            color: random_color(rng),
        })
    }

    stars
}

fn generate_mesh(ctx: &mut Context, stars: &Vec<Star>) -> GameResult<graphics::Mesh> {
    let mut mb = graphics::MeshBuilder::new();
    for star in stars {
        mb.circle(
            graphics::DrawMode::fill(),
            star.pos,
            star.size,
            1.0,
            star.color,
        )?;
    }

    mb.build(ctx)
}
