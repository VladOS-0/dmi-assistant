use std::collections::{BTreeMap, HashMap};

use dmi::icon::{Icon, IconState, Looping};
use iced_gif::Frames;
use image::{imageops::FilterType, DynamicImage};

use crate::{
    dmi_utils::Directions, screens::debugger::StateboxResizing, utils::animate,
};

#[derive(Debug, Clone, Default)]
pub struct ParsedDMI {
    pub original_height: u32,
    pub original_width: u32,

    pub displayed_height: u32,
    pub displayed_width: u32,

    pub states: HashMap<String, ParsedState>,
}

impl ParsedDMI {
    pub fn parse_from_raw(
        raw: Icon,
        resizing: StateboxResizing,
        filter_type: FilterType,
    ) -> Self {
        let original_height = raw.height;
        let original_width = raw.width;

        let displayed_height;
        let displayed_width;
        let new_resizing;

        match resizing {
            StateboxResizing::Original => {
                displayed_height = original_height;
                displayed_width = original_width;
                new_resizing = resizing;
            }
            StateboxResizing::Resized { height, width } => {
                if height > original_height && width > original_width {
                    displayed_height = height;
                    displayed_width = width;
                    new_resizing = resizing;
                } else if height > original_height {
                    displayed_height = height;
                    displayed_width = original_width;
                    new_resizing = StateboxResizing::Resized {
                        height,
                        width: original_width,
                    }
                } else if width > original_width {
                    displayed_height = original_height;
                    displayed_width = width;
                    new_resizing = StateboxResizing::Resized {
                        height: original_height,
                        width,
                    }
                } else {
                    displayed_height = original_height;
                    displayed_width = original_width;
                    new_resizing = StateboxResizing::Original;
                }
            }
        }

        let states: HashMap<String, ParsedState> = raw
            .states
            .into_iter()
            .map(|state| {
                (
                    state.name.clone(),
                    ParsedState::parse_from_raw(
                        state,
                        new_resizing,
                        filter_type,
                    ),
                )
            })
            .collect();

        Self {
            original_height,
            original_width,
            displayed_height,
            displayed_width,
            states,
        }
    }

    pub fn resize(
        &mut self,
        resizing: StateboxResizing,
        filter_type: FilterType,
    ) {
        let new_resizing;

        match resizing {
            StateboxResizing::Original => {
                self.displayed_height = self.original_height;
                self.displayed_width = self.original_width;
                new_resizing = resizing;
            }
            StateboxResizing::Resized { height, width } => {
                if height > self.original_height && width > self.original_width
                {
                    self.displayed_height = height;
                    self.displayed_width = width;
                    new_resizing = resizing;
                } else if height > self.original_height {
                    self.displayed_height = height;
                    self.displayed_width = self.original_width;
                    new_resizing = StateboxResizing::Resized {
                        height,
                        width: self.original_width,
                    }
                } else if width > self.original_width {
                    self.displayed_height = self.original_height;
                    self.displayed_width = width;
                    new_resizing = StateboxResizing::Resized {
                        height: self.original_height,
                        width,
                    }
                } else {
                    self.displayed_height = self.original_height;
                    self.displayed_width = self.original_width;
                    new_resizing = StateboxResizing::Original;
                }
            }
        }

        match new_resizing {
            StateboxResizing::Original => {}
            _ => {
                for state in &mut self.states {
                    state.1.resize(new_resizing, filter_type);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ParsedState {
    pub name: String,

    pub delay: Option<Vec<f32>>,
    pub loop_flag: Looping,
    pub rewind: bool,
    pub frames: u32,
    pub movement: bool,

    pub dirs: BTreeMap<Directions, DirImage>,
}

impl ParsedState {
    pub fn parse_from_raw(
        state: IconState,
        resizing: StateboxResizing,
        filter_type: FilterType,
    ) -> Self {
        let mut dirs: BTreeMap<Directions, DirImage> = BTreeMap::new();
        for dir_index in 0..state.dirs {
            let direction: Directions = dir_index.into();
            let dir_image = DirImage::parse_from_raw(
                &state,
                &resizing,
                direction,
                state.frames,
                state.loop_flag,
                filter_type,
            );
            dirs.insert(direction, dir_image);
        }

        Self {
            name: state.name,
            delay: state.delay,
            loop_flag: state.loop_flag,
            rewind: state.rewind,
            frames: state.frames,
            movement: state.movement,
            dirs,
        }
    }

    pub fn resize(
        &mut self,
        resizing: StateboxResizing,
        filter_type: FilterType,
    ) {
        for dir in &mut self.dirs {
            dir.1
                .resize(self.loop_flag, &self.delay, resizing, filter_type);
        }
    }

    pub fn get_frame(
        &self,
        dir: &Directions,
        frame: usize,
    ) -> Option<&DynamicImage> {
        self.dirs.get(dir)?.get_frame(frame)
    }
    pub fn get_original_frame(
        &self,
        dir: &Directions,
        frame: usize,
    ) -> Option<&DynamicImage> {
        self.dirs.get(dir)?.get_original_frame(frame)
    }

    pub fn get_animated(&self, dir: &Directions) -> Option<&Animated> {
        self.dirs.get(dir)?.get_animated()
    }

    pub fn get_original_animated(&self, dir: &Directions) -> Option<&Animated> {
        self.dirs.get(dir)?.get_original_animated()
    }
}

#[derive(Debug, Clone, Default)]
pub struct DirImage {
    pub resized_frames: Option<Vec<DynamicImage>>,
    pub original_frames: Vec<DynamicImage>,

    pub resized_animated: Option<Animated>,
    pub original_animated: Option<Animated>,
}

impl DirImage {
    pub fn parse_from_raw(
        state: &IconState,
        resizing: &StateboxResizing,
        direction: Directions,
        frame_num: u32,
        loop_flag: Looping,
        filter_type: FilterType,
    ) -> Self {
        let mut original_frames: Vec<DynamicImage> =
            Vec::with_capacity(frame_num as usize);

        for frame in 0..frame_num {
            // I was forced to write it, because dmi crate's get_image is broken
            // All hail stupidity
            let frame = state
                .images
                .get(direction as usize + frame as usize * state.dirs as usize);
            if frame.is_none() {
                eprintln!("{frame:?}");
                break;
            }
            let frame: &DynamicImage = frame.unwrap();
            original_frames.push(frame.clone());
        }

        if original_frames.is_empty() {
            return Self::default();
        }
        let animated =
            animate(original_frames.clone(), &loop_flag, &state.delay)
                .map_err(|err| {
                    eprintln!("{err}");
                    err
                })
                .ok();
        let animated = match animated {
            Some(vec) => Animated::new(vec).ok(),
            None => None,
        };
        match resizing {
            StateboxResizing::Original => Self {
                resized_frames: None,
                original_frames,
                resized_animated: None,
                original_animated: animated,
            },
            StateboxResizing::Resized { height, width } => {
                let resized_frames: Vec<DynamicImage> = original_frames
                    .iter()
                    .map(|frame| frame.resize(*width, *height, filter_type))
                    .collect();
                let resized_animated =
                    animate(resized_frames.clone(), &loop_flag, &state.delay)
                        .map_err(|err| {
                            eprintln!("{err}");
                            err
                        })
                        .ok();
                let resized_animated = match resized_animated {
                    Some(vec) => Animated::new(vec).ok(),
                    None => None,
                };
                Self {
                    resized_frames: Some(resized_frames),
                    original_frames,
                    resized_animated,
                    original_animated: animated,
                }
            }
        }
    }

    pub fn resize(
        &mut self,
        loop_flag: Looping,
        delay: &Option<Vec<f32>>,
        resizing: StateboxResizing,
        filter_type: FilterType,
    ) {
        match resizing {
            StateboxResizing::Original => unreachable!(),
            StateboxResizing::Resized { height, width } => {
                let resized_frames: Vec<DynamicImage> = self
                    .original_frames
                    .iter()
                    .map(|frame| frame.resize(width, height, filter_type))
                    .collect();
                let resized_animated =
                    animate(resized_frames.clone(), &loop_flag, delay)
                        .map_err(|err| {
                            eprintln!("{err}");
                            err
                        })
                        .ok();
                let resized_animated = match resized_animated {
                    Some(vec) => Animated::new(vec).ok(),
                    None => None,
                };
                self.resized_animated = resized_animated;
                self.resized_frames = Some(resized_frames);
            }
        }
    }

    pub fn get_frame(&self, frame: usize) -> Option<&DynamicImage> {
        if let Some(frames) = &self.resized_frames {
            frames.get(frame)
        } else {
            self.original_frames.get(frame)
        }
    }

    pub fn get_original_frame(&self, frame: usize) -> Option<&DynamicImage> {
        self.original_frames.get(frame)
    }

    pub fn get_animated(&self) -> Option<&Animated> {
        if let Some(animated) = &self.resized_animated {
            Some(animated)
        } else {
            self.original_animated.as_ref()
        }
    }

    pub fn get_original_animated(&self) -> Option<&Animated> {
        self.original_animated.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct Animated {
    pub bytes: Vec<u8>,
    pub frames: Frames,
}

impl Animated {
    pub fn new(bytes: Vec<u8>) -> Result<Self, iced_gif::gif::Error> {
        let frames = Frames::from_bytes(bytes.clone())?;
        Ok(Self { bytes, frames })
    }
}
