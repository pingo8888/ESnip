use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use rusqlite::Connection;

use crate::store::settings::HotkeySettings;

pub(crate) struct DbState {
    pub(crate) conn: Mutex<Connection>,
}

impl DbState {
    pub(crate) fn new(conn: Connection) -> Self {
        Self {
            conn: Mutex::new(conn),
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct AppQuitState(pub(crate) Arc<AtomicBool>);

impl AppQuitState {
    pub(crate) fn request_quit(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    pub(crate) fn is_quitting(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

#[derive(Clone, Default)]
pub(crate) struct HotkeyShutdown(pub(crate) Arc<AtomicBool>);

impl HotkeyShutdown {
    pub(crate) fn request_shutdown(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

#[derive(Clone)]
pub(crate) struct HotkeyState(pub(crate) Arc<Mutex<HotkeySettings>>);

impl Default for HotkeyState {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(HotkeySettings::default())))
    }
}

#[derive(Clone)]
pub(crate) struct HotkeyEnabled(pub(crate) Arc<AtomicBool>);

impl Default for HotkeyEnabled {
    fn default() -> Self {
        Self(Arc::new(AtomicBool::new(true)))
    }
}

impl HotkeyEnabled {
    pub(crate) fn set_enabled(&self, enabled: bool) {
        self.0.store(enabled, Ordering::Relaxed);
    }
}
