use super::scene::*;

use ggez::event::{KeyCode, KeyMods};
use ggez::{Context, GameResult};

pub struct Settings {}

impl Scene for Settings {
    fn get_type(&self) -> SceneType {
        SceneType::Settings
    }

    fn draw(&self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        _keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) -> SceneAction {
        SceneAction::None
    }
    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, _y: f32) -> SceneAction {
        SceneAction::None
    }
    fn resize_event(&mut self, _ctx: &mut Context, _width: f32, _height: f32) -> SceneAction {
        SceneAction::None
    }

    fn from_scene(&mut self, _kind: SceneType) {}
}
