use crate::item::*;
use crate::station::gridposition::*;
use crate::station::station::*;
use crate::station::tile::*;

use ggez::graphics::{Color, DrawMode, DrawParam, Mesh};
use ggez::{graphics, timer, Context, GameResult};

use keyframe::{ease, functions::EaseInOut};
use oorandom::Rand32;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::{fmt, time};

// Alias some types to making reading/writing code easier and also in case math libraries change again
type Point2 = glam::Vec2;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
enum Behavior {
    Wander,
    Search(Vec<ItemType>),
    Eat,
    Drink,
    Work,
}

// An Inhabitant of the Station
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Inhabitant {
    // These are world positions (since they can go outside the station)
    pub pos: Point2,          // Current position
    pub dest: Option<Point2>, // Pathfinding destination to reach

    // Pathfinding status
    path: Vec<GridPosition>,
    current_waypoint: usize,
    move_elapsed: f64, // Seconds we've been moving from source to dest

    behaviors: Vec<Behavior>,

    kind: InhabitantType,
    health: u8,
    hunger: u8,
    thirst: u8,
    age: time::Duration,

    items: Vec<Item>,

    id: uuid::Uuid,
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum InhabitantType {
    Pilot,
    Engineer,
    Scientist,
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
            "[{} ({:?}, {}s), Behavior: {:?}, Health: {}, Hunger: {}, Thirst: {}]",
            self.id,
            self.kind,
            self.age.as_secs(),
            self.behaviors.last(),
            self.health,
            self.hunger,
            self.thirst
        )
    }
}

impl Inhabitant {
    pub fn new(pos: Point2, kind: InhabitantType) -> Inhabitant {
        let items = match kind {
            InhabitantType::Ghost => vec![],
            _ => vec![
                Item::new(GridPosition::new(0, 0), ItemType::Food(FoodType::EnergyBar)),
                Item::new(GridPosition::new(1, 0), ItemType::Drink(DrinkType::Water)),
            ],
        };

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
            items: items,
            behaviors: Vec::with_capacity(7),
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

        // Where are we?
        let current_tile = station.get_tile_from_world(self.pos).unwrap();

        // Perform next behavior
        let next = self.behaviors.last();
        match next {
            Some(Behavior::Wander) => {
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
            }
            Some(Behavior::Eat) => {
                if self.has_item(get_food_types()) {
                    // If we have food on our person, eat it
                    println!("{} Eating from inventory", self);
                    self.eat(&Item::new(
                        current_tile.pos,
                        ItemType::Food(FoodType::EnergyBar),
                    )); // TODO: Actually consume the food from inventory
                    self.behaviors.pop();
                } else if current_tile.has_item(get_food_types()) {
                    // If there's food here on this tile, eat it
                    println!("{} Eating from tile", self);
                    self.eat(&Item::new(
                        current_tile.pos,
                        ItemType::Food(FoodType::EnergyBar),
                    )); // TODO: Actually consume the food from the tile
                    self.behaviors.pop();
                } else {
                    // Otherwise, search for it
                    println!("{} Searching for food", self);
                    self.behaviors.push(Behavior::Search(get_food_types()));
                }
            }
            Some(Behavior::Drink) => {
                if self.has_item(get_drink_types()) {
                    // If we have drink on our person, drink it
                    println!("{} Drinking from inventory", self);
                    self.drink(&Item::new(
                        current_tile.pos,
                        ItemType::Drink(DrinkType::Water),
                    )); // TODO: Actually consume the drink from inventory
                    self.behaviors.pop();
                } else if current_tile.has_item(get_drink_types()) {
                    // If there's drink here on this tile, drink it
                    println!("{} Drinking from tile", self);
                    self.drink(&Item::new(
                        current_tile.pos,
                        ItemType::Drink(DrinkType::Water),
                    )); // TODO: Actually consume the drink from the tile
                    self.behaviors.pop();
                } else {
                    // Otherwise, search for it
                    println!("{} Searching for drink", self);
                    self.behaviors.push(Behavior::Search(get_drink_types()));
                }
            }
            Some(Behavior::Search(item_types)) => match self.dest {
                Some(_) => {
                    self.keep_moving(dt, station);
                }
                None => {
                    let mut best_path: Vec<GridPosition> = Vec::new();
                    let found = station.find_items(item_types.to_vec());
                    for pos in found.iter() {
                        let path = station.path_to(current_tile.pos, **pos);
                        if best_path.is_empty() || path.len() < best_path.len() {
                            best_path = path;
                        }
                    }

                    if !best_path.is_empty() {
                        let dest = best_path.pop().unwrap();
                        self.set_destination(
                            station,
                            station.get_tile(dest).unwrap().to_world_position(station),
                        );
                    }
                }
            },
            Some(Behavior::Work) => {
                // TODO
            }
            None => {
                // Decide what to do
                if self.wants_food() >= 0.5 {
                    self.behaviors.push(Behavior::Eat);
                } else if self.wants_drink() >= 0.5 {
                    self.behaviors.push(Behavior::Drink);
                } else {
                    self.behaviors.push(Behavior::Wander);
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
            self.behaviors.pop(); // TODO: Is it safe to assume that arriving somewhere means the current behavior is "done"?
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

    pub fn eat(&mut self, item: &Item) {
        // TODO: Test actually edible?
        self.hunger = self.hunger.saturating_sub(item.get_energy());
    }

    pub fn drink(&mut self, item: &Item) {
        // TODO: Test actually drinkable?
        self.thirst = self.thirst.saturating_sub(item.get_hydration());
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

    fn wants_food(&self) -> f32 {
        if self.kind == InhabitantType::Ghost {
            return 0.0;
        }

        // Linear to hunger
        self.hunger as f32 / 100.0
    }

    fn wants_drink(&self) -> f32 {
        if self.kind == InhabitantType::Ghost {
            return 0.0;
        }

        // Linear to thirst
        self.thirst as f32 / 100.0
    }

    // Do we have an item of this type on us?
    fn has_item(&self, item_types: Vec<ItemType>) -> bool {
        for item in self.items.iter() {
            if item_types.contains(&item.get_type()) {
                return true;
            }

            // If this is a container, we need to iterate inside
            match item.get_type() {
                ItemType::Container(_) => {
                    for subitem in item.get_items().iter() {
                        // Is this what we're looking for?
                        if item_types.contains(&subitem.get_type()) {
                            return true;
                        }
                    }
                }
                _ => (),
            }
        }

        false
    }

    // Given an item uuid, removes it from our inventory
    pub fn remove_item(&mut self, id: uuid::Uuid) {
        self.items.retain(|item| item.get_id() != id)
    }
}

#[cfg(test)]
mod tests {
    use super::{Inhabitant, InhabitantType, Point2};
    use crate::station::gridposition::*;
    use crate::station::tile::*;

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

    #[test]
    fn inhabitant_wants_food() {
        let mut inhabitant = Inhabitant::new(Point2::new(1.0, 1.0), InhabitantType::Engineer);
        assert_eq!(
            0.0,
            inhabitant.wants_food(),
            "New inhabitants don't want food"
        );

        inhabitant.hunger = 50;
        assert_eq!(0.5, inhabitant.wants_food(), "Partly hungry");

        inhabitant.hunger = 100;
        assert_eq!(1.0, inhabitant.wants_food(), "Starving");

        inhabitant.kind = InhabitantType::Ghost;
        assert_eq!(0.0, inhabitant.wants_food(), "Ghosts aren't hungry");
    }

    #[test]
    fn inhabitant_wants_drink() {
        let mut inhabitant = Inhabitant::new(Point2::new(1.0, 1.0), InhabitantType::Engineer);
        assert_eq!(
            0.0,
            inhabitant.wants_drink(),
            "New inhabitants don't want drink"
        );

        inhabitant.thirst = 50;
        assert_eq!(0.5, inhabitant.wants_drink(), "Partly thirsty");

        inhabitant.thirst = 100;
        assert_eq!(1.0, inhabitant.wants_drink(), "Parched");

        inhabitant.kind = InhabitantType::Ghost;
        assert_eq!(0.0, inhabitant.wants_drink(), "Ghosts aren't thirsty");
    }
}
