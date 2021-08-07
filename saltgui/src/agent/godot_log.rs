// use gdnative::godot_print;
// use log::{debug, info, trace, LevelFilter, Log};

// pub(crate) struct GodotLog;

// impl GodotLog {
//     pub fn init() {
//         log::set_boxed_logger(Box::new(Self)).unwrap();
//         log::set_max_level(LevelFilter::Info);
//         info!("Godot logger initialized.");
//     }
// }

// impl Log for GodotLog {
//     fn enabled(&self, _metadata: &log::Metadata) -> bool {
//         true
//     }

//     fn log(&self, record: &log::Record) {
//         godot_print!("[{}] {}", record.level(), record.args());
//     }

//     fn flush(&self) {}
// }
