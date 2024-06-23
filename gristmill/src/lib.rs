pub mod asset;
pub mod input;
mod lang;
pub mod two;

pub use glam as math;
pub use grist::*;
pub use lang::tr;
pub use palette as color;

use miniquad::*;
use serde::{Deserialize, Serialize};
use std::{
    fs::OpenOptions,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tiny_game_loop::GameLoop;

use input::InputEvent;
use math::Vec2;
use two::QuadRenderer;

pub mod window {
    pub use miniquad::window::{order_quit, request_quit, screen_size};
}

pub type RenderingContext = Box<dyn RenderingBackend>;
pub use glyph_brush::ab_glyph::FontArc as Font;

static DROPPED_TEXTURES: Mutex<Vec<TextureId>> = Mutex::new(Vec::new());

#[derive(PartialEq, Eq, Hash)]
struct TextureHandle(TextureId);

impl Drop for TextureHandle {
    fn drop(&mut self) {
        let mut dropped = DROPPED_TEXTURES.lock().unwrap();
        dropped.push(self.0);
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Texture(Arc<TextureHandle>);

impl Texture {
    pub fn new_rgba8(
        context: &mut RenderingContext,
        width: u16,
        height: u16,
        bytes: &[u8],
    ) -> Self {
        let handle = TextureHandle(context.new_texture_from_rgba8(width, height, bytes));
        Texture(Arc::new(handle))
    }
    pub fn new_invalid(context: &mut RenderingContext) -> Self {
        Self::new_rgba8(context, 1, 1, &[255, 0, 255, 255])
    }

    pub fn id(&self) -> TextureId {
        self.0 .0
    }
}

#[derive(Serialize, Deserialize)]
struct WindowConfig {
    width: u32,
    height: u32,
    fullscreen: bool,
    fps: u32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            fullscreen: false,
            fps: 60,
        }
    }
}

impl WindowConfig {
    const FILENAME: &'static str = "window.yaml";
    fn create_default_config_if_missing() -> asset::Result<()> {
        let path = asset::get_path("config", Self::FILENAME);
        if path.exists() {
            return Ok(());
        }
        println!("Creating default file {}", path.to_string_lossy());
        asset::save_yaml_file("config", Self::FILENAME, &WindowConfig::default())
    }
    fn load_config() -> asset::Result<Self> {
        asset::load_yaml_file("config", Self::FILENAME)
    }
}

pub trait Game: Sized + 'static {
    fn set_screen_size(&mut self, width: f32, height: f32);
    fn handle_event(&mut self, event: InputEvent);
    fn quit_requested(&mut self) -> bool {
        true
    }
    fn update(&mut self, frame_time: Duration);
    fn render(&mut self, context: &mut RenderingContext);
}

pub trait GameLoader: 'static {
    type Assets;
    type Game: Game;
    fn fonts() -> Vec<&'static str>;
    fn create_default_files() -> asset::Result<()>;
    fn load(context: &mut RenderingContext) -> asset::Result<Self::Assets>;
    fn create_game(renderer: QuadRenderer, assets: Self::Assets) -> Self::Game;
}

struct Stage<G: Game> {
    context: RenderingContext,
    game_loop: GameLoop,
    time: Instant,
    game: G,
}

impl<G: Game> Stage<G> {
    fn new(mut game: G, context: RenderingContext, window_config: WindowConfig) -> Self {
        game.set_screen_size(window_config.width as f32, window_config.height as f32);
        Stage {
            context,
            game_loop: GameLoop::new_with_fps(window_config.fps, Duration::from_millis(250)),
            time: Instant::now(),
            game,
        }
    }
}

impl<G: Game> EventHandler for Stage<G> {
    fn update(&mut self) {
        let elapsed = self.time.elapsed();
        self.time = Instant::now();
        let update = self.game_loop.update(elapsed);
        if update.num_updates > 0 {
            update.run(|update| self.game.update(update.frame_time));
        } else {
            // Limit framerate
            std::thread::sleep(update.frame_time - elapsed);
        }
    }

    fn draw(&mut self) {
        {
            let mut dropped_textures = DROPPED_TEXTURES.lock().unwrap();
            for texture in dropped_textures.drain(..) {
                self.context.delete_texture(texture);
            }
        }

        self.game.render(&mut self.context);
        self.context.commit_frame();
    }

    fn quit_requested_event(&mut self) {
        if !self.game.quit_requested() {
            miniquad::window::cancel_quit();
        }
    }

    fn resize_event(&mut self, width: f32, height: f32) {
        self.game.set_screen_size(width, height);
    }

    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        self.game.handle_event(InputEvent::MouseMotion {
            position: Vec2::new(x, y),
        });
    }
    fn raw_mouse_motion(&mut self, dx: f32, dy: f32) {
        self.game.handle_event(InputEvent::RawMouseMotion {
            delta: Vec2::new(dx, dy),
        });
    }
    fn mouse_wheel_event(&mut self, _x: f32, _y: f32) {
        // TODO
    }
    fn mouse_button_down_event(&mut self, button: MouseButton, _x: f32, _y: f32) {
        if let Ok(button) = button.try_into() {
            self.game.handle_event(InputEvent::MouseButton {
                button,
                pressed: true,
            });
        }
    }
    fn mouse_button_up_event(&mut self, button: MouseButton, _x: f32, _y: f32) {
        if let Ok(button) = button.try_into() {
            self.game.handle_event(InputEvent::MouseButton {
                button,
                pressed: false,
            });
        }
    }

    fn key_down_event(&mut self, keycode: KeyCode, _keymods: KeyMods, _repeat: bool) {
        self.game.handle_event(InputEvent::Key {
            key: keycode,
            pressed: true,
        });
    }
    fn key_up_event(&mut self, keycode: KeyCode, _keymods: KeyMods) {
        self.game.handle_event(InputEvent::Key {
            key: keycode,
            pressed: false,
        });
    }
    fn char_event(&mut self, _character: char, _keymods: KeyMods, _repeat: bool) {
        // TODO
    }
}

fn load_stage1<G: GameLoader>() -> asset::Result<(WindowConfig, Vec<Font>)> {
    println!("{}", console::style("Loading game (stage 1)").bold());

    #[cfg(debug_assertions)]
    {
        asset::create_dir("config");
        let mut lang_dir = asset::base_path();
        lang_dir.push("lang");
        if !lang_dir.exists() {
            println!("Creating directory {}", lang_dir.to_string_lossy());
            std::fs::create_dir(&lang_dir).expect("could not create lang directory");
            lang_dir.push("en.yaml");
            println!("Creating empty file {}", lang_dir.to_string_lossy());
            std::fs::write(lang_dir, "").expect("could not create lang file");
        }
        WindowConfig::create_default_config_if_missing()?;
        G::create_default_files()?;
    }

    let window_config = WindowConfig::load_config()?;
    lang::load_translations()?;
    let mut fonts = Vec::new();
    for font_file in G::fonts() {
        fonts.push(asset::load_font_file("fonts", font_file)?);
    }
    Ok((window_config, fonts))
}

fn load_stage2<G: GameLoader>(context: &mut RenderingContext) -> asset::Result<G::Assets> {
    println!("{}", console::style("Loading game (stage 2)").bold());
    G::load(context)
}

fn create_game<G: GameLoader>(
    renderer: QuadRenderer,
    assets: G::Assets,
    screen_size: (f32, f32),
) -> G::Game {
    println!("{}", console::style("Starting game loop").bold());
    let mut game = G::create_game(renderer, assets);
    game.set_screen_size(screen_size.0, screen_size.1);
    game
}

fn error_log_path() -> PathBuf {
    let mut path = asset::base_path();
    path.push("error.log");
    path
}

fn append_error_log(message: String) {
    use std::io::Write;
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(error_log_path())
    {
        let _ = writeln!(file, "{}\n", message);
    }
}

#[track_caller]
pub fn nonfatal_error(message: &str) {
    append_error_log(format!(
        "nonfatal at {}:\n{}",
        std::panic::Location::caller(),
        message
    ));
    println!(
        "{}",
        console::style("A nonfatal error occurred. See error.log for details.").red()
    );
    let _ = msgbox::create("Error", message, msgbox::IconType::Error);
}

pub trait ResultExt<T> {
    fn unwrap_nonfatal(self) -> T;
}

impl<T: Default, E: std::error::Error> ResultExt<T> for Result<T, E> {
    #[track_caller]
    fn unwrap_nonfatal(self) -> T {
        match self {
            Ok(value) => value,
            Err(error) => {
                nonfatal_error(&error.to_string());
                Default::default()
            }
        }
    }
}

fn panic_handler(panic_info: &std::panic::PanicInfo) {
    append_error_log(panic_info.to_string());
    println!(
        "{}",
        console::style("A fatal error occurred. See error.log for details.").red()
    );
    let payload = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
        *s
    } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
        s
    } else {
        "An unknown error occured"
    };
    let message = payload
        .strip_prefix("called `Result::unwrap()` on an `Err` value: ")
        .unwrap_or(payload);
    let _ = msgbox::create("Fatal Error", message, msgbox::IconType::Error);
}

pub fn run_game<G: GameLoader>(window_title: &str) {
    let _ = std::fs::remove_file(error_log_path());
    std::panic::set_hook(Box::new(panic_handler));
    let (window_config, fonts) = load_stage1::<G>().unwrap();
    let config = conf::Conf {
        window_title: window_title.to_string(),
        window_width: window_config.width.try_into().unwrap(),
        window_height: window_config.height.try_into().unwrap(),
        fullscreen: window_config.fullscreen,
        window_resizable: false,
        ..Default::default()
    };
    let screen_size = (window_config.width as f32, window_config.height as f32);
    miniquad::start(config, move || {
        let mut context = miniquad::window::new_rendering_backend();
        let assets = load_stage2::<G>(&mut context).unwrap();
        let renderer = QuadRenderer::new(&mut context, fonts, true);
        let game = create_game::<G>(renderer, assets, screen_size);
        Box::new(Stage::new(game, context, window_config))
    });
}
