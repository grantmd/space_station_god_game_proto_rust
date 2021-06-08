use super::scene::*;

use ggez::{Context, GameResult};

pub struct Title {}

impl Scene for Title {
    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn get_type(&self) -> SceneType {
        SceneType::Title
    }

    fn from(&self, kind: SceneType) {}
    fn to(&self, kind: SceneType) {}
}
