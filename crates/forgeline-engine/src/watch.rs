// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Watches the save folder with the operating system's change notifications. There is no timer:
//! with no events the thread sleeps on a channel and uses no CPU.
//!
//! A change callback fires once, after the file has been quiet for `quiet` (so a half-written
//! save is not read). The folder is watched, not the file, because the game may replace the file.
//! If the folder does not exist yet, the nearest existing parent is watched until it appears.

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::thread::JoinHandle;
use std::time::Duration;

/// How long the save must be quiet before the callback fires.
pub const QUIET: Duration = Duration::from_millis(500);

enum Msg {
    /// A file system event; `hits_file` is true if it names the save file.
    Event {
        hits_file: bool,
    },
    Stop,
}

pub struct SaveWatcher {
    tx: Sender<Msg>,
    thread: Option<JoinHandle<()>>,
}

impl SaveWatcher {
    /// Starts watching `dir` for changes to `file` (compared without regard to case).
    /// `on_change` runs on the watcher's own thread.
    pub fn start(
        dir: PathBuf,
        file: &str,
        quiet: Duration,
        on_change: impl Fn() + Send + 'static,
    ) -> io::Result<SaveWatcher> {
        let (tx, rx) = mpsc::channel::<Msg>();
        let target = file.to_ascii_lowercase();
        let cb_tx = tx.clone();
        let mut watcher: RecommendedWatcher =
            notify::recommended_watcher(move |res: notify::Result<Event>| {
                let hits_file = match res {
                    Ok(ev) if matches!(ev.kind, EventKind::Access(_)) => return,
                    Ok(ev) => ev.paths.iter().any(|p| {
                        p.file_name()
                            .is_some_and(|n| n.to_string_lossy().to_ascii_lowercase() == target)
                    }),
                    // The watcher lost events (for example an overflow): re-read to be safe.
                    Err(_) => true,
                };
                let _ = cb_tx.send(Msg::Event { hits_file });
            })
            .map_err(to_io)?;
        let mut watched = arm(&mut watcher, None, &dir)?;

        let thread = std::thread::Builder::new()
            .name("forgeline-watch".into())
            .spawn(move || {
                forgeline_core::priority::set_thread_lowest();
                'outer: loop {
                    // Sleeps here, with no timer, until something happens.
                    let Ok(Msg::Event { hits_file }) = rx.recv() else {
                        return;
                    };
                    let mut relevant = hits_file || watched != dir;
                    // Wait for quiet; every further event restarts the wait.
                    loop {
                        match rx.recv_timeout(quiet) {
                            Ok(Msg::Event { hits_file }) => relevant |= hits_file || watched != dir,
                            Ok(Msg::Stop) | Err(RecvTimeoutError::Disconnected) => break 'outer,
                            Err(RecvTimeoutError::Timeout) => break,
                        }
                    }
                    if !relevant {
                        continue;
                    }
                    // The folder may have appeared, or gone: watch the right place now.
                    if let Ok(w) = arm(&mut watcher, Some(&watched), &dir) {
                        watched = w;
                    }
                    if watched == dir {
                        on_change();
                    }
                }
            })?;
        Ok(SaveWatcher {
            tx,
            thread: Some(thread),
        })
    }
}

impl Drop for SaveWatcher {
    fn drop(&mut self) {
        let _ = self.tx.send(Msg::Stop);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

fn to_io(e: notify::Error) -> io::Error {
    io::Error::other(e.to_string())
}

/// Watches `dir` if it exists, else its nearest existing parent. Moves the watch if it was
/// somewhere else. Returns the folder now watched.
fn arm(w: &mut RecommendedWatcher, old: Option<&Path>, dir: &Path) -> io::Result<PathBuf> {
    let want = dir
        .ancestors()
        .find(|p| p.is_dir())
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no folder to watch"))?;
    if old == Some(want) {
        return Ok(want.to_path_buf());
    }
    if let Some(old) = old {
        let _ = w.unwatch(old);
    }
    w.watch(want, RecursiveMode::NonRecursive).map_err(to_io)?;
    Ok(want.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::channel;

    const QUIET_TEST: Duration = Duration::from_millis(150);

    fn scratch(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("forgeline-watch-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn counter(dir: PathBuf) -> (SaveWatcher, std::sync::mpsc::Receiver<()>) {
        let (tx, rx) = channel();
        let w = SaveWatcher::start(dir, "data001Slot.bin", QUIET_TEST, move || {
            let _ = tx.send(());
        })
        .unwrap();
        (w, rx)
    }

    #[test]
    fn a_write_triggers_one_callback_after_the_quiet_period_even_for_several_writes() {
        let dir = scratch("burst");
        let (_w, rx) = counter(dir.clone());
        let file = dir.join("data001Slot.bin");
        for i in 0..5 {
            std::fs::write(&file, [i]).unwrap();
            std::thread::sleep(Duration::from_millis(20));
        }
        rx.recv_timeout(Duration::from_secs(5))
            .expect("one callback");
        assert!(rx.recv_timeout(QUIET_TEST * 4).is_err(), "only one");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn other_files_in_the_folder_and_silence_trigger_nothing() {
        let dir = scratch("other");
        let (_w, rx) = counter(dir.clone());
        std::fs::write(dir.join("data002Slot.bin"), [1]).unwrap();
        assert!(rx.recv_timeout(QUIET_TEST * 4).is_err());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_folder_that_does_not_exist_yet_is_picked_up_when_it_appears() {
        let base = scratch("late");
        let dir = base.join("remote").join("win64_save");
        let (_w, rx) = counter(dir.clone());
        std::fs::create_dir_all(&dir).unwrap();
        // Creating the folder is itself a change to look at.
        rx.recv_timeout(Duration::from_secs(5))
            .expect("folder seen");
        std::fs::write(dir.join("data001Slot.bin"), [1]).unwrap();
        rx.recv_timeout(Duration::from_secs(5)).expect("file seen");
        let _ = std::fs::remove_dir_all(base);
    }
}
