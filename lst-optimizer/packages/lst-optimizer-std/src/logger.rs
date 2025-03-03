use anyhow::Result;

pub fn setup_global_logger() -> Result<()> {
    fern::Dispatch
        ::new()
        .format(|out, message, record| {
            out.finish(
                format_args!(
                    "{} {} {}",
                    humantime::format_rfc3339_seconds(std::time::SystemTime::now()),
                    record.level(),
                    message
                )
            )
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("runtime.log")?)
        .apply()?;
    Ok(())
}
