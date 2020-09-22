use indicatif::{ProgressBar, ProgressStyle};

pub mod cursive;

pub fn progress_style() -> ProgressStyle {
    ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:32.green/darkgreen} {pos:>7}/{len:7} {msg}")
        .progress_chars("##-")
}

pub fn spinner(msg: &str) -> ProgressBar {
    let style = ProgressStyle::default_spinner()
        .template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .tick_chars("⠁⠁⠉⠙⠚⠒⠂⠂⠒⠲⠴⠤⠄⠄⠤⠠⠠⠤⠦⠖⠒⠐⠐⠒⠓⠋⠉⠈⠈✓"); // TODO make tick green?
    let progress = ProgressBar::new_spinner();
    progress.set_style(style);
    progress.set_message(msg);
    progress.enable_steady_tick(100);
    progress
}

pub fn bar(msg: &str) -> ProgressBar {
    let progress = ProgressBar::new(1);
    progress.set_message(msg);
    progress.set_style(progress_style());
    progress.enable_steady_tick(100);
    progress
}

pub fn wait_and_notice<F, T>(msg: &str, f: F) -> T
where
    F: FnOnce() -> T,
{
    let progress = spinner(msg);
    let result = f();
    progress.finish();
    result
}
