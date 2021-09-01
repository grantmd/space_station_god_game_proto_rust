use super::scene::*;

use ggez::event::{self, KeyCode, KeyMods};
use ggez::graphics::{Color, DrawMode, DrawParam, Text};
use ggez::{graphics, Context, GameResult};

type Point2 = glam::Vec2;

pub struct Quit {}

impl Scene for Quit {
    fn get_type(&self) -> SceneType {
        SceneType::Quit
    }

    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        let (screen_width, screen_height) = graphics::drawable_size(ctx);
        let (width, height) = (530.0, 80.0);
        let dialog_rect = graphics::Rect::new(
            screen_width / 2.0 - width / 2.0,
            screen_height / 2.0 - height / 2.0,
            width,
            height,
        );
        let mesh = graphics::Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            dialog_rect,
            Color::new(0.0, 0.0, 0.0, 1.0),
        )?;
        graphics::draw(ctx, &mesh, DrawParam::default())?;
        let mesh = graphics::Mesh::new_rectangle(
            ctx,
            DrawMode::stroke(1.0),
            dialog_rect,
            Color::new(1.0, 1.0, 1.0, 1.0),
        )?;
        graphics::draw(ctx, &mesh, DrawParam::default())?;

        let instructions = Text::new("Are you sure you want to quit? (Y/N)".to_string());
        let instructions_dims = instructions.dimensions(ctx);
        graphics::queue_text(
            ctx,
            &instructions,
            Point2::new(
                screen_width / 2.0 - instructions_dims.w / 2.0,
                screen_height / 2.0 - instructions_dims.h / 2.0,
            ),
            Some(Color::WHITE),
        );

        // Render all queued text
        graphics::draw_queued_text(
            ctx,
            DrawParam::default(),
            None,
            graphics::FilterMode::Linear,
        )?;

        Ok(())
    }

    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        repeat: bool,
    ) -> SceneAction {
        let mut action = SceneAction::None;
        match keycode {
            // Quit?
            KeyCode::Y if !repeat => event::quit(ctx),
            KeyCode::N if !repeat => action = SceneAction::Pop,
            KeyCode::Escape if !repeat => action = SceneAction::Pop,
            _ => (),
        }

        action
    }
    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, _y: f32) -> SceneAction {
        SceneAction::None
    }
    fn resize_event(&mut self, _ctx: &mut Context, _width: f32, _height: f32) -> SceneAction {
        SceneAction::None
    }

    fn from_scene(&mut self, _kind: SceneType) {}
    fn to_scene(&mut self, _kind: SceneType) {}
}
