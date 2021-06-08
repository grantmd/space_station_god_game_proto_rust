use super::scene::*;

use ggez::{Context, GameResult};

pub struct Game {}

impl Scene for Game {
    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn get_type(&self) -> SceneType {
        SceneType::Game
    }

    fn from(&self, kind: SceneType) {}
    fn to(&self, kind: SceneType) {}
}
