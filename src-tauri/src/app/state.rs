use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
};

use rusqlite::Connection;

use crate::store::settings::{AppSettings, HotkeySettings};

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
pub(crate) struct SettingsState(pub(crate) Arc<Mutex<Option<AppSettings>>>);

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
pub(crate) struct HotkeyDisableCount(pub(crate) Arc<AtomicUsize>);

impl Default for HotkeyDisableCount {
    fn default() -> Self {
        Self(Arc::new(AtomicUsize::new(0)))
    }
}

impl HotkeyDisableCount {
    pub(crate) fn request_disabled(&self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn release_disabled(&self) {
        let mut current = self.0.load(Ordering::Relaxed);

        while current > 0 {
            match self.0.compare_exchange_weak(
                current,
                current - 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return,
                Err(next_current) => current = next_current,
            }
        }
    }

    pub(crate) fn is_disabled(&self) -> bool {
        self.0.load(Ordering::Relaxed) > 0
    }
}
