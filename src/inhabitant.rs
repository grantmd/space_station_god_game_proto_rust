use crate::item::*;
use crate::station::{GridPosition, Station, Tile, TileType};

use ggez::graphics::{Color, DrawMode, DrawParam, Mesh};
use ggez::{graphics, timer, Context, GameResult};

use keyframe::{ease, functions::EaseInOut};
use oorandom::Rand32;
use uuid::Uuid;

use std::{fmt, time};

// Alias some types to making reading/writing code easier and also in case math libraries change again
type Point2 = glam::Vec2;

// An Inhabitant of the Station
#[derive(Debug)]
pub struct Inhabitant {
    // These are world positions (since they can go outside the station)
    pub pos: Point2,          // Current position
    pub dest: Option<Point2>, // Pathfinding destination to reach

    // Pathfinding status
    path: Vec<GridPosition>,
    current_waypoint: usize,
    move_elapsed: f64, // Seconds we've been moving from source to dest

    kind: InhabitantType,
    health: u8,
    hunger: u8,
    thirst: u8,
    age: time::Duration,

    items: Vec<Box<dyn Item>>,

    id: uuid::Uuid,
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

impl fmt::Display for Inhabitant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{} ({:?}, {}), Health: {}/100, Hunger: {}, Thirst: {}]",
            self.id,
            self.kind,
            self.age.as_secs(),
            self.health,
            self.hunger,
            self.thirst
        )
    }
}

impl Inhabitant {
    pub fn new(pos: Point2, kind: InhabitantType) -> Inhabitant {
        Inhabitant {
            id: Uuid::new_v4(),
            pos,
            dest: None,
            path: Vec::new(),
            current_waypoint: 0,
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

        // Move
        match self.dest {
            Some(_) => {
                self.keep_moving(dt, station);
            }
            None => {
                let tile = station.get_random_tile(TileType::Floor, rng);

                if self.can_move_to(tile) {
                    let dest = tile.unwrap().to_world_position(station);
                    self.set_destination(&station, dest);
                }
            }
        }

        Ok(())
    }

    pub fn draw(&mut self, ctx: &mut Context, camera: &crate::Camera) -> GameResult<()> {
        let color = match self.kind {
            InhabitantType::Ghost => Color::new(0.8, 0.8, 0.8, 0.8),
            _ => Color::WHITE,
        };

        let mesh = Mesh::new_circle(
            ctx,
            DrawMode::fill(),
            self.pos,
            crate::TILE_WIDTH / 2.0 - 10.0,
            0.1,
            color,
        )?;
        graphics::draw(
            ctx,
            &mesh,
            DrawParam::default().offset(camera.pos).scale(camera.zoom),
        )?;

        // TODO: Highlight our destination and maybe path if we have one
        if let Some(dest) = self.dest {
            let tile_rect = graphics::Rect::new(
                dest.x - (crate::TILE_WIDTH / 2.0) + 1.0,
                dest.y - (crate::TILE_WIDTH / 2.0) + 1.0,
                crate::TILE_WIDTH - 2.0,
                crate::TILE_WIDTH - 2.0,
            );
            let mesh = Mesh::new_rectangle(
                ctx,
                DrawMode::stroke(1.0),
                tile_rect,
                Color::new(1.0, 1.0, 0.0, 1.0),
            )?;
            graphics::draw(
                ctx,
                &mesh,
                DrawParam::default().offset(camera.pos).scale(camera.zoom),
            )?;
        }

        Ok(())
    }

    pub fn set_destination(&mut self, station: &Station, dest: Point2) {
        if dest != self.pos {
            println!("{} Pathing from {} to {}", self, self.pos, dest);
            // TODO: All this position unit translation is annoying. Cleanup?
            let path = station.path_to(
                station.get_tile_from_world(self.pos).unwrap().pos,
                station.get_tile_from_world(dest).unwrap().pos,
            );

            if !path.is_empty() {
                self.move_elapsed = 0.0;
                self.path = path;
                self.current_waypoint = 0;
                self.dest = Some(dest);
            } else {
                println!("{} No path", self);
            }
        }
    }

    fn keep_moving(&mut self, dt: time::Duration, station: &Station) {
        if self.dest == None {
            return;
        }

        // Keep going until we get there
        let next_waypoint = station
            .get_tile(self.path[self.current_waypoint])
            .unwrap()
            .to_world_position(station);
        if self.pos == next_waypoint {
            self.current_waypoint += 1;
            self.move_elapsed = 0.0;

            // Moving takes work!
            self.add_hunger(1);
            self.add_thirst(1);
        }
        self.move_elapsed += timer::duration_to_f64(dt);

        // The ease functions want mint types
        let source: mint::Point2<f32> = self.pos.into();
        let next: mint::Point2<f32> = next_waypoint.into();

        // Ease in over 2 seconds per square
        self.pos = ease(EaseInOut, source, next, self.move_elapsed / 2.0).into();

        // We there?
        if self.pos == self.dest.unwrap() {
            println!("{} Arrived at {}", self, self.pos);
            self.dest = None;
        }
    }

    pub fn add_hunger(&mut self, value: u8) {
        if self.kind == InhabitantType::Ghost {
            return;
        }

        self.hunger += value;
        if self.hunger >= 100 {
            self.hunger = 100;
            self.take_damage(1);
            println!("{} Starving! Taking damage.", self);
        }
    }

    pub fn add_thirst(&mut self, value: u8) {
        if self.kind == InhabitantType::Ghost {
            return;
        }

        self.thirst += value;
        if self.thirst >= 100 {
            self.thirst = 100;
            self.take_damage(1);
            println!("{} Parched! Taking damage.", self);
        }
    }

    pub fn eat(&mut self, item: &Food) {
        self.hunger = self.hunger.saturating_sub(item.energy);
    }

    pub fn drink(&mut self, item: &Drink) {
        self.thirst = self.thirst.saturating_sub(item.hydration);
    }

    pub fn take_damage(&mut self, amount: u8) {
        if self.kind == InhabitantType::Ghost {
            return;
        }

        self.health = self.health.saturating_sub(amount);
        if self.health == 0 {
            println!("{} I die. I am dead.", self);
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
