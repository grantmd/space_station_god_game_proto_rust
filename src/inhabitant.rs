use crate::item::*;
use crate::station::{GridPosition, Station, Tile, TileType};

use ggez::graphics::{Color, DrawMode, DrawParam, Mesh};
use ggez::{graphics, timer, Context, GameResult};

use keyframe::{ease, functions::EaseInOut};
use oorandom::Rand32;

use std::time;

// Alias some types to making reading/writing code easier and also in case math libraries change again
type Point2 = glam::Vec2;

// An Inhabitant of the Station
#[derive(Debug)]
pub struct Inhabitant {
    // These are all world positions
    pub pos: Point2,
    source: Point2,
    pub dest: Option<Point2>,
    move_elapsed: f64, // Seconds we've been moving from source to dest

    kind: InhabitantType,
    health: u8,
    hunger: u8,
    thirst: u8,
    age: time::Duration,

    items: Vec<Box<dyn Item>>,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum InhabitantType {
    Pilot,
    Engineer,
    Medic,
    Soldier,
    Miner,
    Cook,
    Ghost,
}

impl Inhabitant {
    pub fn new(pos: Point2, kind: InhabitantType) -> Inhabitant {
        Inhabitant {
            pos: pos,
            source: pos,
            dest: None,
            move_elapsed: 0.0,
            kind: kind,
            health: 100,
            hunger: 0,
            thirst: 0,
            age: time::Duration::from_micros(0),
            items: Vec::new(),
        }
    }

    // Whether we can move to a type of tile
    // Doesn't check whether we can _get_ there, but only if we can be there
    pub fn can_move_to(&mut self, tile: Option<&Tile>) -> bool {
        match self.kind {
            // Ghosts can go anywhere, lol
            InhabitantType::Ghost => true,

            // Everyone else needs to test the type of tile
            _ => match tile {
                Some(t) => match t.kind {
                    TileType::Wall(_) => false,
                    TileType::Door(_) => true, // TODO: Check if we can open it?
                    TileType::Floor => true,
                },
                None => false,
            },
        }
    }

    pub fn update(
        &mut self,
        ctx: &mut Context,
        station: &Station,
        rng: &mut Rand32,
    ) -> GameResult<()> {
        let dt = timer::delta(ctx); // Time since last frame

        // Look, we're growing!
        self.age += dt;

        // Take damage when starving
        if self.hunger >= 100 {
            self.take_damage(1);
        }

        // Move
        match self.dest {
            Some(_) => {
                self.keep_moving(dt);
            }
            None => {
                let tile = station.get_random_tile(TileType::Floor, rng);

                if self.can_move_to(tile) {
                    let dest = tile.unwrap().to_world_position(station);
                    self.set_destination(dest);
                }
            }
        }

        Ok(())
    }

    pub fn draw(&mut self, ctx: &mut Context, camera: &crate::Camera) -> GameResult<()> {
        let mesh = Mesh::new_circle(
            ctx,
            DrawMode::fill(),
            self.pos,
            crate::TILE_WIDTH / 2.0 - 10.0,
            0.1,
            Color::WHITE,
        )?;
        graphics::draw(
            ctx,
            &mesh,
            DrawParam::default().offset(camera.pos).scale(camera.zoom),
        )
    }

    pub fn set_destination(&mut self, dest: Point2) {
        if dest != self.pos {
            self.move_elapsed = 0.0;
            self.source = self.pos;
            self.dest = Some(dest);
        }
    }

    fn keep_moving(&mut self, dt: time::Duration) {
        if self.dest == None {
            return;
        }

        // Keep going until we get there
        self.move_elapsed += timer::duration_to_f64(dt);

        // The ease functions want mint types
        let source: mint::Point2<f32> = self.source.into();
        let dest: mint::Point2<f32> = self.dest.unwrap().into();

        // Ease in over 2 seconds per square
        let distance: f64 = self.source.distance(self.dest.unwrap()).into();
        self.pos = ease(
            EaseInOut,
            source,
            dest,
            self.move_elapsed / 2.0 * distance / crate::TILE_WIDTH as f64,
        )
        .into();

        // We there?
        if self.pos == dest.into() {
            self.dest = None;
        }
    }

    pub fn eat(&mut self, item: &Food) {
        self.hunger -= item.energy;
    }

    pub fn take_damage(&mut self, amount: u8) {
        self.health -= amount;
        if self.health <= 0 {
            self.die();
        }
    }

    pub fn die(&mut self) {
        // TODO: What if already a ghost!?
        self.kind = InhabitantType::Ghost;
    }
}
