use std::fmt::Display;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

use dmi::dirs::Dirs;
use dmi::icon::Icon;
use image::imageops::FilterType;
use thiserror::Error;

/// Errors, returned by DMIs parsing.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum DMIParsingError {
    /// File is not found or inaccesible.
    #[error(transparent)]
    NoSuchFile(#[from] io::Error),
    /// This file can not be made into raw DMI.
    #[error(transparent)]
    ErrorDMI(#[from] dmi::error::DmiError),
    /// Error parsing state into RGBA
    #[error("Error parsing state into RGBA")]
    ErrorRGBA,
    /// Error parsing into displayable ParsedDMI
    #[error("Error parsing into displayable ParsedDMI")]
    ErrorParsing,
    /// Other image parsing errors
    #[error(transparent)]
    ImageError(#[from] image::ImageError),
}

pub fn load_and_save_dmi(
    input_file: &String,
    name: &String,
    output_file: &PathBuf,
) -> Result<(), DMIParsingError> {
    let icon = load_dmi(input_file)?;
    for state in icon.states {
        if &state.name == name {
            if let Some(image) = state.images.first() {
                image
                    .as_rgba8()
                    .ok_or(DMIParsingError::ErrorRGBA)?
                    .save(output_file)?;
            }
        }
    }
    Ok(())
}

pub fn load_dmi<T: AsRef<Path>>(
    input_file: T,
) -> Result<Icon, DMIParsingError> {
    Ok(Icon::load(File::open(input_file)?)?)
}

#[derive(Debug, Clone, Copy, Hash, PartialOrd, Ord, Eq, PartialEq)]
pub enum Directions {
    South = 0,
    North = 1,
    East = 2,
    West = 3,
    SouthEast = 4,
    SouthWest = 5,
    NorthEast = 6,
    NorthWest = 7,
}

impl Display for Directions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text_repr = match self {
            Directions::South => "South",
            Directions::North => "North",
            Directions::East => "East",
            Directions::West => "West",
            Directions::SouthEast => "SouthEast",
            Directions::SouthWest => "SouthWest",
            Directions::NorthEast => "NorthEast",
            Directions::NorthWest => "NorthWest",
        };
        write!(f, "{}", text_repr)
    }
}

impl From<u8> for Directions {
    fn from(value: u8) -> Self {
        match value {
            0 => Directions::South,
            1 => Directions::North,
            2 => Directions::East,
            3 => Directions::West,
            4 => Directions::SouthEast,
            5 => Directions::SouthWest,
            6 => Directions::NorthEast,
            7 => Directions::NorthWest,
            _ => Directions::South,
        }
    }
}

impl From<Directions> for Dirs {
    fn from(value: Directions) -> Self {
        match value {
            Directions::South => Dirs::SOUTH,
            Directions::North => Dirs::NORTH,
            Directions::East => Dirs::EAST,
            Directions::West => Dirs::WEST,
            Directions::SouthEast => Dirs::SOUTHEAST,
            Directions::SouthWest => Dirs::SOUTHWEST,
            Directions::NorthEast => Dirs::NORTHEAST,
            Directions::NorthWest => Dirs::NORTHWEST,
        }
    }
}

impl From<&Directions> for Dirs {
    fn from(value: &Directions) -> Self {
        match value {
            Directions::South => Dirs::SOUTH,
            Directions::North => Dirs::NORTH,
            Directions::East => Dirs::EAST,
            Directions::West => Dirs::WEST,
            Directions::SouthEast => Dirs::SOUTHEAST,
            Directions::SouthWest => Dirs::SOUTHWEST,
            Directions::NorthEast => Dirs::NORTHEAST,
            Directions::NorthWest => Dirs::NORTHWEST,
        }
    }
}

impl From<Directions> for u8 {
    fn from(value: Directions) -> u8 {
        match value {
            Directions::South => 0,
            Directions::North => 1,
            Directions::East => 2,
            Directions::West => 3,
            Directions::SouthEast => 4,
            Directions::SouthWest => 5,
            Directions::NorthEast => 6,
            Directions::NorthWest => 7,
        }
    }
}

/// Filter type with derived display to satisfy iced
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CustomFilterType {
    /// Nearest Neighbor
    #[default]
    Nearest,

    /// Linear Filter
    Triangle,

    /// Cubic Filter
    CatmullRom,

    /// Gaussian Filter
    Gaussian,

    /// Lanczos with window 3
    Lanczos3,
}

impl From<CustomFilterType> for FilterType {
    fn from(value: CustomFilterType) -> Self {
        match value {
            CustomFilterType::Nearest => Self::Nearest,
            CustomFilterType::Triangle => Self::Triangle,
            CustomFilterType::CatmullRom => Self::CatmullRom,
            CustomFilterType::Gaussian => Self::Gaussian,
            CustomFilterType::Lanczos3 => Self::Lanczos3,
        }
    }
}

impl Display for CustomFilterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Nearest => "Nearest Neighbor",
            Self::Triangle => "Linear Filter",
            Self::CatmullRom => "Cubic Filter",
            Self::Gaussian => "Gaussian Filter",
            Self::Lanczos3 => "Lanczos with window 3",
        })
    }
}
