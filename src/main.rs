mod apple_search;
mod args;
mod arp;
mod iphone_sync;
mod oui;
mod util;

use apple_search::{search_for_apples, DetectionResult};
use args::Args;
use clap::Parser;
use log::LevelFilter;
use oui::OuiDatabase;
use util::get_default_interface;

fn too_many_apples() -> ! {
    std::process::Command::new("halt")
        .output()
        .expect("cannot run halt");
    unreachable!();
}

fn main() {
    let args = Args::parse();

    let log_level = match args.verbose {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        _ => LevelFilter::Debug,
    };

    env_logger::builder().filter_level(log_level).init();

    let default_interface = get_default_interface();
    let database = OuiDatabase::new();

    let results = search_for_apples(&default_interface, &database);

    let apples_count = results
        .iter()
        .filter(|result| result.detection_result != DetectionResult::NotApple)
        .count();

    if apples_count > args.panic_after {
        too_many_apples();
    }

    for result in &results {
        let emoji = match result.detection_result {
            DetectionResult::NotApple => "💩",
            DetectionResult::AppleByMac => "🍏",
            DetectionResult::AppleByOpenPort => "🍎",
        };

        log::info!("{} {} {}", emoji, result.mac, result.ip);
    }

    log::info!(
        "Detected {}/{} ({} would have killed you)",
        apples_count,
        results.len(),
        args.panic_after
    );
}
