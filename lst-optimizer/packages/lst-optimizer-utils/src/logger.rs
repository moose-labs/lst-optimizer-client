use anyhow::Result;
use fern::colors::{Color, ColoredLevelConfig};

pub use log::{debug, error, info, trace, warn};

pub fn setup_global_logger() -> Result<()> {
    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::White)
        .debug(Color::Magenta)
        .trace(Color::BrightBlack);
    let colors_level = colors_line.info(Color::Green);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{} {} {}",
                humantime::format_rfc3339_seconds(std::time::SystemTime::now()),
                colors_level.color(record.level()),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .chain(fern::log_file("runtime.log")?)
        .apply()?;
    Ok(())
}
