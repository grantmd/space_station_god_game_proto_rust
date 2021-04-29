use crate::item::*;
use crate::station::{Station, Tile, TileType};

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
    // These are all world positions (since they can go outside the station)
    pub pos: Point2,          // Current position
    source: Point2,           // Pathfinding source position
    next_waypoint: Point2,    // Pathfinding next waypoint to move to
    pub dest: Option<Point2>, // Pathfinding destination to reach

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
            pos,
            source: pos,
            next_waypoint: pos,
            dest: None,
            move_elapsed: 0.0,
            kind,
            health: 100,
            hunger: 0,
            thirst: 0,
            age: time::Duration::from_micros(0),
            items: Vec::new(),
        }
    }

    // Whether we can move to a type of tile
    // Doesn't check whether we can _get_ there, but only if we can be there
    pub fn can_move_to(&self, tile: Option<&Tile>) -> bool {
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
                self.keep_moving(dt, station);
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

        // TODO: Highlight our destination and maybe path if we have one
    }

    pub fn set_destination(&mut self, dest: Point2) {
        if dest != self.pos {
            self.move_elapsed = 0.0;
            self.source = self.pos;
            self.next_waypoint = self.pos;
            self.dest = Some(dest);
        }
    }

    fn keep_moving(&mut self, dt: time::Duration, station: &Station) {
        if self.dest == None {
            return;
        }

        // Keep going until we get there
        if self.pos == self.next_waypoint {
            // TODO: All this position unit translation is annoying. Cleanup?
            let path = station.path_to(
                station.get_tile_from_world(self.pos).unwrap().pos,
                station.get_tile_from_world(self.dest.unwrap()).unwrap().pos,
            );
            if !path.is_empty() {
                self.next_waypoint = station
                    .get_tile(path[0])
                    .unwrap()
                    .to_world_position(station);
                self.move_elapsed = 0.0;
            } else {
                self.dest = None;
                return;
            }
        }
        self.move_elapsed += timer::duration_to_f64(dt);

        // The ease functions want mint types
        let source: mint::Point2<f32> = self.pos.into();
        let next: mint::Point2<f32> = self.next_waypoint.into();

        // Ease in over 3 seconds per square
        self.pos = ease(EaseInOut, source, next, self.move_elapsed / 3.0).into();

        // We there?
        if Some(self.pos) == self.dest {
            self.dest = None;
        }
    }

    pub fn eat(&mut self, item: &Food) {
        self.hunger -= item.energy;
    }

    pub fn take_damage(&mut self, amount: u8) {
        self.health -= amount;
        if self.health == 0 {
            self.die();
        }
    }

    pub fn die(&mut self) {
        // TODO: What if already a ghost!?
        self.kind = InhabitantType::Ghost;
    }
}

#[cfg(test)]
mod tests {
    use super::{Inhabitant, InhabitantType, Point2};
    use crate::station::{GridPosition, Tile, TileType, WallDirection};

    #[test]
    fn inhabitant_can_move_to() {
        let inhabitant = Inhabitant::new(Point2::new(1.0, 1.0), InhabitantType::Engineer);

        let floor_tile = Tile::new(GridPosition::new(1, 1), TileType::Floor);
        let wall_tile = Tile::new(GridPosition::new(1, 2), TileType::Wall(WallDirection::Full));
        let door_tile = Tile::new(GridPosition::new(1, 3), TileType::Door(WallDirection::Full));

        assert!(
            inhabitant.can_move_to(Some(&floor_tile)),
            "Inhabitants can move to floors"
        );
        assert!(
            inhabitant.can_move_to(Some(&door_tile)),
            "Inhabitants can move to doors"
        );
        assert!(
            !inhabitant.can_move_to(Some(&wall_tile)),
            "Inhabitants cannot move to walls"
        );
        assert!(
            !inhabitant.can_move_to(None),
            "Inhabitants cannot move to empty tiles"
        );

        let ghost = Inhabitant::new(Point2::new(1.0, 2.0), InhabitantType::Ghost);
        assert!(
            ghost.can_move_to(Some(&floor_tile)),
            "Ghosts can move to floors"
        );
        assert!(
            ghost.can_move_to(Some(&door_tile)),
            "Ghosts can move to doors"
        );
        assert!(
            ghost.can_move_to(Some(&wall_tile)),
            "Ghosts can move to walls"
        );
        assert!(ghost.can_move_to(None), "Ghosts can move to empty tiles");
    }
}
