use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::time::Instant;

use arboard::Clipboard;
use arboard::ImageData;
use dmi::icon::Icon;
use iced::Alignment;
use iced::Background;
use iced::Border;
use iced::Color;
use iced::Element;
use iced::Length;
use iced::Shadow;
use iced::Task;
use iced::alignment::Horizontal;
use iced::alignment::Vertical;
use iced::border::Radius;
use iced::widget::Button;
use iced::widget::Column;
use iced::widget::Container;
use iced::widget::Image;
use iced::widget::Scrollable;
use iced::widget::Space;
use iced::widget::Toggler;
use iced::widget::button;
use iced::widget::column;
use iced::widget::container;
use iced::widget::container::Style;
use iced::widget::pick_list;
use iced::widget::row;
use iced::widget::scrollable;
use iced::widget::scrollable::Direction;
use iced::widget::scrollable::Scrollbar;
use iced::widget::text;
use iced::widget::text_input;
use iced::widget::toggler;
use iced_aw::ColorPicker;
use iced_aw::Grid;
use iced_aw::GridRow;
use iced_aw::NumberInput;
use iced_aw::TabLabel;
use iced_aw::Wrap;
use iced_aw::color_picker;
use iced_gif::Gif;
use iced_toasts::ToastLevel;
use image::imageops::FilterType;
use log::debug;
use log::warn;
use rfd::FileDialog;

use super::Screen;

use crate::DMIAssistant;
use crate::Message;
use crate::dmi_model::ParsedDMI;
use crate::dmi_utils::CustomFilterType;
use crate::dmi_utils::Directions;
use crate::dmi_utils::load_dmi;
use crate::icon;
use crate::utils::bold_text;
use crate::utils::popup;
use crate::wrap;

#[derive(Debug, Clone)]
pub enum ViewerMessage {
    ChangeDMIPath(String),
    LoadDMI,
    DMILoaded(Result<(Icon, ParsedDMI), String>),
    OpenedFileExplorer,
    CopyImage(String, bool, bool, Directions, Option<usize>),

    ToggleSettingsVisibility(bool),

    ToggleDebug(bool),
    ToggleAnimated(bool),
    ToggleResizeDisplay(bool),
    ChangeResize(StateboxResizing),
    ChangeFilterType(CustomFilterType),
    PerformResize,

    ColorPickerOpened(ColorPickerType),
    ColorPickerClosed(ColorPickerType),
    ColorChange(ColorPickerType, Color),
}

#[derive(Default, Debug, Clone)]
pub struct ViewerScreen {
    pub dmi_path: String,
    pub dmi_raw_icon: Icon,
    pub parsed_dmi: ParsedDMI,

    pub loading_dmi_in_progress: bool,
    pub hovered_file: bool,

    pub settings_visible: bool,

    pub color_picker_statebox_visible: bool,
    pub color_picker_text_visible: bool,

    pub display_settings: DisplaySettings,
}

impl Screen for ViewerScreen {
    fn label(&self) -> TabLabel {
        TabLabel::IconText('\u{F1C5}', " Viewer".to_string())
    }

    fn update(app: &mut DMIAssistant, message: Message) -> Task<Message> {
        let screen = &mut app.viewer_screen;
        if let Message::ViewerMessage(screen_message) = message {
            match screen_message {
                ViewerMessage::ChangeDMIPath(path) => {
                    screen.dmi_path = path;
                    Task::none()
                }
                ViewerMessage::LoadDMI => {
                    screen.loading_dmi_in_progress = true;
                    let path = screen.dmi_path.clone();
                    let filter_type: FilterType = screen
                        .display_settings
                        .statebox_default
                        .filter_type
                        .unwrap_or_default()
                        .into();

                    let resize =
                        screen.display_settings.statebox_default.resize;

                    Task::future(async move {
                        let load_start = Instant::now();
                        let opened_dmi = load_dmi(&path);
                        if opened_dmi.is_err() {
                            return wrap![ViewerMessage::DMILoaded(Err(
                                format!("{}", opened_dmi.unwrap_err())
                            ))];
                        }
                        let opened_dmi = opened_dmi.unwrap();

                        let parsed_dmi = ParsedDMI::parse_from_raw(
                            opened_dmi.clone(),
                            resize,
                            filter_type,
                        );
                        debug!(
                            "DMI {} parsed in {}ms",
                            path,
                            load_start.elapsed().as_millis()
                        );
                        wrap![ViewerMessage::DMILoaded(Ok((
                            opened_dmi, parsed_dmi
                        )))]
                    })
                }
                ViewerMessage::DMILoaded(result) => {
                    if let Err(err) = result {
                        warn!("[VIEWER] Failed to load DMI: {err}");
                        screen.loading_dmi_in_progress = false;
                        return Task::done(popup(
                            format!("Failed to load DMI: {}", err),
                            Some("Failed to load DMI"),
                            ToastLevel::Error,
                        ));
                    }
                    let (raw, parsed) = result.unwrap();
                    screen.dmi_raw_icon = raw;
                    screen.parsed_dmi = parsed;
                    screen.loading_dmi_in_progress = false;
                    Task::done(popup(
                        "Successfully loaded DMI",
                        Some("Loaded"),
                        ToastLevel::Success,
                    ))
                }
                ViewerMessage::OpenedFileExplorer => {
                    let file = FileDialog::new()
                        .add_filter("dmi", &["dmi"])
                        .set_directory("/")
                        .pick_file()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or("")
                        .to_string();

                    if !file.is_empty() {
                        Task::done(wrap![ViewerMessage::ChangeDMIPath(file)])
                            .chain(Task::done(wrap![ViewerMessage::LoadDMI]))
                    } else {
                        Task::none()
                    }
                }
                ViewerMessage::ColorPickerOpened(picker) => {
                    match picker {
                        ColorPickerType::DefaultStateboxColor => {
                            screen.color_picker_statebox_visible = true
                        }
                        ColorPickerType::DefaultTextColor => {
                            screen.color_picker_text_visible = true
                        }
                    }
                    Task::none()
                }
                ViewerMessage::ColorPickerClosed(picker) => {
                    match picker {
                        ColorPickerType::DefaultStateboxColor => {
                            screen.color_picker_statebox_visible = false
                        }
                        ColorPickerType::DefaultTextColor => {
                            screen.color_picker_text_visible = false
                        }
                    }
                    Task::none()
                }
                ViewerMessage::ColorChange(picker, color) => {
                    match picker {
                        ColorPickerType::DefaultStateboxColor => {
                            screen
                                .display_settings
                                .statebox_default
                                .background_color = color;
                            screen.color_picker_statebox_visible = false
                        }
                        ColorPickerType::DefaultTextColor => {
                            screen
                                .display_settings
                                .statebox_default
                                .text_color = color;
                            screen.color_picker_text_visible = false
                        }
                    }
                    Task::none()
                }
                ViewerMessage::ToggleSettingsVisibility(visible) => {
                    screen.settings_visible = visible;
                    screen.color_picker_statebox_visible = false;
                    screen.color_picker_text_visible = false;
                    Task::none()
                }
                ViewerMessage::ToggleDebug(active) => {
                    screen.display_settings.statebox_default.debug = active;
                    Task::none()
                }
                ViewerMessage::ToggleAnimated(active) => {
                    screen.display_settings.statebox_default.animated = active;
                    Task::none()
                }
                ViewerMessage::ToggleResizeDisplay(active) => {
                    screen.display_settings.statebox_default.show_resized =
                        active;
                    Task::none()
                }
                ViewerMessage::ChangeResize(resizing) => {
                    screen.display_settings.statebox_default.resize = resizing;
                    Task::none()
                }
                ViewerMessage::ChangeFilterType(filter_type) => {
                    screen.display_settings.statebox_default.filter_type =
                        Some(filter_type);
                    Task::none()
                }
                ViewerMessage::PerformResize => {
                    screen.parsed_dmi.resize(
                        screen.display_settings.statebox_default.resize,
                        screen
                            .display_settings
                            .statebox_default
                            .filter_type
                            .unwrap_or_default()
                            .into(),
                    );
                    Task::done(popup(
                        format!(
                            "Performed resize to {:#?} with filter {:#?}",
                            screen.display_settings.statebox_default.resize,
                            screen
                                .display_settings
                                .statebox_default
                                .filter_type
                        ),
                        Some("Resized"),
                        ToastLevel::Success,
                    ))
                }

                ViewerMessage::CopyImage(
                    state,
                    animated,
                    original,
                    direction,
                    frame,
                ) => {
                    let state = screen.parsed_dmi.states.get(&state);
                    if state.is_none() {
                        return Task::none();
                    }
                    let state = state.unwrap();

                    let image_bytes;

                    if animated {
                        if original {
                            let anim = state.get_original_frame(&direction, 1);
                            if let Some(animated) = anim {
                                image_bytes = animated.as_bytes().to_vec();
                            } else {
                                image_bytes = Vec::new();
                            }
                        } else {
                            let anim = state.get_frame(&direction, 1);
                            if let Some(animated) = anim {
                                image_bytes = animated.as_bytes().to_vec();
                            } else {
                                image_bytes = Vec::new();
                            }
                        }
                    } else if original {
                        let icon = state
                            .get_original_frame(&direction, frame.unwrap());
                        if let Some(image) = icon {
                            image_bytes = image.clone().into_bytes();
                        } else {
                            image_bytes = Vec::new();
                        }
                    } else {
                        let icon = state.get_frame(&direction, frame.unwrap());
                        if let Some(image) = icon {
                            image_bytes = image.clone().into_bytes();
                        } else {
                            image_bytes = Vec::new();
                        }
                    }
                    if image_bytes.is_empty() {
                        return Task::none();
                    }

                    let height = if original {
                        screen.parsed_dmi.original_height
                    } else {
                        screen.parsed_dmi.displayed_height
                    };

                    let width = if original {
                        screen.parsed_dmi.original_width
                    } else {
                        screen.parsed_dmi.displayed_width
                    };

                    let _ = Clipboard::new().unwrap().set_image(ImageData {
                        width: width as usize,
                        height: height as usize,
                        bytes: Cow::Borrowed(&image_bytes),
                    });

                    Task::done(popup(
                        "Copied image to the clipboard",
                        Some("Copied"),
                        ToastLevel::Success,
                    ))
                }
            }
        } else if let Message::Window(_id, event) = message {
            match event {
                iced::window::Event::FileHovered(_) => {
                    screen.hovered_file = true;
                    Task::none()
                }
                iced::window::Event::FilesHoveredLeft => {
                    screen.hovered_file = false;
                    Task::none()
                }
                iced::window::Event::FileDropped(path) => {
                    screen.dmi_path = path
                        .to_str()
                        .unwrap_or("FAILED TO RESOLVE FILE")
                        .to_owned();
                    screen.hovered_file = false;
                    Task::done(wrap![ViewerMessage::LoadDMI])
                }

                _ => Task::none(),
            }
        } else {
            Task::none()
        }
    }

    fn view<'a>(app: &'a DMIAssistant) -> Element<'a, Message> {
        let screen = &app.viewer_screen;
        /*
         *
         * PLACEHOLDERS
         *
         */
        if screen.hovered_file {
            return container("Drop your file here")
                .style(container::bordered_box)
                .padding(50)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into();
        }

        if screen.loading_dmi_in_progress {
            return container(text!("Loading {}...", screen.dmi_path))
                .style(container::bordered_box)
                .padding(50)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into();
        }
        /*
         *
         * PATH INPUT
         *
         */
        let input_path = text_input("Input DMI path", &screen.dmi_path)
            .on_input(|input| wrap![ViewerMessage::ChangeDMIPath(input)])
            .on_paste(|input| wrap![ViewerMessage::ChangeDMIPath(input)])
            .on_submit(wrap![ViewerMessage::LoadDMI])
            .padding(10);

        let button_load: Button<Message> =
            button(row![icon::open(), text(" Open File")])
                .on_press(wrap![ViewerMessage::LoadDMI]);

        let button_explorer: Button<Message> =
            button(row![icon::iconfile(), text(" Browse Files")])
                .on_press(wrap![ViewerMessage::OpenedFileExplorer]);

        let settings_button: Button<Message> = button(icon::settings())
            .on_press(wrap![ViewerMessage::ToggleSettingsVisibility(
                !screen.settings_visible
            )]);

        let input_bar =
            row![settings_button, input_path, button_load, button_explorer]
                .spacing(10)
                .align_y(Vertical::Center)
                .padding(5);

        /*
         *
         * SETTINGS
         *
         */
        let mut settings_bar: Column<Message> = Column::new();
        if screen.settings_visible {
            let stateboxes_color_picker: ColorPicker<Message> = color_picker(
                screen.color_picker_statebox_visible,
                screen.display_settings.statebox_default.background_color,
                Button::new(row![
                    icon::palette(),
                    text(" Set Stateboxes Color")
                ])
                .on_press(wrap![
                    ViewerMessage::ColorPickerOpened(
                        ColorPickerType::DefaultStateboxColor
                    )
                ]),
                wrap![ViewerMessage::ColorPickerClosed(
                    ColorPickerType::DefaultStateboxColor
                )],
                |color| {
                    wrap![ViewerMessage::ColorChange(
                        ColorPickerType::DefaultStateboxColor,
                        color
                    )]
                },
            );

            let text_color_picker: ColorPicker<Message> = color_picker(
                screen.color_picker_text_visible,
                screen.display_settings.statebox_default.background_color,
                Button::new(row![icon::text_cursor(), text(" Set Text Color")])
                    .on_press(wrap![ViewerMessage::ColorPickerOpened(
                        ColorPickerType::DefaultTextColor
                    )]),
                wrap![ViewerMessage::ColorPickerClosed(
                    ColorPickerType::DefaultTextColor
                )],
                |color| {
                    wrap![ViewerMessage::ColorChange(
                        ColorPickerType::DefaultTextColor,
                        color
                    )]
                },
            );

            let debug_info_toggler: Toggler<Message> =
                toggler(screen.display_settings.statebox_default.debug)
                    .label("Debug Info")
                    .on_toggle(|state| {
                        wrap![ViewerMessage::ToggleDebug(state)]
                    });

            let animated_toggler: Toggler<Message> =
                toggler(screen.display_settings.statebox_default.animated)
                    .label("Animated View")
                    .on_toggle(|state| {
                        wrap![ViewerMessage::ToggleAnimated(state)]
                    });
            let resizing_display_toggler: Toggler<Message> =
                toggler(screen.display_settings.statebox_default.show_resized)
                    .label("Show resized images")
                    .on_toggle(|state| {
                        wrap![ViewerMessage::ToggleResizeDisplay(state)]
                    });
            let resize_toggler: Toggler<Message> = toggler(
                screen.display_settings.statebox_default.resize
                    != StateboxResizing::Original,
            )
            .label("Resize images")
            .on_toggle(|state| {
                if state {
                    wrap![ViewerMessage::ChangeResize(
                        StateboxResizing::default()
                    )]
                } else {
                    wrap![ViewerMessage::ChangeResize(
                        StateboxResizing::Original
                    )]
                }
            });
            let resize_picker = match screen
                .display_settings
                .statebox_default
                .resize
            {
                StateboxResizing::Original => container(""),
                StateboxResizing::Resized { height, width } => {
                    let height_number_picker: NumberInput<u32, Message> =
                        NumberInput::new(height, 32..=512, move |new_height| {
                            wrap![ViewerMessage::ChangeResize(
                                StateboxResizing::Resized {
                                    height: new_height,
                                    width,
                                }
                            )]
                        })
                        .step(16);
                    let width_number_picker: NumberInput<u32, Message> =
                        NumberInput::new(width, 32..=512, move |new_width| {
                            wrap![ViewerMessage::ChangeResize(
                                StateboxResizing::Resized {
                                    height,
                                    width: new_width,
                                }
                            )]
                        })
                        .step(16);

                    let filter_types = [
                        CustomFilterType::Nearest,
                        CustomFilterType::Triangle,
                        CustomFilterType::CatmullRom,
                        CustomFilterType::Gaussian,
                        CustomFilterType::Lanczos3,
                    ];

                    let filter_type_picker = pick_list(
                        filter_types,
                        screen.display_settings.statebox_default.filter_type,
                        |filter_type| {
                            wrap![ViewerMessage::ChangeFilterType(filter_type)]
                        },
                    )
                    .placeholder("Select filter type...");

                    container(
                        column![
                            row![
                                text("Resize up to height: "),
                                height_number_picker
                            ],
                            row![
                                text("Resize up to width: "),
                                width_number_picker
                            ],
                            filter_type_picker
                        ]
                        .spacing(10),
                    )
                }
            };

            let resize_button: Button<Message> =
                button("Resize").on_press(wrap![ViewerMessage::PerformResize]);

            settings_bar = column![
                row![stateboxes_color_picker, text_color_picker].spacing(10),
                debug_info_toggler,
                animated_toggler,
                resizing_display_toggler,
                resize_toggler,
                resize_picker,
                resize_button
            ]
            .spacing(10);
        }

        //
        //
        // STATES
        //
        //

        let mut states_wrap = Wrap::new()
            .align_items(Alignment::Start)
            .spacing(10)
            .line_spacing(10);

        for state in &screen.parsed_dmi.states {
            states_wrap = states_wrap.push(screen.display_statebox(state.0))
        }

        let column = column![input_bar, settings_bar, states_wrap]
            .padding(10)
            .spacing(10);

        container(scrollable(column).spacing(10)).padding(10).into()
    }
}

impl ViewerScreen {
    fn get_statebox_settings(
        &self,
        statebox_name: &String,
    ) -> &StateboxSettings {
        self.display_settings
            .unique_stateboxes
            .get(statebox_name)
            .unwrap_or(&self.display_settings.statebox_default)
    }

    fn display_statebox<'a>(
        &'a self,
        state_name: &String,
    ) -> Container<'a, Message> {
        let state = self.parsed_dmi.states.get(state_name);
        if state.is_none() {
            return container(text!(
                "State {} does not exist. It's probably a bug.",
                state_name
            ));
        }
        let state = state.unwrap();
        let settings = self.get_statebox_settings(state_name);
        let header: Column<Message> = if settings.debug {
            column![
                Space::new(1, 3),
                row![text("State: "), bold_text(state.name.clone())],
                Space::new(1, 3),
                text!("Delay: {:?}", state.delay),
                text!("Frames: {}", state.frames),
                text!("Directions: {}", state.dirs.len()),
                text!("Looping: {:?}", state.loop_flag),
                text!("Movement: {}", state.movement),
                text!("Rewind: {}", state.rewind),
                Space::new(1, 10)
            ]
            .padding(5)
            .spacing(5)
        } else {
            column![bold_text(state.name.clone()), Space::new(1, 10)]
                .padding(5)
                .spacing(5)
                .align_x(Horizontal::Center)
        };

        let display: Grid<Message> = {
            let mut dirs: VecDeque<GridRow<Message>> = state
                .dirs
                .keys()
                .map(|direction| {
                    let mut row: GridRow<Message> = GridRow::default();
                    row = row.push(text(direction.to_string()));
                    if settings.animated {
                        let animated = {
                            if settings.show_resized {
                                state.get_animated(direction)
                            } else {
                                state.get_original_animated(direction)
                            }
                        };
                        if let Some(gif) = animated {
                            let gif = Gif::new(&gif.frames);
                            let gif = button(gif)
                                .on_press(wrap![ViewerMessage::CopyImage(
                                    state.name.clone(),
                                    true,
                                    settings.show_resized,
                                    *direction,
                                    None
                                )])
                                .style(|_theme, _status| button::Style {
                                    background: None,
                                    ..Default::default()
                                });
                            row = row.push(gif);
                        }
                    } else {
                        for frame in 0..state.frames {
                            let icon = {
                                if settings.show_resized {
                                    state.get_frame(direction, frame as usize)
                                } else {
                                    state.get_original_frame(
                                        direction,
                                        frame as usize,
                                    )
                                }
                            };
                            if let Some(icon) = icon {
                                let image_widget: Image = Image::new(
                                    iced::widget::image::Handle::from_rgba(
                                        icon.width(),
                                        icon.height(),
                                        icon.clone().into_bytes(),
                                    ),
                                );
                                let image_widget = button(image_widget)
                                    .on_press(wrap![ViewerMessage::CopyImage(
                                        state.name.clone(),
                                        false,
                                        settings.show_resized,
                                        *direction,
                                        Some(frame as usize)
                                    )])
                                    .style(|_theme, _status| button::Style {
                                        background: None,
                                        ..Default::default()
                                    });
                                row = row.push(image_widget);
                            } else {
                                row = row.push(text("?"));
                            }
                        }
                    }
                    row
                })
                .collect();
            if !settings.animated && state.frames > 1 {
                let mut delay_row: GridRow<Message> = GridRow::new();
                delay_row = delay_row.push(text("Delay"));
                for delay in state.delay.as_ref().unwrap_or(&Vec::new()) {
                    delay_row = delay_row.push(text(delay))
                }
                dirs.push_front(delay_row);
            }
            Grid::with_rows(dirs.into())
                .column_width(self.parsed_dmi.displayed_width as f32 * 1.2)
                .horizontal_alignment(Horizontal::Center)
                .spacing(10)
        };

        let display = Scrollable::with_direction(
            display,
            Direction::Horizontal(Scrollbar::default()),
        );
        container(column![header, display])
            .padding(10)
            .style(|_theme| Style {
                text_color: Some(settings.text_color),
                background: Some(Background::Color(settings.background_color)),
                border: Border {
                    color: Color::BLACK,
                    width: 2.0,
                    radius: Radius::new(5),
                },
                shadow: Shadow::default(),
            })
    }
}

#[derive(Debug, Default, Clone)]
pub struct DisplaySettings {
    pub statebox_default: StateboxSettings,
    pub unique_stateboxes: HashMap<String, StateboxSettings>,
}

#[derive(Debug, Clone)]
pub enum ColorPickerType {
    DefaultStateboxColor,
    DefaultTextColor,
}

#[derive(Debug, Clone)]
pub struct StateboxSettings {
    pub background_color: Color,
    pub text_color: Color,

    pub debug: bool,
    pub animated: bool,
    pub show_resized: bool,

    pub resize: StateboxResizing,
    pub filter_type: Option<CustomFilterType>,
}

impl Default for StateboxSettings {
    fn default() -> Self {
        Self {
            background_color: Color::BLACK,
            text_color: Color::WHITE,
            debug: false,
            animated: true,
            show_resized: true,
            resize: StateboxResizing::default(),
            filter_type: Some(CustomFilterType::Nearest),
        }
    }
}

const DEFAULT_HEIGHT_RESIZE: u32 = 64;
const DEFAULT_WIDTH_RESIZE: u32 = 64;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StateboxResizing {
    Original,
    Resized { height: u32, width: u32 },
}

impl Default for StateboxResizing {
    fn default() -> Self {
        StateboxResizing::Resized {
            height: DEFAULT_HEIGHT_RESIZE,
            width: DEFAULT_WIDTH_RESIZE,
        }
    }
}
