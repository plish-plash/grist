use gristmill::{
    asset,
    input::{InputEvent, InputSystem},
    two::QuadRenderer,
    Event, Game, GameLoader, Obj, RenderingContext,
};
use silica::{
    taffy::prelude::*,
    view::{
        button::{Button, SimpleButtonView},
        label::Label,
    },
    Gui,
};
use std::time::Duration;

struct IntModel {
    value: i32,
    changed: Event<i32>,
}

struct ButtonGame {
    input_system: InputSystem,
    renderer: QuadRenderer,
    gui: Gui,
}

impl ButtonGame {
    fn new(input_system: InputSystem, renderer: QuadRenderer) -> Self {
        let mut gui = Gui::new();
        let root = gui.root();
        gui.set_style(
            root,
            Style {
                flex_direction: FlexDirection::Row,
                align_items: Some(AlignItems::Start),
                padding: Rect::length(64.0),
                gap: Size::length(16.0),
                ..Default::default()
            },
        );

        let mut times_clicked = IntModel {
            value: 0,
            changed: Event::new(),
        };

        let label = Obj::new(Label::new());
        let label1 = label.clone();
        times_clicked.changed.add_listener(move |value| {
            label1
                .get_mut()
                .set_text(format!("Times Clicked: {}", *value));
        });

        let mut button = Button::with_label("Click Me!", SimpleButtonView::default());
        button.add_pressed_listener(move |&()| {
            times_clicked.value += 1;
            times_clicked.changed.emit(&times_clicked.value);
        });
        gui.add_view_control(
            root,
            Obj::new(button),
            Style {
                size: Size::from_lengths(128., 32.),
                ..Default::default()
            },
        );

        gui.add_view(
            root,
            label,
            Style {
                size: Size::from_lengths(256., 32.),
                ..Default::default()
            },
        );

        ButtonGame {
            input_system,
            renderer,
            gui,
        }
    }
}

impl Game for ButtonGame {
    fn set_screen_size(&mut self, width: f32, height: f32) {
        self.renderer.set_screen_size(width, height);
        self.gui.set_screen_size(width, height);
    }

    fn handle_event(&mut self, event: InputEvent) {
        self.input_system.handle_event(event);
    }

    fn update(&mut self, _frame_time: Duration) {
        let pointer = self.input_system.pointer();
        self.gui
            .handle_pointer_motion(pointer.position.x, pointer.position.y);
        self.gui.handle_pointer_button(pointer.primary);

        if self.input_system.get("exit").pressed() {
            gristmill::window::request_quit();
        }

        self.input_system.end_frame();
    }

    fn render(&mut self, context: &mut RenderingContext) {
        self.gui.render(&mut self.renderer);
        self.renderer.render_pass(context);
    }
}

impl GameLoader for ButtonGame {
    type Assets = InputSystem;
    type Game = Self;

    fn fonts() -> Vec<&'static str> {
        vec!["OpenSans-Regular.ttf"]
    }

    fn create_default_files() -> asset::Result<()> {
        InputSystem::create_default_config_if_missing()
    }

    fn load(_context: &mut RenderingContext) -> asset::Result<Self::Assets> {
        Ok(InputSystem::load_config()?)
    }

    fn create_game(renderer: QuadRenderer, input_system: Self::Assets) -> Self::Game {
        ButtonGame::new(input_system, renderer)
    }
}

fn main() {
    gristmill::run_game::<ButtonGame>("Button Example");
}
