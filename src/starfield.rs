use ggez::graphics::{Color, DrawMode, DrawParam, Mesh, MeshBuilder};
use ggez::{graphics, Context, GameResult};

use oorandom::Rand32;

type Point2 = glam::Vec2;

#[derive(Clone, PartialEq, Debug)]
pub struct Starfield {
    stars: Vec<Star>,
    mesh: Option<Mesh>,
    rng: oorandom::Rand32,
}

impl Starfield {
    pub fn new(ctx: &mut Context) -> Starfield {
        // Create a seeded random-number generator
        let mut seed: [u8; 8] = [0; 8];
        getrandom::getrandom(&mut seed[..]).expect("Could not create RNG seed");
        let rng = Rand32::new(u64::from_ne_bytes(seed));
        let (screen_width, screen_height) = graphics::drawable_size(ctx);

        let mut s = Starfield {
            rng,
            stars: Vec::with_capacity(1000),
            mesh: None,
        };

        s.generate_stars(screen_width, screen_height);
        s.generate_mesh(ctx).unwrap();

        s
    }

    pub fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let (screen_width, _) = graphics::drawable_size(ctx);

        for star in self.stars.iter_mut() {
            star.pos -= Point2::unit_x() * 0.01;
            if star.pos.x < 0.0 {
                star.pos += Point2::unit_x() * screen_width;
            }
        }

        self.generate_mesh(ctx)?;

        Ok(())
    }

    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        match &self.mesh {
            Some(mesh) => graphics::draw(ctx, mesh, DrawParam::default()),
            None => Ok(()),
        }
    }

    pub fn resize_event(&mut self, ctx: &mut Context, screen_width: f32, screen_height: f32) {
        self.generate_stars(screen_width, screen_height);
        self.generate_mesh(ctx).unwrap();
    }

    // Create stars scaled to screen size
    fn generate_stars(&mut self, screen_width: f32, screen_height: f32) {
        let num_stars = (screen_width * screen_height / 1000.0) as usize;
        self.stars.clear();

        for _ in 0..num_stars {
            let x = self.rng.rand_range(0..screen_width as u32) as f32;
            let y = self.rng.rand_range(0..screen_height as u32) as f32;

            let size = num::pow(self.rng.rand_float() + 0.1, 4) * 2.0;

            self.stars.push(Star {
                pos: Point2::new(x, y),
                size,
                color: random_color(&mut self.rng),
            })
        }
    }

    fn generate_mesh(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mb = &mut MeshBuilder::new();
        for star in self.stars.iter() {
            mb.circle(DrawMode::fill(), star.pos, star.size, 1.0, star.color)?;
        }

        self.mesh = mb.build(ctx).ok();

        Ok(())
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Star {
    pub pos: Point2,
    size: f32,
    color: Color,
}

fn random_color(rng: &mut Rand32) -> Color {
    let color = rng.rand_range(0..7);
    match color {
        0 => Color::new(0.0, 1.0, 1.0, 1.0), // cyan
        1 => Color::new(1.0, 1.0, 0.0, 1.0), // yellow
        _ => Color::WHITE,
    }
}
