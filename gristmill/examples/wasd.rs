use gristmill::{
    asset,
    input::{InputEvent, InputSystem},
    math::Vec2,
    obj_upcast,
    two::{QuadRenderer, Rect, RenderList, Renderable},
    window, Game, GameLoader, Obj, RenderingContext, Texture,
};
use std::time::Duration;

struct Player {
    texture: Texture,
    position: Vec2,
    speed: f32,
}

impl Player {
    const SIZE: Vec2 = Vec2::new(66., 92.);
}

impl Renderable for Player {
    fn texture(&self) -> Option<&Texture> {
        Some(&self.texture)
    }
    fn rect(&self) -> Rect {
        Rect {
            position: self.position,
            size: Self::SIZE,
        }
    }
}

struct WasdGame {
    input_system: InputSystem,
    renderer: QuadRenderer,
    render_list: RenderList,
    player: Obj<Player>,
}

impl WasdGame {
    fn new(input_system: InputSystem, renderer: QuadRenderer, player_texture: Texture) -> Self {
        let player = Obj::new(Player {
            texture: player_texture,
            position: Vec2::from(window::screen_size()) / 2. - Player::SIZE / 2.,
            speed: 150.,
        });
        let mut render_list = RenderList::new();
        render_list.add(obj_upcast!(player));
        WasdGame {
            input_system,
            renderer,
            render_list,
            player,
        }
    }
}

impl Game for WasdGame {
    fn set_screen_size(&mut self, width: f32, height: f32) {
        self.renderer.set_screen_size(width, height);
    }

    fn handle_event(&mut self, event: InputEvent) {
        self.input_system.handle_event(event);
    }

    fn update(&mut self, frame_time: Duration) {
        let mut player = self.player.get_mut();
        let mut move_input = self.input_system.get("move").axis2_state();
        move_input.y *= -1.;
        let speed = player.speed;
        player.position += move_input * speed * frame_time.as_secs_f32();

        if self.input_system.get("exit").pressed() {
            window::request_quit();
        }

        self.input_system.end_frame();
    }

    fn render(&mut self, context: &mut RenderingContext) {
        self.renderer.queue_all(&mut self.render_list);
        self.renderer.render_pass(context);
    }
}

impl GameLoader for WasdGame {
    type Assets = (InputSystem, Texture);
    type Game = Self;

    fn fonts() -> Vec<&'static str> {
        vec!["OpenSans-Regular.ttf"]
    }

    fn create_default_files() -> asset::Result<()> {
        InputSystem::create_default_config_if_missing()
    }

    fn load(context: &mut RenderingContext) -> asset::Result<Self::Assets> {
        let input_system = InputSystem::load_config()?;
        let player_texture = asset::load_png_file(context, "images", "player.png")?;
        Ok((input_system, player_texture))
    }

    fn create_game(
        renderer: QuadRenderer,
        (input_system, player_texture): Self::Assets,
    ) -> Self::Game {
        WasdGame::new(input_system, renderer, player_texture)
    }
}

fn main() {
    gristmill::run_game::<WasdGame>("WASD Example");
}
