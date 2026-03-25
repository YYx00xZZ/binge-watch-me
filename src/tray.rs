// binge-watch-me — a self-hosted media remote controlled from your phone
// Copyright (C) 2026  Aleksandar Parvanov
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use objc2::rc::Retained;
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSMenu, NSMenuItem, NSStatusBar,
    NSStatusItem, NSVariableStatusItemLength,
};
use objc2_foundation::{MainThreadMarker, NSString};

pub fn run<F: FnOnce()>(on_start: F) {
    let mtm = MainThreadMarker::new().expect("must be on main thread");

    // Initialize macOS application context
    let app = unsafe { NSApplication::sharedApplication(mtm) };
    unsafe {
        app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
    }

    // Create status bar item directly via NSStatusBar
    let status_bar = unsafe { NSStatusBar::systemStatusBar() };
    let status_item: Retained<NSStatusItem> = unsafe {
        status_bar.statusItemWithLength(NSVariableStatusItemLength)
    };

    // Set the title text — visible immediately, no icon needed
    if let Some(button) = unsafe { status_item.button(mtm) } {
        let title = NSString::from_str("BWM");
        unsafe { button.setTitle(&title) };
    }

    // Build the menu
    let menu = unsafe { NSMenu::new(mtm) };

    // Open remote item
    let open_title = NSString::from_str("Open remote");
    let open_action = objc2::sel!(openRemote);
    let open_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            mtm.alloc(),
            &open_title,
            Some(open_action),
            &NSString::from_str(""),
        )
    };
    unsafe { menu.addItem(&open_item) };

    // Separator
    unsafe { menu.addItem(&NSMenuItem::separatorItem(mtm)) };

    // Quit item
    let quit_title = NSString::from_str("Quit");
    let quit_action = objc2::sel!(terminate:);
    let quit_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            mtm.alloc(),
            &quit_title,
            Some(quit_action),
            &NSString::from_str("q"),
        )
    };
    unsafe { menu.addItem(&quit_item) };

    unsafe { status_item.setMenu(Some(&menu)) };

    tracing::info!("Status bar item created successfully");

    // Run startup logic
    on_start();

    // Run the macOS event loop — this blocks the main thread
    unsafe { app.run() };
}
