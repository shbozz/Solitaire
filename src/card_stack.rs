/* card_stack.rs
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
use adw::gio::ListModel;
use gtk::{glib, gdk};
use adw::prelude::*;
use adw::subclass::prelude::*;
use std::cell::Cell;
use crate::renderer;

glib::wrapper! {
    pub struct CardStack(ObjectSubclass<imp::CardStack>)
        @extends gtk::Fixed, gtk::Widget;
}

pub fn get_index(card_name: &str, children: &ListModel) -> Result<u32, glib::Error> {
    // Attempt to locate the child with the given card name
    let total_children = children.n_items();

    // Loop through all the children widgets to find the matching card
    for i in 0..total_children {
        let child = children.item(i).expect("Failed to get child from CardStack");
        let image = child.downcast::<gtk::Image>().expect("Child is not a gtk::Image (find)");
        if image.widget_name() == card_name {
            return Ok(i);
        }
    }

    Err(glib::Error::new(glib::FileError::Exist, format!("Card named '{}' was not found in the stack.", card_name).as_str()))
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct CardStack {
        pub is_stackable: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CardStack {
        const NAME: &'static str = "CardStack";
        type Type = super::CardStack;
        type ParentType = gtk::Fixed;
    }
    impl ObjectImpl for CardStack {}
    impl WidgetImpl for CardStack {
        fn size_allocate(&self, width: i32, height: i32, _baseline: i32) {
            //self.parent_size_allocate(width, height, baseline);
            let widget = self.obj();
            let children = widget.observe_children();
            let child_count = children.n_items();
            // Don't bother with empty stacks
            if child_count == 0 {
                return;
            }
            
            if child_count == 1 {
                widget.first_child().unwrap().set_width_request(width);
                return;
            }
            
            let card_height = (width as f32 * renderer::ASPECT).floor() as i32; // Use floor() because the lower height means spacing is not messed up
            let vertical_offset = std::cmp::min((height - card_height) / (child_count as i32 - 1), card_height / 2) as u32;

            // Position each card with proper spacing
            for i in 0..child_count {
                if let Some(child) = children.item(i) {
                    if let Ok(image) = child.downcast::<gtk::Image>() {
                        // Set the explicit size request for the image
                        image.set_size_request(width, card_height);
                        
                        // Position the card vertically with the proper offset
                        // The formula ensures cards are properly staggered with the calculated offset
                        let y_pos = (i * vertical_offset) as f64;
                        widget.move_(&image, 0.0, y_pos);
                    }
                }
            }
            widget.set_size_request(width, height);
        }
    }
    impl FixedImpl for CardStack {}
}

impl CardStack {
    pub fn new() -> Self {
        glib::Object::new()
    }
    // Most stack methods could use these
    pub fn get_card(&self, card_name: &str) -> Result<gtk::Image, glib::Error> {
        // Attempt to locate the child with the given card name
        let children = self.observe_children();
        let total_children = children.n_items();

        // Loop through all the children widgets to find the matching card
        for i in 0..total_children {
            let child = children.item(i).expect("Failed to get child from CardStack");
            let image = child.downcast::<gtk::Image>().expect("Child is not a gtk::Image (find)");
            if image.widget_name() == card_name {
                return Ok(image);
            }
        }

        Err(glib::Error::new(glib::FileError::Exist, format!("Card named '{}' was not found in the stack.", card_name).as_str()))
    }

    pub fn enable_drop(&self) {
        let drop_target = gtk::DropTarget::new(glib::Type::OBJECT, gdk::DragAction::MOVE);
        //drop_target.set_highlight(false); or something, the highlighting isn't ideal 
        let stack_clone = self.clone();
        drop_target.connect_drop(move |_, val, _, _| {
            let Ok(drop_stack) = val.get::<CardStack>() else {
                glib::g_error!("Tried to drop a non-CardStack onto a CardStack", "Solitaire");
                return false;
            };
            stack_clone.merge_stack(&drop_stack);
            true
        });
        self.add_controller(drop_target);
    }

    // FIXME this causes "Broken accounting of active state for widget" when the top card is moved
    pub fn split_to_new_on(&self, card_name: &str) -> CardStack {
        // Attempt to locate the child with the given card name
        let children = self.observe_children();
        let total_children = children.n_items();
        let new_stack = CardStack::new();

        // First, find the starting index
        let start_index = get_index(card_name, &children).expect("Couldn't get card");
        for _i in start_index..total_children {
            let child = children.item(start_index).expect("Failed to get child from CardStack");
            let image = child.downcast::<gtk::Image>().expect("Child is not a gtk::Image (split:1)");
            self.remove(&image);
            new_stack.add_card(&image);
        }
        self.imp().size_allocate(self.width(), self.height(), self.baseline());
        
        new_stack
    }

    pub fn merge_stack(&self, stack: &CardStack) {
        let items = stack.observe_children().n_items();
        for _i in 0..items {
            let child = stack.first_child().expect("Failed to get first child from CardStack");
            let image = child.downcast::<gtk::Image>().expect("Child is not a gtk::Image (merge)");
            stack.remove(&image);
            self.add_card(&image);
        }
        self.imp().size_allocate(self.width(), self.height(), self.baseline());
        stack.unrealize();
    }

    pub fn add_card(&self, card_image: &gtk::Image) {
        // Only add the image if it doesn't already have a parent
        if card_image.parent().is_none() {
            self.put(card_image, 0.0, 0.0);
        } else {
            // If the image already has a parent, log a warning
            eprintln!("Warning: Attempted to add a widget that already has a parent");
        }
    }
    
    pub fn dissolve_to_row(self, grid: &gtk::Grid, row: i32) {
        let items = self.observe_children().n_items();
        for i in 0..items {
            let child = self.first_child().expect("Failed to get first child from CardStack");
            let image = child.downcast::<gtk::Image>().expect("Child is not a gtk::Image (dissolve)");
            self.remove(&image);
            grid.attach(&image, i as i32, row, 1, 1);
            image.set_height_request(1);
        }
        grid.remove(&self);
        self.unrealize();
    }
    
    pub fn focus_card(&self, card_name: &str) {
        self.get_card(card_name).expect("Couldn't get card").grab_focus();
    }
}
