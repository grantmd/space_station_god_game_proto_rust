use ggez::{Context, GameResult};

// A scene represents a screen to be drawn. This can be something like a loading screen, a menu, the game itself, etc
// Responsible for drawing itself, doing updates, and handling input events
pub trait Scene {
    fn draw(&self, ctx: &mut Context) -> GameResult<()>;

    fn update(&mut self, ctx: &mut Context) -> GameResult<()>;

    fn get_type(&self) -> SceneType;

    fn from(&self, kind: SceneType);
    fn to(&self, kind: SceneType);
}

// The list of unique, valid scene types
pub enum SceneType {
    Title,
    Game,
    Load,
    Save,
    Quit,
    Settings,
}
