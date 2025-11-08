use std::fs::{read_dir, remove_dir_all, remove_file};
use std::ops::Mul;
use std::path::PathBuf;
use std::{fs, path::Path};

use directories::ProjectDirs;
use dmi::icon::Looping;
use iced::{
    Font,
    font::Weight,
    widget::{
        self, Image, Text,
        text::{self, IntoFragment},
    },
};
use iced_toasts::{ToastLevel, toast};
use image::codecs::gif::{GifEncoder, Repeat};
use image::{Delay, DynamicImage, ImageError};
use log::{error, warn};

use crate::Message;
use crate::config::Config;

const MAX_LOGFILES_COUNT: usize = 10;

const PROJECT_QUALIFIER: &str = "com";
const PROJECT_ORGANISATION: &str = "Vlad0s";
const PROJECT_NAME: &str = "DMIAssistant";

pub fn popup<T: AsRef<str>>(
    text: T,
    custom_header: Option<&str>,
    level: ToastLevel,
) -> Message {
    Message::PushToast(
        toast(text.as_ref())
            .title(custom_header.unwrap_or(match level {
                ToastLevel::Info => "Info",
                ToastLevel::Success => "Success",
                ToastLevel::Warning => "Warning",
                ToastLevel::Error => "Error",
            }))
            .level(level)
            .into(),
    )
}

pub enum Directories {
    Log,
    Cache,
    Data,
    Config,
}

pub fn get_project_dir(dir_type: Directories) -> PathBuf {
    let dirs = ProjectDirs::from(
        PROJECT_QUALIFIER,
        PROJECT_ORGANISATION,
        PROJECT_NAME,
    )
    .unwrap();

    match dir_type {
        Directories::Log => dirs.data_dir().join("logs"),
        Directories::Cache => dirs.cache_dir().to_path_buf(),
        Directories::Data => dirs.data_local_dir().to_path_buf(),
        Directories::Config => dirs.config_local_dir().to_path_buf(),
    }
}

pub fn prepare_dirs(config: &Config) {
    // Better safe then sorry
    if config.cache_dir.ends_with("/home")
        || config.log_dir.to_string_lossy().len() < 5
    {
        panic!(
            "cache_dir is set to {} and is probably to dangerous to remove",
            config.cache_dir.to_string_lossy()
        );
    }
    if config.log_dir.ends_with("/home")
        || config.log_dir.to_string_lossy().len() < 5
    {
        panic!(
            "log_dir is set to {} and is probably to dangerous to remove",
            config.log_dir.to_string_lossy()
        );
    }
    let _ = fs::remove_dir_all(&config.cache_dir);
    fs::create_dir_all(&config.cache_dir).unwrap();
    fs::create_dir_all(&config.data_dir).unwrap();

    let mut log_files: Vec<PathBuf> = read_dir(&config.log_dir)
        .unwrap()
        .filter_map(|entry| {
            entry
                .map(|raw_entry| {
                    config.log_dir.join(
                        raw_entry.file_name().to_string_lossy().into_owned(),
                    )
                })
                .ok()
        })
        .collect();
    if log_files.len() > MAX_LOGFILES_COUNT {
        println!("{}", log_files.len());
        log_files.sort();
        let (older_files, _) =
            log_files.split_at(log_files.len() - MAX_LOGFILES_COUNT - 1);
        for older_file in older_files {
            remove_file(older_file).unwrap_or_else(|err| {
                warn!(
                    "Failed to remove old log file (as file) {}: {}",
                    older_file.to_string_lossy(),
                    err
                )
            });
            remove_dir_all(older_file).unwrap_or_else(|err| {
                warn!(
                    "Failed to remove old log file (as dir) {}: {}",
                    older_file.to_string_lossy(),
                    err
                )
            });
        }
    }
}

pub fn cleanup(config: &Config) {
    // Better safe then sorry
    if config.cache_dir.ends_with("/home")
        || config.cache_dir.to_string_lossy() == "/"
        || config.cache_dir.to_string_lossy() == ""
    {
        panic!(
            "cache_dir is set to {} and is probably to dangerous to remove",
            config.cache_dir.to_string_lossy()
        );
    }
    let _ = fs::remove_dir_all(&config.cache_dir);
}

pub fn placeholder_widget() -> Image {
    widget::image(Path::new("static").join("placeholder.jpg"))
        .height(32)
        .width(32)
}

pub fn placeholder() -> Image {
    widget::image(Path::new("static").join("placeholder.jpg"))
        .height(32)
        .width(32)
}

pub fn bold_text<'a, T, Theme>(string: T) -> Text<'a, Theme, iced::Renderer>
where
    T: IntoFragment<'a>,
    Theme: text::Catalog + 'a,
{
    Text::new(string).font(Font {
        weight: Weight::Bold,
        ..Default::default()
    })
}

pub fn animate(
    frames: Vec<DynamicImage>,
    loop_flag: &Looping,
    delay: &Option<Vec<f32>>,
) -> Result<Vec<u8>, ImageError> {
    let mut animated: Vec<u8> = Vec::new();
    let mut animated_encoder = GifEncoder::new_with_speed(&mut animated, 10);
    animated_encoder
        .set_repeat(match loop_flag {
            Looping::Indefinitely => Repeat::Infinite,
            // interesting fact - iced_gif does not support finite looping. Oopsie.
            Looping::NTimes(num) => Repeat::Finite(num.get() as u16),
        })
        .unwrap_or_else(|err| error!("Error setting repeat: {err}"));
    let result = animated_encoder.encode_frames(
        frames.into_iter().enumerate().map(|(i, frame)| {
            image::Frame::from_parts(
                frame.into_rgba8(),
                0,
                0,
                Delay::from_numer_denom_ms(
                    delay
                        .as_deref()
                        .unwrap_or_default()
                        .get(i)
                        .unwrap_or(&1.0)
                        .mul(100.0) // Delay in BYOND is measured in ticks (0.1s). In iced_gif it's measured
                        .round() as u32, //                                                         in ms (0.001s).
                    1,
                ),
            )
        }),
    );
    std::mem::drop(animated_encoder);

    result.and(Ok(animated))
}

pub fn copy_image_as_file_contents(
    image_data: &[u8],
    _filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    {
        use clipboard_win::{Clipboard, System, formats};

        Clipboard::new()?;

        let file_format = formats::register("FileContents")?;
        clipboard_win::raw::set(file_format, image_data)?;

        let filename_format = formats::register("FilenameW")?;
        let wide_filename: Vec<u16> =
            _filename.encode_utf16().chain(std::iter::once(0)).collect();
        clipboard_win::raw::set(filename_format, &wide_filename)?;

        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        use std::io::Write;
        use std::process::Command;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(image_data)?;

        let output = Command::new("osascript")
            .arg("-e")
            .arg(format!(
                "set the clipboard to (read (POSIX file \"{}\") as {{class:(«class GIF »)}})",
                temp_file.path().to_str().unwrap()
            ))
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "macOS copy failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        std::mem::forget(temp_file);
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        use std::io::Write;
        use std::process::Command;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new()
            .map_err(|err| format!("Failed to create a tempfile: {}", err))?;
        temp_file
            .write_all(image_data)
            .map_err(|err| format!("Failed to write to a tempfile: {}", err))?;
        temp_file
            .flush()
            .map_err(|err| format!("Failed to flush a tempfile: {}", err))?;

        let wayland_display = std::env::var("WAYLAND_DISPLAY").is_ok();
        let x11_display = std::env::var("DISPLAY").is_ok();

        if wayland_display {
            let output = Command::new("wl-copy")
                .arg("--type")
                .arg("image/gif")
                .arg("--paste-once")
                .arg(temp_file.path())
                .output()?;

            if !output.status.success() {
                let output = Command::new("wl-copy")
                    .arg("--type")
                    .arg("image/gif")
                    .arg(temp_file.path())
                    .output()?;

                if !output.status.success() {
                    return Err(format!(
                        "wl-copy failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    )
                    .into());
                }
            }
        } else if x11_display {
            let output = Command::new("timeout")
                .arg("5s")
                .arg("xclip")
                .arg("-selection")
                .arg("clipboard")
                .arg("-t")
                .arg("image/gif")
                .arg("-i")
                .arg(temp_file.path())
                .output()?;

            if !output.status.success() {
                return Err(format!(
                    "xclip failed or timed out: {}",
                    String::from_utf8_lossy(&output.stderr)
                )
                .into());
            }
        } else {
            // No display server detected
            return Err(
                "No display server detected (neither Wayland nor X11)".into()
            );
        }

        Ok(())
    }

    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "linux"
    )))]
    {
        use arboard::{Clipboard, ImageData};
        use std::borrow::Cow;

        let mut clipboard = Clipboard::new()?;

        clipboard.set_image(ImageData {
            width: 64,
            height: 64,
            bytes: Cow::Borrowed(image_data),
        })?;

        Ok(())
    }
}
