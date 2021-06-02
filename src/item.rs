use ggez::graphics::{Color, DrawMode, DrawParam, Mesh};
use ggez::{graphics, Context, GameError, GameResult};

use uuid::Uuid;

use core::fmt;

// Alias some types to making reading/writing code easier and also in case math libraries change again
type Point2 = glam::Vec2;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ItemType {
    Food(FoodType),
    Drink(DrinkType),
    Container(ContainerType),
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum FoodType {
    EnergyBar,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum DrinkType {
    Water,
    Coffee,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ContainerType {
    Fridge,
    Locker,
}

// An item is the base of objects that live inside the station on tiles and inhabitants can interact
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
        let capacity = match kind {
            ItemType::Container(_) => 10,
            _ => 0,
        };

        Item {
            id: Uuid::new_v4(),
            kind: kind,
            pos,
            items: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }

    pub fn get_name(&self) -> String {
        match self.kind {
            ItemType::Food(food_type) => {
                format!("Yummy yummy food. Restores {} hunger", 10)
            }
            ItemType::Drink(drink_type) => {
                format!("A thirst-quenching beverage. Restores {} thirst", 10)
            }
            ItemType::Container(container_type) => {
                format!("Storage container. Has {} items.", self.items.len())
            }
        }
    }

    pub fn draw(
        &self,
        ctx: &mut Context,
        station_pos: Point2,
        camera: &crate::Camera,
    ) -> GameResult<()> {
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
    // Given an item uuid, removes it from the fridge
    pub fn remove_item(&mut self, id: uuid::Uuid) {
        self.items.retain(|item| item.id == id)
    }

    pub fn get_energy(&self) -> u8 {
        match self.kind {
            ItemType::Food(food_type) => match food_type {
                FoodType::EnergyBar => 10,
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
    use crate::station::GridPosition;

    #[test]
    fn new_fridge_contains_items() {
        let fridge = Item::new(
            GridPosition::new(1, 1),
            ItemType::Container(ContainerType::Fridge),
        );
        assert_eq!(1, fridge.items.len());
    }

    #[test]
    fn fridge_add_item() {
        let fridge = Item::new(
            GridPosition::new(1, 1),
            ItemType::Container(ContainerType::Fridge),
        );
        assert!(fridge
            .add_item(Item::new(fridge.pos, ItemType::Food(FoodType::EnergyBar)))
            .is_ok());
        assert_eq!(1, fridge.items.len());
    }

    #[test]
    fn fridge_max_items() {
        let fridge = Item::new(
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
        let fridge = Item::new(
            GridPosition::new(1, 1),
            ItemType::Container(ContainerType::Fridge),
        );
        let food = Item::new(fridge.pos, ItemType::Food(FoodType::EnergyBar));
        let id = food.get_id();
        assert!(fridge.add_item(food).is_ok());
        assert_eq!(1, fridge.items.len());

        fridge.remove_item(id);
        assert_eq!(0, fridge.items.len());
    }
}
