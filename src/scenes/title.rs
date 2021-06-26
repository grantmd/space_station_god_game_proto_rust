use super::game::*;
use super::load::*;
use super::quit::*;
use super::scene::*;

use ggez::event::{KeyCode, KeyMods};
use ggez::graphics::{Color, DrawParam, Font, PxScale, Text, TextFragment};
use ggez::{graphics, Context, GameResult};

type Point2 = glam::Vec2;

pub struct Title {}

impl Scene for Title {
    fn get_type(&self) -> SceneType {
        SceneType::Title
    }

    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        let (screen_width, screen_height) = graphics::drawable_size(ctx);

        // Big game title
        let title_font = Font::new(ctx, "/fonts/Moonhouse-yE5M.ttf")?;
        let title = Text::new(
            TextFragment::new("SPACE\nSTATION\nGOD\nGAME")
                .font(title_font)
                .scale(PxScale::from(80.0)),
        );
        graphics::queue_text(ctx, &title, Point2::new(10.0, 0.0), Some(Color::WHITE));
        let height = title.height(ctx);

        // Instructions
        let instructions = Text::new(format!(
            "Press N for a new game, L to load a previous save."
        ));
        graphics::queue_text(
            ctx,
            &instructions,
            Point2::new(10.0, height + 10.0),
            Some(Color::new(1.0, 1.0, 0.0, 1.0)),
        );

        // Copyright
        let copyright = Text::new(format!("Copyright 2021 Myles Grant"));
        let copyright_dims = copyright.dimensions(ctx);
        let music = Text::new(format!("Music: www.bensound.com"));
        let music_dims = music.dimensions(ctx);
        graphics::queue_text(
            ctx,
            &copyright,
            Point2::new(
                screen_width - copyright_dims.w - 10.0,
                screen_height - copyright_dims.h - 20.0 - music_dims.h,
            ),
            Some(Color::WHITE),
        );
        graphics::queue_text(
            ctx,
            &music,
            Point2::new(
                screen_width - music_dims.w - 10.0,
                screen_height - music_dims.h - 10.0,
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
        match keycode {
            // Quit
            KeyCode::Escape | KeyCode::Q if !repeat => {
                SceneAction::Push(Box::new(Quit {}))
            }

            // Create a new game
            KeyCode::N if !repeat => {
                println!("Creating new game");
                SceneAction::PopAndPush(Box::new(Game::new(ctx)))
            }

            // Load a game
            KeyCode::L if !repeat => {
                println!("Loading new game");
                SceneAction::Push(Box::new(Load {}))
            }

            // Everything else does nothing
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
