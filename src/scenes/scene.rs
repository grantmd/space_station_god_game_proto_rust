use ggez::event::{KeyCode, KeyMods};
use ggez::{Context, GameResult};

// A scene represents a screen to be drawn. This can be something like a loading screen, a menu, the game itself, etc
// Responsible for drawing itself, doing updates, and handling input events
pub trait Scene {
    fn get_type(&self) -> SceneType;

    fn draw(&self, ctx: &mut Context) -> GameResult<()>;
    fn update(&mut self, ctx: &mut Context) -> GameResult<()>;

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        keymods: KeyMods,
        repeat: bool,
    ) -> SceneAction;
    fn mouse_wheel_event(&mut self, ctx: &mut Context, x: f32, y: f32) -> SceneAction;
    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) -> SceneAction;
}

// The list of unique, valid scene types
#[derive(Eq, PartialEq, Debug)]
pub enum SceneType {
    Title,
    Game,
    Load,
    Save,
    Quit,
    Settings,
}

pub enum SceneAction {
    Pop,
    Push(Box<dyn Scene>),
    PopAndPush(Box<dyn Scene>),
    None,
}
