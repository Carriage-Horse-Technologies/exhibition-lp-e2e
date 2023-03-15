use std::{error::Error, fs, sync::Arc, time::Duration};

use headless_chrome::{protocol::cdp::Page, Browser, LaunchOptionsBuilder, Tab};
use log::{debug, error};

use crate::{get_load_time, init_event, init_event_listener, init_sandbox_chrome, CONFIG};

pub fn test_exhibition_lp() -> Result<(), Box<dyn Error>> {
    let (_browser, tab) = init_sandbox_chrome()?;

    let target_url = &CONFIG.target_url;
    init_event_listener(&tab, &target_url)?;

    test_exhibition_lp_entrance(&tab)?;
    test_exhibition_lp_home(&tab)?;

    Ok(())
}

pub(crate) fn test_exhibition_lp_entrance(tab: &Arc<Tab>) -> Result<(), Box<dyn Error>> {
    tab.navigate_to(&format!("{}/entrance", CONFIG.target_url))?;
    tab.wait_until_navigated()?;

    let ss = tab.capture_screenshot(Page::CaptureScreenshotFormatOption::Png, None, None, true)?;
    fs::write(CONFIG.get_screenshot_path("exhibition-entrance.png"), &ss)?;
    let load_time = get_load_time(&tab)?;
    if load_time > CONFIG.fail_load_time {
        // タイムアウト
        error!(
            "timeout target url: {}; load time: {}",
            tab.get_url(),
            load_time
        );
    }

    //Input Username
    let input_username = tab.wait_for_element("#github_username")?;
    input_username.click()?;
    tab.send_character("user")?;
    let ss = tab.capture_screenshot(Page::CaptureScreenshotFormatOption::Png, None, None, true)?;
    fs::write(
        CONFIG.get_screenshot_path("exhibition-entrance-input-username.png"),
        &ss,
    )?;
    let go_element = tab.wait_for_element("body > div > div > main > div > form > button")?;
    go_element.click()?;
    std::thread::sleep(Duration::from_secs(1));
    tab.wait_until_navigated()?;
    let ss = tab.capture_screenshot(Page::CaptureScreenshotFormatOption::Png, None, None, true)?;
    fs::write(
        CONFIG.get_screenshot_path("exhibition-entrance-to-home.png"),
        &ss,
    )?;

    Ok(())
}

pub(crate) fn test_exhibition_lp_home(tab: &Arc<Tab>) -> Result<(), Box<dyn Error>> {
    tab.navigate_to(&CONFIG.target_url)?;
    tab.wait_until_navigated()?;

    let ss = tab.capture_screenshot(Page::CaptureScreenshotFormatOption::Png, None, None, true)?;
    fs::write(CONFIG.get_screenshot_path("exhibition-entrance.png"), &ss)?;
    let load_time = get_load_time(&tab)?;
    if load_time > CONFIG.fail_load_time {
        // タイムアウト
        error!(
            "timeout target url: {}; load time: {}",
            tab.get_url(),
            load_time
        );
    }

    // chara click
    let chara = tab.wait_for_element(r###"body > div > div > main > div > div:nth-child(1) > div.absolute.select-none.-top-\[32px\].-left-\[32px\].w-\[64px\].h-\[64px\].rounded-full.transform-gpu.translate-x-\[40vw\].translate-y-\[50vh\].z-\[900\].ease-out.duration-200.overflow-hidden.border-4.border-green-500"###)?;
    chara.click()?;

    let new_prod = tab.wait_for_element(r###"body > div > div > main > div > div:nth-child(3) > div.flex.flex-row.justify-center.items-center.mb-\[170px\] > div > div.w-fit.h-fit.m-8.flex.max-w-\[512px\].hover\:border-2.hover\:border-picture-frame.mt-\[300px\].bg-gradient-to-r.from-\[\#6080B0\].via-\[\#08DCF9\].to-\[\#FF2775\].p-2.m-0.z-\[600\] > figure"###)?;
    new_prod.click()?;
    std::thread::sleep(Duration::from_secs(1));
    let ss = tab.capture_screenshot(Page::CaptureScreenshotFormatOption::Png, None, None, true)?;
    fs::write(
        CONFIG.get_screenshot_path("exhibition-new-prod-click.png"),
        &ss,
    )?;

    Ok(())
}
