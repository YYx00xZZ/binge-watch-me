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

use std::cell::RefCell;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, OnceLock,
};

use objc2::{define_class, msg_send, rc::Retained, runtime::AnyObject, MainThreadOnly};
use objc2_app_kit::{
    NSAlert, NSAlertFirstButtonReturn, NSApplication, NSApplicationActivationPolicy, NSMenu,
    NSMenuItem, NSStatusBar, NSStatusItem, NSVariableStatusItemLength,
};
use objc2_foundation::{MainThreadMarker, NSObject, NSObjectProtocol, NSString, NSTimer};

use crate::updater;

// ---------------------------------------------------------------------------
// Main-thread-only storage for ObjC objects accessed by the timer delegate.
// Safe to use as thread_local because NSApp.run() never migrates the main
// thread, and all accesses are from the main thread only.
// ---------------------------------------------------------------------------

thread_local! {
    static TRAY_MENU: RefCell<Option<Retained<NSMenu>>> = RefCell::new(None);
    static UPDATE_MENU_ITEM: RefCell<Option<Retained<NSMenuItem>>> = RefCell::new(None);
    static STATUS_ITEM_REF: RefCell<Option<Retained<NSStatusItem>>> = RefCell::new(None);
}

/// Written by the background update checker; read by the main-thread timer.
static UPDATE_SLOT: OnceLock<Arc<Mutex<Option<updater::UpdateInfo>>>> = OnceLock::new();

/// Set to true once the user confirms an update. The timer shows
/// "Downloading…" in the status bar until the process exits.
static DOWNLOADING: AtomicBool = AtomicBool::new(false);

// ---------------------------------------------------------------------------
// TrayDelegate — ObjC class that handles timer callbacks and menu actions.
// ---------------------------------------------------------------------------

/// Placeholder ivars (no real state needed — everything is in statics/thread-locals).
struct TrayDelegateIvars;

define_class!(
    // SAFETY:
    // - NSObject has no subclassing requirements.
    // - TrayDelegate does not implement Drop.
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[name = "BingeWatchMeTrayDelegate"]
    #[ivars = TrayDelegateIvars]
    struct TrayDelegate;

    // SAFETY: NSObjectProtocol has no safety requirements.
    unsafe impl NSObjectProtocol for TrayDelegate {}

    impl TrayDelegate {
        /// Called by NSTimer every 5 seconds on the main thread.
        /// Adds the "Update available" menu item when an update is found,
        /// and updates the status bar title while a download is in progress.
        #[unsafe(method(checkTrayState:))]
        fn check_tray_state(&self, _sender: &AnyObject) {
            if DOWNLOADING.load(Ordering::Relaxed) {
                STATUS_ITEM_REF.with(|r| {
                    if let Some(item) = r.borrow().as_ref() {
                        if let Some(button) = item.button(self.mtm()) {
                            button.setTitle(&NSString::from_str("Downloading\u{2026}"));
                        }
                    }
                });
                return;
            }

            // Only add the menu item once.
            if UPDATE_MENU_ITEM.with(|r| r.borrow().is_some()) {
                return;
            }

            // Check whether an update has been found.
            let info = match UPDATE_SLOT.get() {
                Some(slot) => slot.lock().unwrap().clone(),
                None => return,
            };
            let info = match info {
                Some(i) => i,
                None => return,
            };

            let mtm = self.mtm();
            let title = NSString::from_str(&format!("Update available v{}", info.version));
            let update_item = unsafe {
                NSMenuItem::initWithTitle_action_keyEquivalent(
                    mtm.alloc(),
                    &title,
                    Some(objc2::sel!(installUpdate:)),
                    &NSString::from_str(""),
                )
            };

            // Set this delegate as the target so installUpdate: is dispatched here.
            let target: &AnyObject = self;
            unsafe { update_item.setTarget(Some(target)) };

            TRAY_MENU.with(|r| {
                if let Some(menu) = r.borrow().as_ref() {
                    menu.insertItem_atIndex(&update_item, 0);
                }
            });

            UPDATE_MENU_ITEM.with(|r| *r.borrow_mut() = Some(update_item));
            tracing::info!("Update menu item added for v{}", info.version);
        }

        /// Called when the user clicks "Update available vX.Y.Z" in the menu.
        /// Shows a native NSAlert for confirmation, then kicks off the install.
        #[unsafe(method(installUpdate:))]
        fn install_update_action(&self, _sender: &AnyObject) {
            let info = match UPDATE_SLOT.get() {
                Some(slot) => slot.lock().unwrap().clone(),
                None => return,
            };
            let info = match info {
                Some(i) => i,
                None => return,
            };

            let mtm = self.mtm();
            let alert = NSAlert::new(mtm);
            alert.setMessageText(&NSString::from_str("Update available"));
            alert.setInformativeText(&NSString::from_str(&format!(
                "Version {} is ready to install. \
                 The app will download the update and restart automatically.",
                info.version
            )));
            alert.addButtonWithTitle(&NSString::from_str("Install and Restart"));
            alert.addButtonWithTitle(&NSString::from_str("Not Now"));

            let response = alert.runModal();
            if response == NSAlertFirstButtonReturn {
                DOWNLOADING.store(true, Ordering::Relaxed);
                tracing::info!("User confirmed update to v{}", info.version);
                updater::install_update(info);
            }
        }
    }
);

impl TrayDelegate {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(TrayDelegateIvars);
        unsafe { msg_send![super(this), init] }
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub fn run<F: FnOnce()>(update_available: Arc<Mutex<Option<updater::UpdateInfo>>>, on_start: F) {
    let mtm = MainThreadMarker::new().expect("must be on main thread");

    // Make the Arc visible to the timer delegate.
    UPDATE_SLOT.get_or_init(|| update_available);

    // Initialize macOS application context
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    // Create status bar item directly via NSStatusBar
    let status_bar = NSStatusBar::systemStatusBar();
    let status_item: Retained<NSStatusItem> =
        status_bar.statusItemWithLength(NSVariableStatusItemLength);

    // Set the title text — visible immediately, no icon needed
    if let Some(button) = status_item.button(mtm) {
        button.setTitle(&NSString::from_str("BWM"));
    }

    // Build the menu
    let menu = NSMenu::new(mtm);

    // "Open remote" — action not yet wired (no Objective-C responder for openRemote)
    let open_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            mtm.alloc(),
            &NSString::from_str("Open remote"),
            Some(objc2::sel!(openRemote)),
            &NSString::from_str(""),
        )
    };
    menu.addItem(&open_item);

    // Separator
    menu.addItem(&NSMenuItem::separatorItem(mtm));

    // "Quit" — uses the built-in NSApplication terminate: selector
    let quit_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            mtm.alloc(),
            &NSString::from_str("Quit"),
            Some(objc2::sel!(terminate:)),
            &NSString::from_str("q"),
        )
    };
    menu.addItem(&quit_item);

    status_item.setMenu(Some(&menu));

    // Store references for the timer delegate to use (clone = retain)
    TRAY_MENU.with(|r| *r.borrow_mut() = Some(menu.clone()));
    STATUS_ITEM_REF.with(|r| *r.borrow_mut() = Some(status_item.clone()));

    // Create the delegate and schedule a repeating timer on the main run loop
    let delegate = TrayDelegate::new(mtm);
    let _timer = unsafe {
        NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
            5.0,
            &*delegate,
            objc2::sel!(checkTrayState:),
            None,
            true,
        )
    };

    tracing::info!("Status bar item created successfully");

    // Run startup logic (spawns server + browser-open thread)
    on_start();

    // Run the macOS event loop — blocks the main thread for the lifetime of the app
    app.run();
}
