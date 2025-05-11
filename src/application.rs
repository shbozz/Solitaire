/* application.rs
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

use gettextrs::gettext;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};
use crate::config::VERSION;
use crate::SolitaireWindow;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SolitaireApplication {}

    #[glib::object_subclass]
    impl ObjectSubclass for SolitaireApplication {
        const NAME: &'static str = "SolitaireApplication";
        type Type = super::SolitaireApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for SolitaireApplication {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_gactions();
            obj.set_accels_for_action("app.quit", &["<primary>q"]);
            obj.set_accels_for_action("win.hint", &["<primary>h"]);
            obj.set_accels_for_action("win.redo", &["<primary><shift>z"]);
            obj.set_accels_for_action("win.undo", &["<primary>z"]);
        }
    }

    impl ApplicationImpl for SolitaireApplication {
        // We connect to the activate callback to create a window when the application
        // has been launched. Additionally, this callback notifies us when the user
        // tries to launch a "second instance" of the application. When they try
        // to do that, we'll just present any existing window.
        fn activate(&self) {
            let application = self.obj();
            // Get the current window or create one if necessary
            let window = application.active_window().unwrap_or_else(|| {
                let window = SolitaireWindow::new(&*application);
                SolitaireWindow::draw_init(&window);
                window.upcast()
            });

            // Ask the window manager/compositor to present the window
            window.present();
        }
    }

    impl GtkApplicationImpl for SolitaireApplication {}
    impl AdwApplicationImpl for SolitaireApplication {}
}

glib::wrapper! {
    pub struct SolitaireApplication(ObjectSubclass<imp::SolitaireApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl SolitaireApplication {
    pub fn new(application_id: &str, flags: &gio::ApplicationFlags) -> Self {
        glib::Object::builder()
            .property("application-id", application_id)
            .property("flags", flags)
            .property("resource-base-path", "/io/github/shbozz/Solitaire")
            .build()
    }

    fn setup_gactions(&self) {
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| app.quit())
            .build();
        let about_action = gio::ActionEntry::builder("about")
            .activate(move |app: &Self, _, _| app.show_about())
            .build();
        self.add_action_entries([quit_action, about_action]);
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();
        let about = adw::AboutDialog::builder()
            .application_name("Solitaire")
            .application_icon("io.github.shbozz.Solitaire")
            .developer_name("Shbozz")
            .version(VERSION)
            .developers(vec!["Shbozz"])
            .artists(vec!["Vincent Bermel", "Shbozz"])
            // Translators: Replace "translator-credits" with your name/username, and optionally an email or URL.
            .translator_credits(&gettext("translator-credits"))
            .copyright("© 2025 Shbozz \
                        \n© 2024 Vincent Bermel")
            .license_type(gtk::License::Gpl30)
            .issue_url("https://gitlab.gnome.org/shbozz/solitaire/-/issues")
            .website("https://shbozz.github.io/Solitaire")
            .comments("Solitaire is a simple collection of card games.")
            .build();

        about.present(Some(&window));
    }
}