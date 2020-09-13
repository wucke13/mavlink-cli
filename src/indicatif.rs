use indicatif::{ProgressBar, ProgressStyle};

pub fn progress_style() -> ProgressStyle {
    ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.green/darkgreen} {pos:>7}/{len:7} {msg}")
        .progress_chars("##-")
}

pub fn new_spinner(msg: &str) -> ProgressBar {
    let progress = ProgressBar::new_spinner();
    progress.set_message(msg);
    progress.enable_steady_tick(100);
    progress
}

pub fn new_bar(msg: &str) -> ProgressBar {
    let progress = ProgressBar::new(1);
    progress.set_message(msg);
    progress.set_style(progress_style());
    progress.enable_steady_tick(100);
    progress
}
