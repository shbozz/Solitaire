/* games.rs
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
use std::sync::Mutex;
use gtk::prelude::*;
use gtk::{DragSource, gdk::DragAction, gdk, glib, GestureClick};
use crate::card_stack::*;
use crate::renderer;

pub const JOKERS: [&str; 2] = ["joker_red", "joker_black"];
pub const SUITES: [&str; 4] = ["club", "diamond", "heart", "spade"];
pub const RANKS: [&str; 13] = ["ace", "2", "3", "4", "5", "6", "7", "8", "9", "10", "jack", "queen", "king"];
pub const GAMES: [&str; 6] = ["Klondike", "Spider", "FreeCell", "Tri-Peaks", "Pyramid", "Yukon"];
static CURRENT_GAME: Mutex<String> = Mutex::new(String::new());

// Links to all the included games
pub fn load_game(game: &str, grid: &gtk::Grid) {
    
    // Get children from the grid
    let children = grid.observe_children();

    for i in 0..14 {
        // Create a new card stack for this position
        let card_stack = CardStack::new();
        card_stack.set_overflow(gtk::Overflow::Hidden);

        // Calculate layout position
        let row = i / 7;
        let col = i % 7;

        // Add cards to the stack, reusing available images
        for _j in 0..4 {
            // Always get the first item from the collection (index 0)
            // as the collection shifts when items are removed
            if let Some(obj) = children.item(0) {
                if let Ok(image) = obj.downcast::<gtk::Image>() {
                    grid.remove(&image);
                    card_stack.add_card(&image);
                    add_drag_to_card(&image);
                    connect_click(&image);
                }
            } else {
                glib::g_error!("solitaire", "Failed to get child from grid");
            }
        }

        // Enable drag and drop for gameplay
        card_stack.enable_drop();

        // Attach the card stack to the grid at the calculated position
        grid.attach(&card_stack, col, row, 1, 1);
    }

    // Store the current game type
    let mut game_string = CURRENT_GAME.lock().unwrap();
    *game_string = game.to_string();

    // setup the resize handler for responsive layout
    renderer::setup_resize(grid);

    // Log game loading
    println!("Loaded game: {}", game);
}

pub fn unload(_grid: &gtk::Grid) {
    CURRENT_GAME.lock().unwrap().clear();
    renderer::unregister_resize();
}

pub fn load_recent() {

}

// pub fn get_current_game() -> String {
//     CURRENT_GAME.lock().unwrap().clone()
// }

pub fn add_drag_to_card(card: &gtk::Image) {
    let drag_source = DragSource::builder()
        .actions(DragAction::MOVE)  // allow moving the stack
        .build();

    let card_clone = card.clone();
    drag_source.connect_prepare(move |src, _, _| {
        let stack = card_clone.parent().unwrap().downcast::<CardStack>().unwrap();
        let move_stack = stack.split_to_new_on(&*card_clone.widget_name());
        // Convert the CardStack (a GObject) into a GValue, then a ContentProvider.
        let value = move_stack.upcast::<glib::Object>().to_value();
        let provider = gdk::ContentProvider::for_value(&value);
        src.set_content(Some(&provider));  // attach the data provider
        Some(provider)  // must return Some(provider) from prepare
    });

    drag_source.connect_drag_begin(|src, drag| {
        let icon = gtk::DragIcon::for_drag(drag);
        let provider = src.content().unwrap();
        let value = provider.value(glib::Type::OBJECT).unwrap();
        // I'd rather have no DnD icon instead of a crash
        if let Ok(obj) = value.get::<glib::Object>() {
            if let Ok(original_stack) = obj.downcast::<CardStack>() {
                let stack_clone = original_stack.clone();
                icon.set_child(Some(&stack_clone));
                let width = stack_clone.first_child().unwrap().width_request(); // HACK
                println!("width: {} height: {}", width, width * 6);
                stack_clone.imp().size_allocate(width, width * 6, 0);
                // width * 6 is arbitrary, if you have anything better, tell me!
            }
        }
    });
    
    card.add_controller(drag_source);
}

fn connect_click(card: &gtk::Image) {
    let click = GestureClick::new();
    let card_clone = card.clone();
    click.connect_released(move |_click, n_press, _x, _y| {
        if n_press == 1{
            renderer::flip_card(&card_clone);    
        } else if n_press == 2 {
            glib::g_message!("solitaire", "double click")
        } else { 
            return;
        }
    });
    card.add_controller(click);
}