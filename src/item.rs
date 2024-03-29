use crate::camera::Camera;
use crate::station::gridposition::*;

use ggez::graphics::{Color, DrawMode, DrawParam, Mesh};
use ggez::{graphics, Context, GameError, GameResult};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use core::fmt;

// Alias some types to making reading/writing code easier and also in case math libraries change again
type Point2 = glam::Vec2;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum ItemType {
    Food(FoodType),
    Drink(DrinkType),
    Container(ContainerType),
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, EnumIter, Serialize, Deserialize)]
pub enum FoodType {
    EnergyBar,
    MealReadyToEat, // Because MRE is not a valid Enum name
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, EnumIter, Serialize, Deserialize)]
pub enum DrinkType {
    Water,
    Coffee,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, EnumIter, Serialize, Deserialize)]
pub enum ContainerType {
    Fridge,
    Locker,
}

// Helper functions to return all possible subtypes of a given item type
pub fn get_food_types() -> Vec<ItemType> {
    let mut types = vec![];
    for kind in FoodType::iter() {
        types.push(ItemType::Food(kind));
    }
    types
}

pub fn get_drink_types() -> Vec<ItemType> {
    let mut types = vec![];
    for kind in DrinkType::iter() {
        types.push(ItemType::Drink(kind));
    }
    types
}

pub fn get_container_types() -> Vec<ItemType> {
    let mut types = vec![];
    for kind in ContainerType::iter() {
        types.push(ItemType::Container(kind));
    }
    types
}

// An item is the base of objects that live inside the station on tiles and inhabitants can interact
#[derive(Serialize, Deserialize, Clone)]
pub struct Item {
    id: uuid::Uuid,
    kind: ItemType,
    pub pos: super::GridPosition,
    items: Vec<Item>,
    capacity: usize,
}

impl fmt::Debug for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{} ({:?})] {}", self.id, self.kind, self.get_name())
    }
}

impl Item {
    pub fn new(pos: super::GridPosition, kind: ItemType) -> Item {
        // Containers have a capacity, other stuff doesn't
        let capacity = match kind {
            ItemType::Container(_) => 10,
            _ => 0,
        };

        // Create the item itself
        let mut i = Item {
            id: Uuid::new_v4(),
            kind,
            pos,
            items: Vec::with_capacity(capacity),
            capacity,
        };

        // Some item types modify after creation
        match kind {
            // Fill the fridge with energy bars and water
            ItemType::Container(ContainerType::Fridge) => {
                i.add_item(Item::new(
                    GridPosition::new(0, 0),
                    ItemType::Food(FoodType::EnergyBar),
                ))
                .unwrap();
                i.add_item(Item::new(
                    GridPosition::new(1, 0),
                    ItemType::Drink(DrinkType::Water),
                ))
                .unwrap();
                i.add_item(Item::new(
                    GridPosition::new(2, 0),
                    ItemType::Food(FoodType::EnergyBar),
                ))
                .unwrap();
                i.add_item(Item::new(
                    GridPosition::new(3, 0),
                    ItemType::Drink(DrinkType::Water),
                ))
                .unwrap();
                i.add_item(Item::new(
                    GridPosition::new(4, 0),
                    ItemType::Food(FoodType::EnergyBar),
                ))
                .unwrap();
                i.add_item(Item::new(
                    GridPosition::new(5, 0),
                    ItemType::Drink(DrinkType::Water),
                ))
                .unwrap();
                i.add_item(Item::new(
                    GridPosition::new(6, 0),
                    ItemType::Food(FoodType::EnergyBar),
                ))
                .unwrap();
                i.add_item(Item::new(
                    GridPosition::new(7, 0),
                    ItemType::Drink(DrinkType::Water),
                ))
                .unwrap();
                i.add_item(Item::new(
                    GridPosition::new(8, 0),
                    ItemType::Food(FoodType::EnergyBar),
                ))
                .unwrap();
                i.add_item(Item::new(
                    GridPosition::new(9, 0),
                    ItemType::Drink(DrinkType::Water),
                ))
                .unwrap();
            }
            _ => (),
        }

        // Return it
        i
    }

    // Get the ID of an object. Used instead of making the property public
    // because we don't want other modules to be able to set it
    pub fn get_id(&self) -> Uuid {
        self.id
    }

    pub fn get_name(&self) -> String {
        match self.kind {
            ItemType::Food(food_type) => match food_type {
                FoodType::EnergyBar => format!(
                    "Your basic energy bar. Restores {} hunger",
                    self.get_energy()
                ),
                FoodType::MealReadyToEat => {
                    format!("An entire MRE. Restores {} hunger", self.get_energy())
                }
            },
            ItemType::Drink(drink_type) => match drink_type {
                DrinkType::Water => format!(
                    "Your standard bottle of water. Restores {} thirst",
                    self.get_hydration()
                ),
                DrinkType::Coffee => format!(
                    "A cup of \"fresh\"-brewed coffed. Restores {} thirst and {} energy",
                    self.get_hydration(),
                    self.get_energy()
                ),
            },
            ItemType::Container(container_type) => match container_type {
                ContainerType::Fridge => {
                    format!("Keeps food and drink cold. Has {} items.", self.items.len())
                }
                ContainerType::Locker => {
                    format!("Storage container. Has {} items.", self.items.len())
                }
            },
        }
    }

    pub fn draw(&self, ctx: &mut Context, station_pos: Point2, camera: &Camera) -> GameResult<()> {
        let pos = Point2::new(
            (crate::TILE_WIDTH * self.pos.x as f32) - (crate::TILE_WIDTH / 2.0),
            (crate::TILE_WIDTH * self.pos.y as f32) - (crate::TILE_WIDTH / 2.0),
        );
        let mesh = match self.kind {
            ItemType::Food(_) => Mesh::new_circle(
                ctx,
                DrawMode::fill(),
                pos,
                crate::TILE_WIDTH / 2.0 - 10.0,
                0.1,
                Color::new(1.0, 1.0, 0.0, 1.0),
            )?,
            ItemType::Drink(_) => Mesh::new_circle(
                ctx,
                DrawMode::fill(),
                pos,
                crate::TILE_WIDTH / 2.0 - 10.0,
                0.1,
                Color::new(1.0, 1.0, 0.0, 1.0),
            )?,
            ItemType::Container(_) => Mesh::new_rectangle(
                ctx,
                DrawMode::fill(),
                graphics::Rect::new(pos.x + 10.0, pos.y + 10.0, 10.0, 10.0),
                Color::new(0.5, 0.5, 0.5, 1.0),
            )?,
        };
        graphics::draw(
            ctx,
            &mesh,
            DrawParam::default()
                .dest(station_pos)
                .offset(camera.pos)
                .scale(camera.zoom),
        )
    }

    pub fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Update all the contents
        for item in self.items.iter_mut() {
            item.update(ctx)?;
        }

        Ok(())
    }

    pub fn get_type(&self) -> ItemType {
        self.kind
    }

    pub fn get_items(&self) -> &Vec<Item> {
        &self.items
    }
    pub fn add_item(&mut self, item: Item) -> GameResult<()> {
        if self.items.len() >= self.capacity {
            return Err(GameError::CustomError(
                "Container is at capacity".to_string(),
            ));
        }

        self.items.push(item);

        Ok(())
    }

    // Given an item uuid, removes it from the container
    pub fn remove_item(&mut self, id: uuid::Uuid) {
        self.items.retain(|item| item.id != id)
    }

    pub fn get_energy(&self) -> u8 {
        match self.kind {
            ItemType::Food(food_type) => match food_type {
                FoodType::EnergyBar => 10,
                FoodType::MealReadyToEat => 50,
            },
            ItemType::Drink(drink_type) => match drink_type {
                DrinkType::Coffee => 3,
                DrinkType::Water => 0,
            },
            _ => 0,
        }
    }

    pub fn get_hydration(&self) -> u8 {
        match self.kind {
            ItemType::Drink(drink_type) => match drink_type {
                DrinkType::Water => 10,
                DrinkType::Coffee => 8,
            },
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ContainerType, FoodType, Item, ItemType};
    use crate::station::gridposition::*;

    #[test]
    fn new_fridge_contains_items() {
        let fridge = Item::new(
            GridPosition::new(1, 1),
            ItemType::Container(ContainerType::Fridge),
        );
        assert_eq!(fridge.capacity, fridge.items.len()); // Fridges come full
    }

    #[test]
    fn fridge_add_item() {
        let mut fridge = Item::new(
            GridPosition::new(1, 1),
            ItemType::Container(ContainerType::Fridge),
        );
        fridge.items.clear(); // Empty the fridge
        assert!(fridge
            .add_item(Item::new(fridge.pos, ItemType::Food(FoodType::EnergyBar)))
            .is_ok());
        assert_eq!(1, fridge.items.len());
    }

    #[test]
    fn fridge_max_items() {
        let mut fridge = Item::new(
            GridPosition::new(1, 1),
            ItemType::Container(ContainerType::Fridge),
        );
        while fridge.items.len() < fridge.capacity {
            assert!(fridge
                .add_item(Item::new(fridge.pos, ItemType::Food(FoodType::EnergyBar)))
                .is_ok());
        }

        let result = fridge.add_item(Item::new(fridge.pos, ItemType::Food(FoodType::EnergyBar)));
        assert!(result.is_err());
    }

    #[test]
    fn fridge_remove_item() {
        let mut fridge = Item::new(
            GridPosition::new(1, 1),
            ItemType::Container(ContainerType::Fridge),
        );

        let id = fridge.get_items().last().unwrap().get_id();
        fridge.remove_item(id);
        assert_eq!(9, fridge.items.len()); // Fridges come full, so we should be down one item
    }
}
