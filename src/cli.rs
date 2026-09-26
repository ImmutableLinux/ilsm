use indicatif::ProgressBar;
use std::time::Duration;

pub struct Spinner {
    bar: ProgressBar,
    message: String,
}

impl Spinner {
    pub fn new(message: &str) -> Self {
        let bar = ProgressBar::new_spinner();

        bar.enable_steady_tick(Duration::from_millis(100));
        bar.set_message(message.to_string());

        Self {
            bar,
            message: message.to_string(),
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.bar.finish_with_message(format!("{}done", self.message));
    }
}