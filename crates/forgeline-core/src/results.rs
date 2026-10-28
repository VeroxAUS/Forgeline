// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Machine-readable results files: one JSON file per measurement, written by `tools/bench` and the
//! CI headless check. Published numbers are generated from these files, never typed by hand
//! (`docs/plans/phase-2-foundation.md`, section 3.3).
//!
//! The format is closed on purpose: unknown fields are rejected, every text value is checked, and
//! the frame trace holds numbers only. A file therefore cannot carry a path, a user name, a hunter ID
//! or game data by accident. The repo hygiene test runs [`validate`] on every committed file.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const SCHEMA: u32 = 1;

/// What the numbers were measured on. No machine name, user name or path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Machine {
    pub cpu: String,
    pub gpu: String,
    pub ram_bytes: u64,
    /// Operating system and architecture, for example `windows x86_64`.
    pub os: String,
}

/// A thinned frame-time trace: the full-resolution copy stays local, never in the repo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frames {
    /// `[bucket start ms, min ms, median ms, max ms]` per 100 ms bucket.
    pub buckets: Vec<[f64; 4]>,
    /// `[offset ms, frame time ms]` for every frame over the budget's lines (16 ms and 50 ms).
    pub over_budget: Vec<[f64; 2]>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchResult {
    pub schema: u32,
    /// UTC, `YYYY-MM-DDTHH:MM:SSZ`.
    pub date: String,
    pub app_version: String,
    /// Short or full commit hash, or `unknown`.
    pub git_commit: String,
    /// For example `ci-headless`, `idle`, `cold-start`, `recompute`.
    pub scenario: String,
    /// True for a CI runner reading; never mixed up with a reading on the maintainer's PC.
    pub ci: bool,
    pub machine: Machine,
    /// Wilds running or not, benchmark spot, window mode, admin or not, and so on.
    pub conditions: BTreeMap<String, String>,
    /// The unit of `runs` and `median`, for example `bytes` or `ms`.
    pub unit: String,
    /// Every raw value.
    pub runs: Vec<f64>,
    pub median: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frames: Option<Frames>,
}

impl BenchResult {
    /// Builds a result for the current build and machine. `runs` must not be empty.
    pub fn new(scenario: &str, ci: bool, unit: &str, runs: Vec<f64>) -> BenchResult {
        let median = median(&runs).unwrap_or(0.0);
        BenchResult {
            schema: SCHEMA,
            date: utc_now(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            git_commit: git_commit_from_env(),
            scenario: scenario.to_string(),
            ci,
            machine: current_machine(),
            conditions: BTreeMap::new(),
            unit: unit.to_string(),
            runs,
            median,
            frames: None,
        }
    }

    pub fn to_json(&self) -> String {
        let mut s = serde_json::to_string_pretty(self).expect("results serialise");
        s.push('\n');
        s
    }

    /// Writes `<dir>/<date>-<scenario>.json` and returns its path. Creates `dir` if missing.
    pub fn write_into(&self, dir: &Path) -> std::io::Result<PathBuf> {
        std::fs::create_dir_all(dir)?;
        let stamp = self.date.replace(':', "");
        let path = dir.join(format!("{stamp}-{}.json", self.scenario));
        std::fs::write(&path, self.to_json())?;
        Ok(path)
    }
}

pub fn median(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut v = values.to_vec();
    v.sort_by(|a, b| a.total_cmp(b));
    let mid = v.len() / 2;
    Some(if v.len() % 2 == 1 {
        v[mid]
    } else {
        (v[mid - 1] + v[mid]) / 2.0
    })
}

/// Checks a results file: right fields only, sane values, and no text that looks like a path.
pub fn validate(json: &str) -> Result<BenchResult, String> {
    let r: BenchResult =
        serde_json::from_str(json).map_err(|e| format!("not a results file: {e}"))?;
    if r.schema != SCHEMA {
        return Err(format!("unknown schema {}", r.schema));
    }
    if !is_utc_stamp(&r.date) {
        return Err(format!("date {:?} is not YYYY-MM-DDTHH:MM:SSZ", r.date));
    }
    if r.scenario.is_empty()
        || !r
            .scenario
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(format!(
            "scenario {:?} must be lowercase letters, digits and dashes",
            r.scenario
        ));
    }
    if r.git_commit != "unknown"
        && !(r.git_commit.len() >= 7 && r.git_commit.chars().all(|c| c.is_ascii_hexdigit()))
    {
        return Err("git_commit must be a hex hash or `unknown`".into());
    }
    if r.runs.is_empty() || r.runs.iter().any(|v| !v.is_finite()) {
        return Err("runs must hold at least one finite value".into());
    }
    let expected = median(&r.runs).unwrap_or(f64::NAN);
    if !r.median.is_finite() || (r.median - expected).abs() > 1e-9 * expected.abs().max(1.0) {
        return Err(format!(
            "median {} does not match runs (expected {expected})",
            r.median
        ));
    }
    let mut texts: Vec<&str> = vec![
        &r.app_version,
        &r.scenario,
        &r.unit,
        &r.machine.cpu,
        &r.machine.gpu,
        &r.machine.os,
    ];
    for (k, v) in &r.conditions {
        texts.push(k);
        texts.push(v);
    }
    for t in texts {
        if t.contains('\\') || t.contains('/') || t.contains(":\\") || t.len() > 200 {
            return Err(format!("text {t:?} looks like a path or is too long"));
        }
    }
    if let Some(f) = &r.frames {
        let finite = f.buckets.iter().flatten().all(|v| v.is_finite())
            && f.over_budget.iter().flatten().all(|v| v.is_finite());
        if !finite {
            return Err("frame trace holds a value that is not a finite number".into());
        }
    }
    Ok(r)
}

fn is_utc_stamp(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 20
        && b[4] == b'-'
        && b[7] == b'-'
        && b[10] == b'T'
        && b[13] == b':'
        && b[16] == b':'
        && b[19] == b'Z'
        && b.iter()
            .enumerate()
            .all(|(i, c)| matches!(i, 4 | 7 | 10 | 13 | 16 | 19) || c.is_ascii_digit())
}

/// The current time as `YYYY-MM-DDTHH:MM:SSZ`.
pub fn utc_now() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format_utc(secs)
}

fn format_utc(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    // Civil date from days since 1970-01-01 (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rem / 3_600,
        rem % 3_600 / 60,
        rem % 60
    )
}

fn git_commit_from_env() -> String {
    std::env::var("GITHUB_SHA")
        .ok()
        .filter(|s| s.len() >= 7 && s.chars().all(|c| c.is_ascii_hexdigit()))
        .unwrap_or_else(|| "unknown".into())
}

fn current_machine() -> Machine {
    Machine {
        cpu: cpu_name(),
        gpu: "none (headless)".into(),
        ram_bytes: total_ram_bytes().unwrap_or(0),
        os: format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
    }
}

#[cfg(windows)]
fn cpu_name() -> String {
    std::env::var("PROCESSOR_IDENTIFIER").unwrap_or_else(|_| "unknown".into())
}

#[cfg(not(windows))]
fn cpu_name() -> String {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|t| {
            t.lines()
                .find_map(|l| l.strip_prefix("model name"))
                .and_then(|l| l.split_once(':'))
                .map(|(_, v)| v.trim().to_string())
        })
        .unwrap_or_else(|| "unknown".into())
}

#[cfg(windows)]
fn total_ram_bytes() -> Option<u64> {
    use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    // SAFETY: plain data struct with its size set, as the API requires.
    unsafe {
        let mut s: MEMORYSTATUSEX = std::mem::zeroed();
        s.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
        (GlobalMemoryStatusEx(&mut s) != 0).then_some(s.ullTotalPhys)
    }
}

#[cfg(not(windows))]
fn total_ram_bytes() -> Option<u64> {
    std::fs::read_to_string("/proc/meminfo")
        .ok()?
        .lines()
        .find_map(|l| l.strip_prefix("MemTotal:"))
        .and_then(|v| v.trim().trim_end_matches("kB").trim().parse::<u64>().ok())
        .map(|k| k * 1024)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> BenchResult {
        let mut r = BenchResult::new("ci-headless", true, "bytes", vec![10.0, 30.0, 20.0]);
        r.conditions.insert("wilds".into(), "not running".into());
        r
    }

    #[test]
    fn a_fresh_result_validates_and_round_trips() {
        let r = sample();
        assert_eq!(r.median, 20.0);
        let back = validate(&r.to_json()).unwrap();
        assert_eq!(back, r);
    }

    #[test]
    fn unknown_fields_are_rejected() {
        let mut v: serde_json::Value = serde_json::from_str(&sample().to_json()).unwrap();
        v["hunter_id"] = "123".into();
        assert!(validate(&v.to_string()).is_err());
    }

    #[test]
    fn paths_and_wrong_medians_are_rejected() {
        let mut r = sample();
        r.conditions
            .insert("note".into(), "C:\\Users\\someone\\saves".into());
        assert!(validate(&r.to_json()).unwrap_err().contains("path"));
        let mut r = sample();
        r.median = 99.0;
        assert!(validate(&r.to_json()).unwrap_err().contains("median"));
        let mut r = sample();
        r.date = "yesterday".into();
        assert!(validate(&r.to_json()).is_err());
    }

    #[test]
    fn frames_hold_numbers_only() {
        let mut r = sample();
        r.frames = Some(Frames {
            buckets: vec![[0.0, 6.0, 7.0, 9.0]],
            over_budget: vec![[120.0, 21.5]],
        });
        assert!(validate(&r.to_json()).is_ok());
    }

    #[test]
    fn utc_formatting_is_correct() {
        assert_eq!(format_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(format_utc(1_793_159_520), "2026-10-28T03:52:00Z");
        assert_eq!(format_utc(951_782_400), "2000-02-29T00:00:00Z");
    }

    #[test]
    fn write_into_names_the_file_by_date_and_scenario() {
        let dir = std::env::temp_dir().join(format!("forgeline-results-{}", std::process::id()));
        let path = sample().write_into(&dir).unwrap();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        assert!(
            name.ends_with("-ci-headless.json") && !name.contains(':'),
            "{name}"
        );
        assert!(validate(&std::fs::read_to_string(&path).unwrap()).is_ok());
        let _ = std::fs::remove_dir_all(dir);
    }
}
