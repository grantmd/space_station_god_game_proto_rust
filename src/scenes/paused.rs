use super::scene::*;

use ggez::event::{KeyCode, KeyMods};
use ggez::graphics::{Color, DrawMode, DrawParam, Font, PxScale, Text, TextFragment};
use ggez::{graphics, Context, GameResult};

type Point2 = glam::Vec2;

pub struct Paused {}

impl Scene for Paused {
    fn get_type(&self) -> SceneType {
        SceneType::Paused
    }

    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        let (screen_width, screen_height) = graphics::drawable_size(ctx);
        let screen_rect = graphics::Rect::new(0.0, 0.0, screen_width, screen_height);
        let mesh = graphics::Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            screen_rect,
            Color::new(1.0, 1.0, 1.0, 0.1),
        )?;
        graphics::draw(ctx, &mesh, DrawParam::default())?;

        let paused_font = Font::new(ctx, "/fonts/Moonhouse-yE5M.ttf")?;
        let paused_display = Text::new(
            TextFragment::new("PAUSED")
                .font(paused_font)
                .scale(PxScale::from(100.0)),
        );
        let dims = paused_display.dimensions(ctx);
        graphics::queue_text(
            ctx,
            &paused_display,
            Point2::new(
                screen_width / 2.0 - dims.w / 2.0,
                screen_height / 2.0 - dims.h / 2.0,
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
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        repeat: bool,
    ) -> SceneAction {
        match keycode {
            KeyCode::Space if !repeat => {
                println!("Unpausing");
                SceneAction::Pop
            }
            _ => SceneAction::None,
        }
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
