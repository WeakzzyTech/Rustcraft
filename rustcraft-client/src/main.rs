// A release build is a window, not a console program; Windows would
// otherwise open a terminal beside it.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;

fn main() {
    rustcraft_client::run(
        rustcraft_client::LaunchArgs::parse(),
        rustcraft_client::Platform { data_root: None },
    );
}
