// Generated automatically by iced_fontello at build time.
// Do not edit manually. Source: ../assets/fonts/fontello.toml
// b9342d19c700b770d34da38c7a8feb87ccca799c73b726ca09fbccd1f53ffa2a
use iced::Font;
use iced::widget::{Text, text};

pub const FONT: &[u8] = include_bytes!("../assets/fonts/fontello.ttf");

pub fn edit<'a>() -> Text<'a> {
    icon("\u{270E}")
}

pub fn folder<'a>() -> Text<'a> {
    icon("\u{1F4C2}")
}

pub fn iconfile<'a>() -> Text<'a> {
    icon("\u{F1C5}")
}

pub fn open<'a>() -> Text<'a> {
    icon("\u{F15C}")
}

pub fn palette<'a>() -> Text<'a> {
    icon("\u{1F3A8}")
}

pub fn save<'a>() -> Text<'a> {
    icon("\u{1F4BE}")
}

pub fn search<'a>() -> Text<'a> {
    icon("\u{1F50D}")
}

pub fn settings<'a>() -> Text<'a> {
    icon("\u{26EF}")
}

pub fn text_cursor<'a>() -> Text<'a> {
    icon("\u{F246}")
}

pub fn trash<'a>() -> Text<'a> {
    icon("\u{E10A}")
}

fn icon(codepoint: &str) -> Text<'_> {
    text(codepoint).font(Font::with_name("fontello"))
}
