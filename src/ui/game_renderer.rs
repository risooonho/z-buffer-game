// © Copyright 2019-2020, Atamert Ölçgen
//
// This file is part of z-buffer-game.
//
// z-buffer-game is free software: you can redistribute it and/or modify it
// under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or (at your
// option) any later version.
//
// z-buffer-game is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY
// or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Lesser General Public
// License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with z-buffer-game.  If not, see <https://www.gnu.org/licenses/>.

use crate::data::{Location, Rectangle, VisibleObject};
use crate::stage::game::Game;
use crate::ui::constants::{
    BOTTOM_PANEL_HEIGHT, MAP_MIN_SIZE, SIDE_PANEL_WIDTH,
};
use crate::ui::render::Render;
use crate::ui::tile::{self, Tile};
use std::convert::TryFrom;
use std::fmt;
use tcod::colors;
use tcod::console::{blit, Console, Offscreen, TextAlignment};

pub struct GameRenderer {
    bottom_panel: Offscreen,
    map: Offscreen,
    root: Offscreen,
    side_panel: Offscreen,
}

impl GameRenderer {
    pub fn new(width: u32, height: u32) -> GameRenderer {
        let root = Offscreen::new(width as i32, height as i32);
        let (map_w, map_h) =
            Self::calculate_map_viewport_size((width, height)).unwrap();
        let map = Offscreen::new(map_w as i32, map_h as i32);
        let bottom_panel =
            Offscreen::new(width as i32, (height - map_h) as i32);
        let side_panel = Offscreen::new((width - map_w) as i32, map_h as i32);
        GameRenderer {
            bottom_panel,
            map,
            root,
            side_panel,
        }
    }

    pub(self) fn calculate_map_viewport_size(
        requested_size: (u32, u32),
    ) -> Option<(u32, u32)> {
        let (req_w, req_h) = requested_size;
        let (map_min_w, map_min_h) = MAP_MIN_SIZE;
        if (req_w >= map_min_w + SIDE_PANEL_WIDTH)
            && (req_h >= map_min_h + BOTTOM_PANEL_HEIGHT)
        {
            Some((req_w - SIDE_PANEL_WIDTH, req_h - BOTTOM_PANEL_HEIGHT))
        } else {
            None
        }
    }

    fn blit(&mut self) {
        let w = self.root.width();
        let h = self.root.height();
        let mw = self.map.width();
        let mh = self.map.height();
        blit(
            &self.map,
            (0, 0),
            (mw, mh),
            &mut self.root,
            (0, 0),
            1.0,
            1.0,
        );
        blit(
            &self.bottom_panel,
            (0, 0),
            (w, h - mh),
            &mut self.root,
            (0, mh),
            1.0,
            1.0,
        );
        blit(
            &self.side_panel,
            (0, 0),
            (w - mw, mh),
            &mut self.root,
            (mw, 0),
            1.0,
            1.0,
        );
    }
}

impl fmt::Debug for GameRenderer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GameRenderer")
    }
}

impl Render for GameRenderer {
    type StageType = Game;

    fn borrow_root(&self) -> &Offscreen {
        &self.root
    }

    fn update(&mut self, stage: &Game) {
        stage.with_scene_data(|scene_data| {
            let mut map = &self.map;
            let w = u16::try_from(map.width())
                .expect("Map window does not fit into u16");
            let h = u16::try_from(map.height())
                .expect("Map window does not fit into u16");

            // Fill the map with some glyph.
            for y in 0..map.height() {
                for x in 0..map.width() {
                    tile::MAP_BACKGROUND.put(&mut map, x, y, 0);
                }
            }

            let t: u64 = scene_data.t_millis();
            let boundaries =
                Rectangle::centered_around(scene_data.cursor_location(), w, h);
            scene_data.for_each_map_tile(
                |Location { x, y }, objects| {
                    for obj in objects {
                        tile::from_visible_object(*obj).put(
                            &mut map,
                            x - boundaries.min_x,
                            y - boundaries.min_y,
                            t,
                        );
                    }
                },
                boundaries,
            );
            {
                let Location { x: cx, y: cy } = scene_data.cursor_location();
                tile::CURSOR.put(
                    &mut map,
                    cx - boundaries.min_x,
                    cy - boundaries.min_y,
                    t,
                );
            }
        });

        stage.with_scene_data(|scene_data| {
            let mut bottom_panel = &self.bottom_panel;
            let w = bottom_panel.width();
            let h = bottom_panel.height();

            for y in 0..h {
                for x in 0..w {
                    tile::UI_BACKGROUND.put(&mut bottom_panel, x, y, 0);
                }
            }

            // TODO: Put the color in constants
            bottom_panel.set_default_foreground(colors::DARKEST_SEPIA);
            scene_data.for_each_game_log(5, |(idx, msg)| {
                bottom_panel.print_rect(0, idx as i32, w, 1, msg.contents());
            });
        });

        stage.with_scene_data(|scene_data| {
            let mut side_panel = &self.side_panel;
            let w = side_panel.width();
            let h = side_panel.height();

            for y in 0..h {
                for x in 0..w {
                    tile::UI_BACKGROUND.put(&mut side_panel, x, y, 0);
                }
            }

            side_panel.print_rect(
                w / 2,
                0,
                w,
                1,
                scene_data.get_game_time_str(),
            );

            let cursor_location = scene_data.cursor_location();

            // TODO: Put the color in constants
            side_panel.set_default_foreground(colors::DARKEST_SEPIA);
            side_panel.set_alignment(TextAlignment::Center);
            let s: String = format!(
                "[{: >4}:{: >4}]",
                cursor_location.x, cursor_location.y
            );
            side_panel.print_rect(w / 2, 1, w, 1, &s);

            let objects: Vec<VisibleObject> =
                scene_data.get_objects_for_location(&cursor_location);
            for (i, obj) in objects.iter().enumerate() {
                let s: String = format!("{:?}", obj);
                side_panel.print_rect(w / 2, h - i as i32 - 3, w, 1, &s);
            }
        });

        self.blit();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 4K display & 8 pixel glyphs, times 2
    const MAX_X: u32 = 3840 / 8 * 2;
    const MAX_Y: u32 = 2160 / 8 * 2;

    #[test]
    fn map_dimensions_are_never_less_than_minimum_requirements() {
        let (map_min_width, map_min_height) = MAP_MIN_SIZE;
        let min_width = map_min_width + SIDE_PANEL_WIDTH;
        let min_height = map_min_height + BOTTOM_PANEL_HEIGHT;

        for b in 0..MAX_Y {
            for a in 0..MAX_X {
                match GameRenderer::calculate_map_viewport_size((a, b)) {
                    Some((w, h)) => {
                        assert!(
                            w >= map_min_width,
                            "Map width {} is calculated as less than minimum required!",
                            w
                        );
                        assert_eq!(a, w + SIDE_PANEL_WIDTH);
                        assert!(
                            h >= map_min_height,
                            "Map height {} is calculated as less than minimum required!",
                            h
                        );
                        assert_eq!(b, h + BOTTOM_PANEL_HEIGHT);
                    }
                    None => assert!(a < min_width || b < min_height),
                }
            }
        }
    }
}
