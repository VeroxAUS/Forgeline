// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{env, fs, process::ExitCode};

const USAGE: &str =
    "usage: plan-page <input.md> [-o <output.html>] [--title <text>] [--standalone]";

fn main() -> ExitCode {
    let mut input = None;
    let mut output = None;
    let mut title = None;
    let mut standalone = false;
    let mut args = env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "-o" | "--output" => output = args.next(),
            "--title" => title = args.next(),
            "--standalone" => standalone = true,
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            _ if input.is_none() => input = Some(a),
            _ => {
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            }
        }
    }
    let Some(input) = input else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };
    let md = match fs::read_to_string(&input) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("cannot read {input}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let page = plan_page::render_page(&md, title.as_deref(), standalone);
    match output {
        Some(path) => {
            if let Err(e) = fs::write(&path, page) {
                eprintln!("cannot write {path}: {e}");
                return ExitCode::FAILURE;
            }
        }
        None => print!("{page}"),
    }
    ExitCode::SUCCESS
}
