use gdnative::godot_print;
use log::{debug, info, trace, LevelFilter, Log};

pub struct GodotLog;

impl GodotLog {
    pub fn init() {
        if log::set_boxed_logger(Box::new(Self)).is_ok() {
            log::set_max_level(LevelFilter::Info);
            info!("Godot logger initialized.");
        } else {
            info!("Logger already initialized.");
        }
    }
}

impl Log for GodotLog {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        godot_print!("[{}] {}", record.level(), record.args());
    }

    fn flush(&self) {}
}
