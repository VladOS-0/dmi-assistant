#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, fs, panic, path::Path};

use chrono::Local;
use dmi_assistant::{
    DEFAULT_THEME, DMIAssistant, Message, config::Config, icon::FONT,
    utils::prepare_dirs,
};
use dotenv::dotenv;
use iced::{
    Font, Size, Subscription, Task,
    advanced::graphics::image::image_rs::ImageFormat,
    font, keyboard,
    window::{self, icon::from_file_data},
};
use log::{LevelFilter, error, info};

const DEFAULT_APP_LOG_LEVEL: LevelFilter = LevelFilter::Info;
const DEFAULT_LIBS_LOG_LEVEL: LevelFilter = LevelFilter::Error;

pub fn main() -> iced::Result {
    dotenv().ok();
    let config = Config::load();
    fs::create_dir_all(&config.log_dir).unwrap();
    setup_logger(&config.log_dir).expect("Logger initialization failed");
    prepare_dirs(&config);
    panic::set_hook(Box::new(|err| {
        error!(
            "[THREAD PANICKED!] Location: {:?} | Payload: {:?}",
            err.location(),
            err.payload()
        )
    }));

    info!(
        "\n\n----------------------{} v{}----------------------\n\n",
        env!("CARGO_CRATE_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    info!("Config is: {:?}", &config.log_dir);

    iced::application("DMI assistant", DMIAssistant::update, DMIAssistant::view)
        .theme(|_| DEFAULT_THEME)
        .subscription(subscription)
        .settings(iced::Settings {
            default_font: Font::MONOSPACE,
            default_text_size: 14.into(),
            antialiasing: true,
            ..Default::default()
        })
        .window(window::Settings {
            size: Size::new(1500.0, 900.0),
            position: window::Position::Centered,
            decorations: true,
            icon: from_file_data(
                include_bytes!("../assets/images/icon.png"),
                Some(ImageFormat::Png),
            )
            .ok(),
            exit_on_close_request: false,
            ..Default::default()
        })
        .font(FONT)
        .font(iced_fonts::NERD_FONT_BYTES)
        .run_with(|| (DMIAssistant::new(config), Task::none()))
}

fn subscription(_state: &DMIAssistant) -> Subscription<Message> {
    Subscription::batch(vec![
        keyboard::on_key_press(|key, modifiers| {
            Some(Message::Keyboard(key, modifiers))
        }),
        window::events().map(|(id, event)| Message::Window(id, event)),
    ])
}

pub fn settings() -> iced::Settings {
    iced::Settings {
        default_font: font::Font::MONOSPACE,
        default_text_size: 18.0.into(),
        antialiasing: true,
        ..Default::default()
    }
}

fn setup_logger<T: AsRef<Path>>(log_dir: &T) -> Result<(), fern::InitError> {
    let app_log_level: LevelFilter = env::var("APP_LOG_LEVEL")
        .unwrap_or_default()
        .parse()
        .unwrap_or(DEFAULT_APP_LOG_LEVEL);
    let libs_log_level: LevelFilter = env::var("LIBS_LOG_LEVEL")
        .unwrap_or_default()
        .parse()
        .unwrap_or(DEFAULT_LIBS_LOG_LEVEL);
    let log_file_name =
        format!("{}.log", Local::now().format("%Y-%m-%d-%H-%M-%S"));

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}] ({}) {} - {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level_for(env!("CARGO_CRATE_NAME"), app_log_level)
        .level(libs_log_level)
        .chain(std::io::stdout())
        .chain(fern::log_file(log_dir.as_ref().join(log_file_name))?)
        .apply()?;
    Ok(())
}
