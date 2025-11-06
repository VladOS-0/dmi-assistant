use std::ops::Mul;
use std::{fs, path::Path};

use dmi::icon::Looping;
use iced::{
    font::Weight,
    widget::{
        self,
        text::{self, IntoFragment},
        Image, Text,
    },
    Font,
};
use iced_toasts::{toast, ToastLevel};
use image::codecs::gif::{GifEncoder, Repeat};
use image::{Delay, DynamicImage, ImageError};

use crate::Message;

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

pub fn cleanup() {
    let _ = fs::remove_dir_all("temp");
}

pub fn init_temp() {
    // if we can't just create folders, we are fucked up already
    fs::create_dir_all("temp").unwrap();
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
        .unwrap_or_else(|err| eprintln!("Error setting repeat: {err}"));
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
