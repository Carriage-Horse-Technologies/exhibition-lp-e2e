pub mod test_exhibition_lp;

use core::fmt;
use std::io::Write;
use std::{
    error::{self, Error},
    sync::Arc,
    time::Duration,
};

use env_logger::{fmt::Color, Builder};
use headless_chrome::{
    protocol::cdp::{types::Event, Fetch, Log::LogEntryLevel, Network, Page, Performance, DOM},
    Tab,
};
use headless_chrome::{Browser, LaunchOptionsBuilder};
use log::{error, info, warn, Level};
use once_cell::sync::Lazy;

#[derive(Debug, Default)]
pub struct Config {
    pub timeout: Duration,
    pub fail_load_time: f64,
    pub target_url: String,
    pub screenshot_dir: String,
}

impl Config {
    pub fn get_screenshot_path(&self, file_name: &str) -> String {
        format!("{}/{}", self.screenshot_dir, file_name)
    }
}

pub const CONFIG: Lazy<Config> = Lazy::new(|| Config {
    timeout: Duration::from_secs(60),
    fail_load_time: 3.,
    target_url: "https://exhibition.yukinissie.com".to_string(),
    screenshot_dir: "screenshot".to_string(),
});

pub fn init_logger() {
    let mut builder = Builder::new();

    builder.format(|buf, record| {
        let level_color = match record.level() {
            Level::Trace => Color::White,
            Level::Debug => Color::Blue,
            Level::Info => Color::Green,
            Level::Warn => Color::Yellow,
            Level::Error => Color::Red,
        };
        let mut level_style = buf.style();
        level_style.set_color(level_color);

        writeln!(
            buf,
            "{level} {file}:{line} {args}",
            level = level_style.value(record.level()),
            args = level_style.value(record.args()),
            file = level_style.value(&record.file().unwrap_or("____unknown")[4..]), // src/file.rs -> file.rs
            line = level_style.value(record.line().unwrap_or(0)),
        )
    });
    builder.filter_level(log::LevelFilter::Debug);
    builder.write_style(env_logger::WriteStyle::Auto);

    builder.init();
}

fn init_sandbox_chrome() -> Result<(Browser, Arc<Tab>), Box<dyn Error>> {
    let launch_options = LaunchOptionsBuilder::default()
        .sandbox(true)
        .window_size(Some((1280, 720)))
        .build()?;
    let browser = Browser::new(launch_options)?;

    let tab = browser.wait_for_initial_tab()?;
    init_event(&tab)?;

    Ok((browser, tab))
}

fn init_event(tab: &Arc<Tab>) -> Result<(), Box<dyn Error>> {
    tab.set_default_timeout(CONFIG.timeout);

    tab.enable_log()?;
    tab.enable_runtime()?;
    tab.enable_debugger()?;
    tab.enable_profiler()?;
    tab.enable_fetch(None, Some(true))?;
    tab.call_method(Performance::Enable { time_domain: None })?;
    tab.call_method(DOM::Enable(None))?;
    tab.call_method(Network::Enable {
        max_post_data_size: None,
        max_resource_buffer_size: None,
        max_total_buffer_size: None,
    })?;
    tab.call_method(Page::Enable(None))?;

    Ok(())
}

fn init_event_listener(tab: &Arc<Tab>, goto_url: &str) -> Result<(), Box<dyn Error>> {
    tab.add_event_listener(Arc::new({
        let tab = tab.clone();
        let goto_url = goto_url.to_string();
        move |event: &Event| match event {
            // Fetch errorなどによるログ
            Event::LogEntryAdded(e) => {
                let log_entry = &e.params.entry;
                info!("LogEntryAdded {:#?}", log_entry);
                match log_entry.level {
                    LogEntryLevel::Error => {
                        error!(
                            "goto_url: {}; {} source: {:?}; url: {}; LN: {:?}; network_request_id: {:?}",
                            goto_url,
                            log_entry.text,
                            log_entry.source,
                            log_entry.url.as_ref().unwrap_or(&String::new()),
                            log_entry.line_number,
                            log_entry.network_request_id
                        );
                    }
                    LogEntryLevel::Warning => {
                        warn!(
                            "goto_url: {}; {} source: {:?}; url: {}; LN: {:?}; network_request_id: {:?}",
                            goto_url,
                            log_entry.text,
                            log_entry.source,
                            log_entry.url.as_ref().unwrap_or(&String::new()),
                            log_entry.line_number,
                            log_entry.network_request_id
                        );
                    }
                    LogEntryLevel::Info => (),
                    LogEntryLevel::Verbose => (),
                }
            }
            Event::NetworkLoadingFailed(e) => {
                let loading_failed = &e.params;

                error!("NetworkLoadingFailed {:?}", loading_failed);
            }
            _ => (),
        }
    }))?;

    Ok(())
}

fn get_load_time(tab: &Arc<Tab>) -> Result<f64, Box<dyn Error>> {
    let metrics_obj = tab.call_method(Performance::GetMetrics(None))?;
    let metrics = &metrics_obj.metrics;
    // error!("{:#?}", metrics);
    let load_time = metrics
        .iter()
        .find(|m| m.name == "FirstMeaningfulPaint")
        .unwrap()
        .value
        - metrics
            .iter()
            .find(|m| m.name == "NavigationStart")
            .unwrap()
            .value;

    Ok(load_time)
}
