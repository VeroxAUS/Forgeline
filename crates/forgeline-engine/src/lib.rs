// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The engine: one low-priority worker thread that keeps the `Snapshot` up to date.
//!
//! At start it reads the view cache, so `snapshot()` has the last view at once. The worker then
//! checks in the background whether the save changed since the cache was written and recomputes
//! only if it did. It then sleeps on a channel, with no timer, until the file watcher reports a
//! change or `refresh` is called. A change during a recompute runs one more recompute after it.
//! A failed read keeps the last good summary and sets a visible error.
//!
//! The save is only ever read: opened with sharing, read whole into memory and checked for
//! unchanged size and modified time across the read. Nothing here writes to the game, Steam or
//! the save folder.

pub mod cache;
mod compute;
pub mod snapshot;
pub mod watch;

pub use compute::{compute, ComputeInput, SaveOverride};
pub use snapshot::Snapshot;

use forgeline_core::datadir::DataDir;
use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

/// Where the engine looks. A dev or test build always passes its own data directory.
pub struct EngineConfig {
    pub data: DataDir,
    /// Steam folder; found by the usual search if `None`.
    pub steam_root: Option<PathBuf>,
    /// Which Steam account folder to use when several have Wilds saves.
    pub account_id: Option<u32>,
    /// Read this save copy instead of looking for the live one (tests and tools).
    pub save_override: Option<SaveOverride>,
}

enum Msg {
    SaveChanged,
    Refresh,
    Stop,
}

pub struct Engine {
    current: Arc<Mutex<Arc<Snapshot>>>,
    tx: Sender<Msg>,
    worker: Option<JoinHandle<()>>,
}

impl Engine {
    /// Starts the worker. `notify` is called (from the worker) after each new snapshot is
    /// published; the window uses it to ask for one repaint.
    pub fn start(config: EngineConfig, notify: impl Fn() + Send + Sync + 'static) -> Engine {
        let first = cache::load(&config.data.view_cache()).unwrap_or_else(Snapshot::loading);
        let current = Arc::new(Mutex::new(Arc::new(first)));
        let (tx, rx) = mpsc::channel();
        let worker = {
            let (current, tx) = (current.clone(), tx.clone());
            std::thread::Builder::new()
                .name("forgeline-engine".into())
                .spawn(move || run(config, current, tx, rx, notify))
                .expect("the operating system can start a thread")
        };
        Engine {
            current,
            tx,
            worker: Some(worker),
        }
    }

    /// The latest snapshot. Cheap: a lock held for one pointer copy.
    pub fn snapshot(&self) -> Arc<Snapshot> {
        self.current.lock().expect("snapshot lock").clone()
    }

    /// Asks for a recompute that re-reads the data status and the save only if it changed. Call
    /// it after an update finishes.
    pub fn refresh(&self) {
        let _ = self.tx.send(Msg::Refresh);
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        let _ = self.tx.send(Msg::Stop);
        if let Some(w) = self.worker.take() {
            let _ = w.join();
        }
    }
}

fn run(
    config: EngineConfig,
    current: Arc<Mutex<Arc<Snapshot>>>,
    tx: Sender<Msg>,
    rx: mpsc::Receiver<Msg>,
    notify: impl Fn() + Send + Sync + 'static,
) {
    forgeline_core::priority::set_thread_lowest();
    // A crash between the update's two renames leaves no `current`; put the old data back before
    // the first read. Once, here: later it could race a running update's own swap. Failure is
    // not fatal: the view then shows that no data is extracted.
    let _ = forgeline_update::swap::recover(&config.data);
    let cache_path = config.data.view_cache();
    let mut watcher: Option<(PathBuf, watch::SaveWatcher)> = None;
    let mut force = false;
    loop {
        let prev = current.lock().expect("snapshot lock").clone();
        let input = ComputeInput {
            data: &config.data,
            steam_root: config.steam_root.clone(),
            account_id: config.account_id,
            save_override: config.save_override.as_ref(),
            force_read: force,
        };
        let out = compute(&input, &prev);
        let mut next = out.snapshot;

        // Watch the save's folder; re-arm only if it moved.
        match out.save_path.as_deref().and_then(|p| p.parent()) {
            Some(dir) if watcher.as_ref().is_none_or(|(d, _)| d != dir) => {
                let name = out
                    .save_path
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let tx = tx.clone();
                match watch::SaveWatcher::start(dir.to_path_buf(), &name, watch::QUIET, move || {
                    let _ = tx.send(Msg::SaveChanged);
                }) {
                    Ok(w) => {
                        watcher = Some((dir.to_path_buf(), w));
                        next.watch_error = None;
                    }
                    Err(e) => {
                        watcher = None;
                        next.watch_error =
                            Some(format!("not watching the save folder for changes: {e}"));
                    }
                }
            }
            Some(_) => next.watch_error = prev.watch_error.clone(),
            None => {
                watcher = None;
            }
        }

        let changed = !next.same_view(&prev) || next.watch_error != prev.watch_error;
        if changed {
            next.refreshing = false;
            next.version = prev.version + 1;
            if !next.same_view(&prev) {
                cache::store(&cache_path, &next);
            }
            *current.lock().expect("snapshot lock") = Arc::new(next);
            notify();
        } else if prev.refreshing {
            // The cached view is current: only the "refreshing" mark goes.
            let mut done = (*prev).clone();
            done.refreshing = false;
            done.version = prev.version + 1;
            *current.lock().expect("snapshot lock") = Arc::new(done);
            notify();
        }

        // Sleep until told; no timer. Changes that arrived during the recompute are already
        // queued and run it once more.
        force = false;
        match rx.recv() {
            Ok(Msg::SaveChanged) => force = true,
            Ok(Msg::Refresh) => {}
            Ok(Msg::Stop) | Err(_) => return,
        }
        while let Ok(m) = rx.try_recv() {
            match m {
                Msg::SaveChanged => force = true,
                Msg::Refresh => {}
                Msg::Stop => return,
            }
        }
    }
}
