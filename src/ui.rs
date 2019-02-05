//! User interface.

use crate::asset;
use crate::conf;
use crate::stage::Stage;
use crate::ui::game_renderer::GameRenderer;
use crate::ui::main_menu_renderer::MainMenuRenderer;
use log::debug;
use std::mem::{discriminant, Discriminant};
use tcod::console::{self, Root};
use tcod::system::get_fps;

mod main_menu_renderer;

/// User interface related data
pub struct UI {
    fps: u32,
    root_console: Root,
    renderer: Option<(Discriminant<Stage>, Renderer)>,
}

impl UI {
    pub fn new() -> UI {
        let font_file = asset::Assets::FontTerminal16x16GsRo.extract().unwrap();

        let root = console::Root::initializer()
            .title(conf::window_title())
            .size(
                conf::screen_width_char() as i32,
                conf::screen_height_char() as i32,
            )
            .font(font_file, console::FontLayout::AsciiInRow)
            .init();

        tcod::system::set_fps(conf::max_fps() as i32);

        UI {
            root_console: root,
            fps: 0,
            renderer: None,
        }
    }

    /// Render UI based on the current stage.
    pub fn draw(&mut self, stage: &Stage) {
        self.fps = get_fps() as u32;

        if self.is_stage_changed(stage) {
            let renderer = match stage {
                Stage::Game(_) => Renderer::Game,
                Stage::MainMenu(_) => Renderer::MainMenu(MainMenuRenderer::new()),
            };
            debug!("Updating renderer as {:?}.", &renderer);
            self.renderer = Some((discriminant(stage), renderer));
        }

        match &stage {
            Stage::Game(_) => {
                let mut renderer = GameRenderer::new();
                let root: &mut Root = &mut self.root_console;
                renderer.blit(root);
                root.flush();
            }
            Stage::MainMenu(m) => {
                let renderer: &mut MainMenuRenderer = match self.renderer {
                    Some((_, Renderer::MainMenu(ref mut r))) => r,
                    _ => unreachable!(),
                };
                renderer.update(m);
                let root: &mut Root = &mut self.root_console;
                renderer.blit(root);
                root.flush();
            }
        };
    }

    pub fn is_running(&self) -> bool {
        !self.root_console.window_closed()
    }

    #[inline]
    fn is_stage_changed(&self, stage: &Stage) -> bool {
        self.renderer
            .as_ref()
            .map_or(true, |(d, _)| *d != discriminant(stage))
    }
}

/// Since [Stage](crate::stage::Stage) is an `enum` and dependency is from
/// [ui](crate::ui) to [stage](crate::stage) we have Renderer as an enum to
/// match its structure.
// TODO: Consider using a trait for the wrapped values in the variants.
#[derive(Debug)]
enum Renderer {
    Game,
    MainMenu(MainMenuRenderer),
}

// TODO: Instead of passing root into the renderer, get offscreen console and
// blit it onto the root within UI.
mod game_renderer {
    use crate::conf;
    use tcod::console::{self, Console, Root};

    pub struct GameRenderer {}

    impl GameRenderer {
        pub fn new() -> GameRenderer {
            GameRenderer {}
        }

        pub fn blit(&mut self, root: &mut Root) {
            root.clear();
            root.set_alignment(console::TextAlignment::Center);
            root.print_rect(
                (conf::screen_width_char() / 2) as i32,
                (conf::screen_height_char() / 2 + 2) as i32,
                conf::screen_width_char() as i32,
                1,
                "Game Stage",
            );
            root.flush();
        }
    }
}
