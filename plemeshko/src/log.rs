use std::{
    borrow::Cow,
    sync::atomic::{AtomicBool, Ordering},
};

pub use ::log::{debug, error, info, trace, warn};
use anyhow::{bail, Result};
use colored::{ColoredString, Colorize};

macro_rules! log_trg {
    () => { env!("CARGO_PKG_NAME") };
    ($($s:expr),+ $(,)?) => { concat!(log_trg!(), $(":",$s),*) };
}

static SUPPORT_COLORS: AtomicBool = AtomicBool::new(false);

pub fn colorize<'a>(text: &'a str, col: impl FnOnce(&'a str) -> ColoredString) -> Cow<'a, str> {
    if SUPPORT_COLORS.load(Ordering::Relaxed) {
        Cow::Owned(col(text).to_string())
    } else {
        Cow::Borrowed(text)
    }
}

pub fn initialize_log() -> Result<()> {
    const ENV_LOG_TARGET: &str = "PLEMESHKO_LOG";
    let Ok(target) = std::env::var(ENV_LOG_TARGET) else {
        return Ok(());
    };

    let time_format =
        time::format_description::parse("[hour]:[minute]:[second].[subsecond digits:3]")?;

    if &target == "stdout" || &target == "stderr" {
        SUPPORT_COLORS.store(true, Ordering::Relaxed);
        let fern_colors = fern::colors::ColoredLevelConfig::new()
            .info(fern::colors::Color::Cyan)
            .warn(fern::colors::Color::Yellow)
            .error(fern::colors::Color::Red);
        let fern_dispatch = fern::Dispatch::new().level(log::LevelFilter::Warn).format(
            move |out, message, record| {
                let time = time::OffsetDateTime::from(std::time::SystemTime::now());
                out.finish(format_args!(
                    "{}[{}][{}] {}",
                    time.format(&time_format).unwrap(),
                    if record.target().starts_with(log_trg!()) {
                        record.target().green()
                    } else {
                        record.target().normal()
                    },
                    fern_colors.color(record.level()),
                    message
                ))
            },
        );
        match target.as_str() {
            "stdout" => fern_dispatch.chain(std::io::stdout()),
            "stderr" => fern_dispatch.chain(std::io::stderr()),
            _ => panic!(),
        }
        .apply()?;
        return Ok(());
    }
    if target.len() < 2 || target.starts_with('/') {
        bail!("Can't parse env: {ENV_LOG_TARGET}");
    }
    fern::Dispatch::new()
        .level(log::LevelFilter::Debug)
        .format(move |out, message, record| {
            let time = time::OffsetDateTime::from(std::time::SystemTime::now());
            out.finish(format_args!(
                "{}[{}][{}] {}",
                time.format(&time_format).unwrap(),
                record.target(),
                record.level(),
                message
            ))
        })
        .chain(fern::log_file(target.chars().skip(1).collect::<String>())?)
        .apply()?;
    Ok(())
}
