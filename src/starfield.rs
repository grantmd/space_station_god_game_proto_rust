use ggez;
use glam;

use ggez::graphics::{Color, DrawParam};
use ggez::{graphics, Context, GameResult};

use oorandom::Rand32;

type Point2 = glam::Vec2;

pub struct Starfield {
    rng: Rand32,
    stars: Vec<Star>,
}

impl Starfield {
    pub fn new(ctx: &mut Context) -> Starfield {
        let (screen_width, screen_height) = graphics::drawable_size(ctx);

        // Create a seeded random-number generator
        let mut seed: [u8; 8] = [0; 8];
        getrandom::getrandom(&mut seed[..]).expect("Could not create RNG seed");
        let mut rng = Rand32::new(u64::from_ne_bytes(seed));

        // Create stars scaled to screen size
        let num_stars = (screen_width * screen_height / 1000.0) as usize;
        let mut stars = Vec::with_capacity(num_stars);
        for _ in 0..num_stars {
            let x = rng.rand_range(0..screen_width as u32) as f32;
            let y = rng.rand_range(0..screen_height as u32) as f32;

            let size = rng.rand_float() * 2.0;

            stars.push(Star {
                pos: Point2::new(x, y),
                size: size,
                color: random_color(&mut rng),
            })
        }

        Starfield {
            rng: rng,
            stars: stars,
        }
    }

    pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        for star in &self.stars {
            let mesh = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                star.pos,
                star.size,
                0.1,
                star.color,
            )?;
            graphics::draw(ctx, &mesh, DrawParam::default())?;
        }
        Ok(())
    }
}

pub struct Star {
    pos: Point2,
    size: f32,
    color: Color,
}

fn random_color(rng: &mut Rand32) -> Color {
    let color = rng.rand_range(0..3);
    match color {
        0 => Color::WHITE,
        1 => Color::new(0.0, 0.0, 1.0, 1.0),
        2 => Color::new(0.0, 1.0, 0.0, 1.0),
        3 => Color::new(1.0, 0.0, 0.0, 1.0),
        _ => Color::BLACK,
    }
}
