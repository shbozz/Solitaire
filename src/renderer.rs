/* renderer.rs
 *
 * Copyright 2025 Shbozz
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use adw::subclass::prelude::*;
use adw::prelude::*;
use cairo::Context;
use gtk::gdk::*;
use rsvg::{CairoRenderer, Loader};
use crate::card_stack::*;

// We can't afford to use safe variables here,
// we are reading (& writing) multiple times a second
static mut GAME_HEIGHT: i32 = 0;
static mut GAME_WIDTH: i32 = 0;
static mut TICK_CALLBACK_ID: Option<gtk::TickCallbackId> = None;

pub const ASPECT:f32 = 1.4;
pub fn draw_card(name: &str, renderer: &CairoRenderer) -> MemoryTexture {
    let surface = cairo::ImageSurface::
        create(cairo::Format::ARgb32, 250, 350)
        .expect("Couldn't create surface");
    let cr = Context::new(&surface).expect("Couldn't create cairo context");
    // Render a single SVG layer, marked by a <g>
    renderer
        .render_element(&cr, Some(&format!("#{name}")), &cairo::Rectangle::new(0.0, 0.0, 250.0, 350.0))
        .expect(&format!("Failed to render layer {name}"));

    drop(cr);
    let stride = surface.stride() as usize;
    let data = surface.take_data().expect("Failed to get data from surface");
    // Create a texture from the surface
    let bytes = glib::Bytes::from(&data[..]);
    MemoryTexture::new(
        250,
        350,
        MemoryFormat::B8g8r8a8Premultiplied, // Match ARGB32 surface
        &bytes,
        stride,
    )
}

pub fn flip_card(card: &gtk::Image) {
    // There has to be a better way to do this
    glib::g_message!("solitaire", "Loading SVG");
    let resource = gio::resources_lookup_data("/org/gnome/Solitaire/assets/minimum_dark.svg", gio::ResourceLookupFlags::NONE)
        .expect("Failed to load resource data");
    glib::g_message!("solitaire", "loaded resource data");
    let handle = Loader::new()
        .read_stream(&gio::MemoryInputStream::from_bytes(&resource), None::<&gio::File>, None::<&gio::Cancellable>)
        .expect("Failed to load SVG");
    let renderer = rsvg::CairoRenderer::new(&handle);
    glib::g_message!("solitaire", "Done Loading SVG");
    
    // It's pretty simple, the state is stored in the widget name
    let current_name = card.widget_name();
    if current_name.contains("_b") {
        card.set_widget_name(&current_name.replace("_b", ""));
        let texture = draw_card(&card.widget_name(), &renderer);
        card.set_paintable(Some(texture.upcast_ref::<Paintable>()));
    }
    else {
        card.set_widget_name((current_name.to_string() + "_b").as_str());
        let texture = draw_card("back", &renderer);
        card.set_paintable(Some(texture.upcast_ref::<Paintable>()));
    }
}

fn update_geometry(grid: &gtk::Grid, height: i32, width: i32) {
    const NUM_COLS: i32 = 7;
    const NUM_ROWS: i32 = 2;
    const CARD_HEIGHTS_NEEDED: i32 = 6;
    const SCREEN_USAGE_RATIO: i32 = 9;  // 90% usage becomes 9/10

    // Calculate dimensions in fewer operations
    let available_width = width * SCREEN_USAGE_RATIO / 10;
    let available_height = height * SCREEN_USAGE_RATIO / 10;

    // Calculate maximum height based on constraints
    let max_height_by_width = (available_width / NUM_COLS) as f32 * ASPECT;
    let max_height_by_height = available_height / CARD_HEIGHTS_NEEDED;

    // Use the more constraining dimension
    let card_height = std::cmp::min(max_height_by_width as i32, max_height_by_height);
    let card_width = (card_height as f32 / ASPECT) as i32;
    let tableau_row_height = card_height * 3;

    // Process each stack widget in the grid
    let stacks = grid.observe_children();
    let mut stack_index = 0;
    for _i in 0..NUM_ROWS {
        for _j in 0..NUM_COLS {
            if let Some(object) = stacks.item(stack_index as u32) {
                if let Ok(stack) = object.downcast::<CardStack>() {
                    // Apply calculated dimensions
                    stack.imp().size_allocate(card_width, tableau_row_height, 0);
                    stack_index += 1;
                }
            }
        }
    }
}

pub fn setup_resize(card_grid: &gtk::Grid) {
    let window = card_grid.root().expect("Couldn't get window");
    unsafe {
        let tick_callback = card_grid.add_tick_callback(move |grid, _frame| {
            let mut do_update = false;
            let height = grid.height();
            let width = window.width();
            if height != GAME_HEIGHT {
                GAME_HEIGHT = height;
                do_update = true;
            }
            if width != GAME_WIDTH {
                GAME_WIDTH = width;
                do_update = true;
            }
            if do_update {
                update_geometry(&grid, height, width);
            }
            glib::ControlFlow::Continue
        });
        TICK_CALLBACK_ID = Some(tick_callback);
    }
}

pub fn unregister_resize() {
    unsafe {
        if let Some(id) = TICK_CALLBACK_ID.take() { // Maybe a raw pointer would work here
            id.remove();
        }
        GAME_HEIGHT = 0;
        GAME_WIDTH = 0;
    }
}