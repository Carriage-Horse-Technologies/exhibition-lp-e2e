use std::env;

use exhibition_lp_e2e::{init_logger, test_exhibition_lp::test_exhibition_lp, CONFIG};

fn main() {
    if cfg!(debug_assertions) {
        env::set_var("RUST_LOG", "debug");
    } else {
        env::set_var("RUST_LOG", "info");
    }
    init_logger();
    std::fs::create_dir_all(&CONFIG.screenshot_dir).expect("Failed to create_dir_all");

    log::debug!("Start test.");
    if let Err(e) = test_exhibition_lp() {
        log::error!("{:?}", e);
    }
}
