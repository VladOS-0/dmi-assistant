use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsStr,
    path::PathBuf,
    time::Instant,
};

use arboard::Clipboard;
use iced::{
    Element, Font, Length, Task,
    advanced::{
        self,
        widget::{Operation, operation},
    },
    alignment::{Horizontal, Vertical},
    color,
    font::Weight,
    keyboard::{Key, Modifiers},
    widget::{
        self, Column, Container, Space, TextInput, button, column, container,
        rich_text, row, scrollable, span, text, text_input,
    },
};
use iced_aw::{NumberInput, TabLabel};
use iced_toasts::ToastLevel;
use log::{debug, error};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::{
    DMIAssistant, Message, ViewerMessage,
    dmi_utils::load_dmi,
    icon,
    screens::{Screen, Screens},
    utils::{bold_text, popup},
    wrap,
};

const DEFAULT_PAGE_SIZE: usize = 20;
const DEFAULT_DELIMETER: &str = ", ";
const DEFAULT_RECURSION_DEPTH: usize = 20;

const MAIN_EXPLORER_SCROLLABLE_ID: &str = "Main Explorer Scrollabe";
const MAIN_EXPLORER_CONTAINER_ID: &str = "Main Explorer Container";

#[derive(Debug, Clone)]
pub enum ExplorerMessage {
    ChangeInputDMIPath(String),
    OpenedFileExplorer(bool),

    LoadDMI(PathBuf),
    DMILoaded((PathBuf, Result<Vec<String>, String>)),

    CopyDMI(PathBuf),
    CopyText(String),
    OpenInViewer(PathBuf),

    RemoveDMI(PathBuf),
    ClearAll,

    ChangeFilteredText(String),
    ToggleFilter(bool),

    JumpToPage(usize, usize),

    ToggleSettingsVisibility(bool),
    SaveSettings,
    LoadSettings,
    ResetSettings,
    ChangePageSize(usize),
    ChangeDelimeter(String),
    ChangeRecursionDepth(usize),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorerSettings {
    pub page_size: usize,
    pub delimeter: String,
    pub recursion_depth: usize,
}

impl Default for ExplorerSettings {
    fn default() -> Self {
        Self {
            page_size: DEFAULT_PAGE_SIZE,
            delimeter: DEFAULT_DELIMETER.to_string(),
            recursion_depth: DEFAULT_RECURSION_DEPTH,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ExplorerScreen {
    pub hovered_file: bool,
    pub path_in_input: String,
    pub loading_dmis: BTreeSet<PathBuf>,
    pub parsed_dmis: BTreeMap<PathBuf, Vec<String>>,
    pub filtered_text: String,
    pub filter_opened: bool,
    pub current_page: usize,
    pub settings: ExplorerSettings,
    pub settings_visible: bool,
}

impl ExplorerScreen {
    fn filter_view<'a>(&self) -> Container<'a, Message> {
        if self.filter_opened {
            container(
                text_input("Enter text to find...", &self.filtered_text)
                    .on_input(|input| {
                        wrap![ExplorerMessage::ChangeFilteredText(input)]
                    })
                    .on_paste(|input| {
                        wrap![ExplorerMessage::ChangeFilteredText(input)]
                    })
                    .padding(10),
            )
            .style(container::bordered_box)
            .padding(10)
        } else {
            container("")
        }
    }
}

impl Screen for ExplorerScreen {
    fn label(&self) -> TabLabel {
        TabLabel::IconText('\u{1F50D}', " Explorer".to_string())
    }

    fn update(app: &mut DMIAssistant, message: Message) -> Task<Message> {
        let screen = &mut app.explorer_screen;
        match message {
            Message::Window(_id, event) => match event {
                iced::window::Event::FileHovered(_) => {
                    screen.hovered_file = true;
                    Task::none()
                }
                iced::window::Event::FilesHoveredLeft => {
                    screen.hovered_file = false;
                    Task::none()
                }
                iced::window::Event::FileDropped(path) => {
                    let base_path: PathBuf = path
                        .to_str()
                        .unwrap_or("FAILED TO RESOLVE FILE")
                        .to_owned()
                        .into();

                    let dummy = PathBuf::new();

                    if base_path.is_dir() {
                        screen.hovered_file = false;
                        return Task::batch(
                            WalkDir::new(&base_path)
                                .max_depth(screen.settings.recursion_depth)
                                .into_iter()
                                .filter_map(|entry| {
                                    entry
                                        .and_then(|entry| {
                                            {
                                                entry.metadata().map(
                                                    |metadata| {
                                                        if metadata.is_file() {
                                                            entry
                                                                .path()
                                                                .to_path_buf()
                                                        } else {
                                                            dummy.clone()
                                                        }
                                                    },
                                                )
                                            }
                                        })
                                        .ok()
                                })
                                .filter(|path| {
                                    path.extension() == Some(OsStr::new("dmi"))
                                })
                                .map(|path| {
                                    Task::done(wrap![ExplorerMessage::LoadDMI(
                                        path.to_path_buf()
                                    )])
                                }),
                        );
                    }
                    if screen.loading_dmis.contains(&path)
                        || screen.parsed_dmis.contains_key(&path)
                    {
                        return Task::none();
                    }
                    screen.hovered_file = false;
                    Task::done(wrap![ExplorerMessage::LoadDMI(path)])
                }

                _ => Task::none(),
            },

            Message::Keyboard(key, modifiers) => {
                if modifiers.contains(Modifiers::CTRL)
                    && (key == Key::Character("f".into())
                        || key == Key::Character("F".into())
                        || key == Key::Character("а".into())
                        || key == Key::Character("А".into()))
                {
                    return Task::done(wrap![ExplorerMessage::ToggleFilter(
                        !screen.filter_opened
                    )]);
                }

                Task::none()
            }

            Message::ExplorerMessage(explorer_message) => {
                match explorer_message {
                    ExplorerMessage::LoadDMI(path) => {
                        screen.loading_dmis.insert(path.clone());
                        Task::future(async move {
                            let load_start = Instant::now();
                            let opened_dmi = load_dmi(path.clone());
                            if opened_dmi.is_err() {
                                return wrap![ExplorerMessage::DMILoaded((
                                    path,
                                    Err(format!("{}", opened_dmi.unwrap_err()),)
                                ))];
                            }
                            let opened_dmi = opened_dmi.unwrap();

                            let existing_states: Vec<String> = opened_dmi
                                .states
                                .iter()
                                .map(|state| state.name.clone())
                                .collect();

                            debug!(
                                "DMI {} parsed in {}ms",
                                path.to_string_lossy(),
                                load_start.elapsed().as_millis()
                            );
                            wrap![ExplorerMessage::DMILoaded((
                                path,
                                Ok(existing_states)
                            ))]
                        })
                    }
                    ExplorerMessage::DMILoaded((path, loaded)) => {
                        if let Err(err) = loaded {
                            error!(
                                "Failed to load DMI {}; Reason: {}",
                                path.to_string_lossy(),
                                err
                            );
                            screen.loading_dmis.remove(&path);
                            return Task::done(popup(
                                format!(
                                    "Failed to load DMI: {}; Reason: {}",
                                    path.to_string_lossy(),
                                    err
                                ),
                                Some("Load failed"),
                                ToastLevel::Warning,
                            ));
                        }
                        if screen.loading_dmis.remove(&path) {
                            screen
                                .parsed_dmis
                                .insert(path.clone(), loaded.unwrap());
                        }

                        Task::done(popup(
                            format!("Loaded {}", path.to_string_lossy(),),
                            Some("Loaded DMI"),
                            ToastLevel::Success,
                        ))
                    }
                    ExplorerMessage::CopyDMI(path) => {
                        let states = screen
                            .parsed_dmis
                            .get(&path)
                            .unwrap_or(&Vec::new())
                            .join(&screen.settings.delimeter);
                        let _ = Clipboard::new().unwrap().set_text(states);
                        Task::done(popup(
                            "All states were copied",
                            Some("Copied"),
                            ToastLevel::Success,
                        ))
                    }
                    ExplorerMessage::CopyText(text) => {
                        let _ = Clipboard::new().unwrap().set_text(text);
                        Task::done(popup(
                            "Text was copied",
                            Some("Copied"),
                            ToastLevel::Success,
                        ))
                    }
                    ExplorerMessage::RemoveDMI(path) => {
                        screen.parsed_dmis.remove(&path);
                        Task::done(popup(
                            format!(
                                "{} was removed from explorer",
                                path.to_string_lossy()
                            ),
                            Some("Removed"),
                            ToastLevel::Success,
                        ))
                    }
                    ExplorerMessage::ClearAll => {
                        screen.parsed_dmis.clear();
                        screen.loading_dmis.clear();
                        Task::done(popup(
                            "Explorer was cleared",
                            Some("Removed All"),
                            ToastLevel::Success,
                        ))
                    }
                    ExplorerMessage::ChangeInputDMIPath(new_string) => {
                        screen.path_in_input = new_string;
                        Task::none()
                    }
                    ExplorerMessage::OpenedFileExplorer(browse_dirs) => {
                        let files = if browse_dirs {
                            FileDialog::new()
                                .set_title("Open folders with DMIs")
                                .set_directory("/")
                                .pick_folders()
                        } else {
                            FileDialog::new()
                                .set_title("Open DMIs")
                                .set_directory("/")
                                .add_filter("dmi", &["dmi"])
                                .pick_files()
                        };

                        if let Some(paths) = files {
                            let dummy = PathBuf::new();

                            Task::batch(paths.into_iter().map(|path| {
                                if path.is_dir() {
                                    Task::batch(
                                        WalkDir::new(path)
                                            .max_depth(
                                                screen.settings.recursion_depth,
                                            )
                                            .into_iter()
                                            .filter_map(|entry| {
                                                entry
                                                .and_then(|entry| {
                                                    {
                                                        entry.metadata().map(
                                                            |metadata| {
                                                                if metadata
                                                                    .is_file()
                                                                {
                                                                    entry
                                                                .path()
                                                                .to_path_buf()
                                                                } else {
                                                                    dummy
                                                                        .clone()
                                                                }
                                                            },
                                                        )
                                                    }
                                                })
                                                .ok()
                                            })
                                            .filter(|path| {
                                                path.extension()
                                                    == Some(OsStr::new("dmi"))
                                            })
                                            .map(|path| {
                                                Task::done(wrap![
                                                    ExplorerMessage::LoadDMI(
                                                        path.to_path_buf()
                                                    )
                                                ])
                                            }),
                                    )
                                } else {
                                    Task::done(wrap![ExplorerMessage::LoadDMI(
                                        path
                                    )])
                                }
                            }))
                        } else {
                            Task::none()
                        }
                    }
                    ExplorerMessage::ChangeFilteredText(new_text) => {
                        screen.filtered_text = new_text;
                        let scroll = Box::new(operation::scope(
                            advanced::widget::Id::new(
                                MAIN_EXPLORER_CONTAINER_ID,
                            ),
                            operation::scrollable::snap_to::<Message>(
                                advanced::widget::Id::new(
                                    MAIN_EXPLORER_SCROLLABLE_ID,
                                ),
                                scrollable::RelativeOffset { x: 0.0, y: 0.0 },
                            ),
                        ));
                        scroll.finish();
                        Task::done(wrap![ExplorerMessage::JumpToPage(0, 0)])
                    }
                    ExplorerMessage::ToggleFilter(status) => {
                        screen.filter_opened = status;
                        let scroll = Box::new(
                            operation::scrollable::snap_to::<Message>(
                                advanced::widget::Id::new(
                                    MAIN_EXPLORER_SCROLLABLE_ID,
                                ),
                                scrollable::RelativeOffset { x: 0.0, y: 0.0 },
                            ),
                        );
                        scroll.finish();
                        Task::none()
                    }
                    ExplorerMessage::JumpToPage(page, displayed_dmis_count) => {
                        if page
                            <= displayed_dmis_count / screen.settings.page_size
                        {
                            screen.current_page = page;
                        }

                        Task::none()
                    }
                    ExplorerMessage::ToggleSettingsVisibility(visible) => {
                        screen.settings_visible = visible;
                        Task::none()
                    }
                    ExplorerMessage::SaveSettings => {
                        app.config.explorer_settings = screen.settings.clone();
                        app.config.save();
                        Task::done(popup(
                            "Saved settings to Config.toml",
                            Some("Saved"),
                            ToastLevel::Success,
                        ))
                    }
                    ExplorerMessage::LoadSettings => {
                        screen.settings = app.config.explorer_settings.clone();
                        Task::done(popup(
                            "Loaded settings from the in-memory config",
                            Some("Loaded"),
                            ToastLevel::Success,
                        ))
                    }
                    ExplorerMessage::ResetSettings => {
                        screen.settings = ExplorerSettings::default();
                        Task::done(popup(
                            "Settings were reset to default",
                            Some("Reset"),
                            ToastLevel::Success,
                        ))
                    }
                    ExplorerMessage::ChangePageSize(page_size) => {
                        screen.settings.page_size = page_size;
                        Task::done(wrap![ExplorerMessage::JumpToPage(0, 0)])
                    }
                    ExplorerMessage::ChangeDelimeter(delimeter) => {
                        screen.settings.delimeter = delimeter;
                        Task::none()
                    }
                    ExplorerMessage::OpenInViewer(path_buf) => Task::batch([
                        Task::done(Message::ChangeScreen(Screens::Viewer)),
                        Task::done(wrap![ViewerMessage::ChangeDMIPath(
                            path_buf.to_string_lossy().into()
                        )])
                        .chain(Task::done(wrap![ViewerMessage::LoadDMI])),
                    ]),
                    ExplorerMessage::ChangeRecursionDepth(depth) => {
                        screen.settings.recursion_depth = depth;
                        Task::none()
                    }
                }
            }
            _ => Task::none(),
        }
    }

    fn view<'a>(app: &'a DMIAssistant) -> Element<'a, Message> {
        let screen = &app.explorer_screen;

        /*
         *
         * PLACEHOLDERS
         *
         */
        if screen.hovered_file {
            return container("You can release the mouse now, y\'know")
                .style(container::bordered_box)
                .padding(50)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into();
        }

        let settings_button = button(icon::settings()).on_press(wrap![
            ExplorerMessage::ToggleSettingsVisibility(!screen.settings_visible)
        ]);

        let input_path: TextInput<Message> =
            text_input("Input DMI path", &screen.path_in_input)
                .on_input(|input| {
                    wrap![ExplorerMessage::ChangeInputDMIPath(input)]
                })
                .on_paste(|input| {
                    wrap![ExplorerMessage::ChangeInputDMIPath(input)]
                })
                .on_submit(wrap![ExplorerMessage::LoadDMI(
                    screen.path_in_input.clone().into()
                )])
                .padding(10);

        let button_search = button(row![icon::search(), text(" Filter")])
            .on_press(wrap![ExplorerMessage::ToggleFilter(
                !screen.filter_opened
            )]);

        let button_load = button(row![icon::open(), text(" Open File")])
            .on_press(wrap![ExplorerMessage::LoadDMI(
                screen.path_in_input.clone().into()
            )]);

        let button_file_explorer =
            button(row![icon::iconfile(), text(" Browse Files")])
                .on_press(wrap![ExplorerMessage::OpenedFileExplorer(false)]);

        let button_folder_explorer =
            button(row![icon::folder(), text(" Browse Folders")])
                .on_press(wrap![ExplorerMessage::OpenedFileExplorer(true)]);

        let clear_all = button(row![icon::trash(), text(" Clear All")])
            .on_press(wrap![ExplorerMessage::ClearAll])
            .style(button::danger);

        let input_controls = row![
            settings_button,
            input_path,
            button_load,
            button_file_explorer,
            button_folder_explorer
        ]
        .align_y(Vertical::Center)
        .spacing(5);

        let mut settings_bar: Column<Message> = Column::new();
        if screen.settings_visible {
            let page_size_picker = row![
                bold_text("Page Size: "),
                NumberInput::new(
                    screen.settings.page_size,
                    10..=200,
                    move |new_page_size| {
                        wrap![ExplorerMessage::ChangePageSize(new_page_size)]
                    },
                )
                .step(10)
            ]
            .align_y(Vertical::Center)
            .spacing(5);

            let delimeter_picker = row![
                bold_text("Delimeter For Copy All: "),
                container(
                    text_input(
                        "Enter the delimeter...",
                        &screen.settings.delimeter
                    )
                    .on_input(|input| {
                        wrap![ExplorerMessage::ChangeDelimeter(input)]
                    })
                    .on_paste(|input| {
                        wrap![ExplorerMessage::ChangeDelimeter(input)]
                    })
                    .width(60)
                    .padding(5),
                ),
            ]
            .align_y(Vertical::Center)
            .spacing(5);

            let recusion_depth_picker = row![
                bold_text("Recursion Depth: "),
                NumberInput::new(
                    screen.settings.recursion_depth,
                    1..=100,
                    move |new_depth| {
                        wrap![ExplorerMessage::ChangeRecursionDepth(new_depth)]
                    },
                )
                .step(1)
            ]
            .align_y(Vertical::Center)
            .spacing(5);

            let save_settings = button(row![icon::save(), "  Save Settings"])
                .on_press(wrap![ExplorerMessage::SaveSettings])
                .style(button::success);
            let load_settings =
                button(row![icon::folder(), "  Reset Settings to Config"])
                    .on_press(wrap![ExplorerMessage::LoadSettings]);
            let reset_settings =
                button(row![icon::trash(), "  Reset Settings to Default"])
                    .on_press(wrap![ExplorerMessage::ResetSettings])
                    .style(button::danger);

            settings_bar = column![
                page_size_picker,
                delimeter_picker,
                recusion_depth_picker,
                row![save_settings, load_settings, reset_settings].spacing(5)
            ]
            .spacing(10);
        }

        let output_controls =
            row![button_search, clear_all].padding(5).spacing(5);

        if !screen.loading_dmis.is_empty() {
            let mut tooltip =
                format!("Loading ({})...\n\n", screen.loading_dmis.len());
            for dmi in &screen.loading_dmis {
                tooltip += &dmi.to_string_lossy();
                tooltip += "\n";
            }
            let tooltip = text(tooltip);
            return container(
                column![
                    input_controls,
                    settings_bar,
                    container(tooltip)
                        .style(container::bordered_box)
                        .padding(50)
                        .center_x(Length::Fill)
                        .center_y(Length::Fill)
                ]
                .spacing(10),
            )
            .padding(20)
            .into();
        }

        if screen.parsed_dmis.is_empty() {
            return container(
                column![
                    input_controls,
                    settings_bar,
                    container(bold_text(
                        "... or drop your icon files or folders there!"
                    ))
                    .style(container::bordered_box)
                    .padding(50)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                ]
                .spacing(10),
            )
            .padding(20)
            .into();
        }

        let mut parsed_dmis_column: Column<Message> = Column::new();
        let mut displayed_dmis_count: usize = 0;

        for (path, dmi) in &screen.parsed_dmis {
            let mut dmi_states_column: Column<Message> = Column::new();

            let filter_selected_dmi =
                path.to_string_lossy().contains(&screen.filtered_text);
            let mut filter_selected_state = false;

            for state in dmi {
                let mut filter_selected_this_state = false;
                if state.contains(&screen.filtered_text) {
                    filter_selected_state = true;
                    filter_selected_this_state = true;
                }
                if filter_selected_dmi || filter_selected_this_state {
                    let selected_mark: text::Rich<Message> =
                        if screen.filtered_text.is_empty() {
                            rich_text([span("")])
                        } else if filter_selected_this_state {
                            rich_text([span("+  ")
                                .color(color!(0x89fc41))
                                .size(20)])
                        } else {
                            rich_text([span("-  ")
                                .color(color!(0xfc4144))
                                .size(20)])
                        };
                    dmi_states_column = dmi_states_column.push(row![
                        row![selected_mark, text!("{}  ", state)],
                        button(icon::save())
                            .on_press(wrap![ExplorerMessage::CopyText(
                                state.clone()
                            )])
                            .style(button::secondary)
                    ])
                }
            }
            if filter_selected_state || filter_selected_dmi {
                displayed_dmis_count += 1;

                if displayed_dmis_count / screen.settings.page_size
                    != screen.current_page
                {
                    continue;
                }

                let selected_mark: text::Rich<Message> = if screen
                    .filtered_text
                    .is_empty()
                {
                    rich_text([span("")])
                } else if filter_selected_dmi {
                    rich_text([span("+  ").color(color!(0x89fc41)).size(20)])
                } else {
                    rich_text([span("-  ").color(color!(0xfc4144)).size(20)])
                };
                parsed_dmis_column =
                    parsed_dmis_column.push(container(column![
                        row![selected_mark, bold_text(path.to_string_lossy())],
                        row![
                            button(row![icon::search(), text(" View")])
                                .on_press(wrap![ExplorerMessage::OpenInViewer(
                                    path.clone()
                                )])
                                .style(button::success),
                            button(row![icon::save(), text(" Copy All")])
                                .on_press(wrap![ExplorerMessage::CopyDMI(
                                    path.clone()
                                )]),
                            button(row![icon::save(), text(" Copy Path")])
                                .on_press(wrap![ExplorerMessage::CopyText(
                                    path.to_string_lossy().to_string()
                                )])
                                .style(button::secondary),
                            button(row![icon::trash(), text(" Clear")])
                                .on_press(wrap![ExplorerMessage::RemoveDMI(
                                    path.clone()
                                )])
                                .style(button::danger),
                        ]
                        .spacing(4),
                        dmi_states_column,
                        Space::with_height(20)
                    ]));
            }
        }

        let upper_page_controls =
            if displayed_dmis_count > screen.settings.page_size {
                let zeroth_page_button =
                    button("<<").on_press(wrap![ExplorerMessage::JumpToPage(
                        0,
                        displayed_dmis_count
                    )]);
                let previous_page_button =
                    button("<").on_press(wrap![ExplorerMessage::JumpToPage(
                        if screen.current_page != 0 {
                            screen.current_page - 1
                        } else {
                            0
                        },
                        displayed_dmis_count
                    )]);
                let next_page_button =
                    button(">").on_press(wrap![ExplorerMessage::JumpToPage(
                        screen.current_page + 1,
                        displayed_dmis_count
                    )]);
                let last_page_button =
                    button(">>").on_press(wrap![ExplorerMessage::JumpToPage(
                        displayed_dmis_count / screen.settings.page_size,
                        displayed_dmis_count
                    )]);
                let page_text = text!(
                    "Viewing {} page from {} | DMIs {} - {} of {}",
                    screen.current_page + 1,
                    displayed_dmis_count / screen.settings.page_size + 1,
                    screen.settings.page_size * screen.current_page + 1,
                    (screen.settings.page_size * screen.current_page
                        + screen.settings.page_size
                        + 1)
                    .min(displayed_dmis_count),
                    displayed_dmis_count
                )
                .font(Font {
                    weight: Weight::Bold,
                    ..Default::default()
                });
                container(
                    row![
                        zeroth_page_button,
                        previous_page_button,
                        page_text,
                        next_page_button,
                        last_page_button
                    ]
                    .spacing(10)
                    .padding(5)
                    .align_y(Vertical::Center),
                )
                .align_x(Horizontal::Center)
            } else {
                let dmi_count_text =
                    text!("Viewing {} DMIs", displayed_dmis_count).font(Font {
                        weight: Weight::Bold,
                        ..Default::default()
                    });
                container(dmi_count_text)
                    .padding(5)
                    .align_y(Vertical::Center)
                    .align_x(Horizontal::Center)
            };

        let lower_page_controls =
            if displayed_dmis_count > screen.settings.page_size {
                let zeroth_page_button =
                    button("<<").on_press(wrap![ExplorerMessage::JumpToPage(
                        0,
                        displayed_dmis_count
                    )]);
                let previous_page_button =
                    button("<").on_press(wrap![ExplorerMessage::JumpToPage(
                        if screen.current_page != 0 {
                            screen.current_page - 1
                        } else {
                            0
                        },
                        displayed_dmis_count
                    )]);
                let next_page_button =
                    button(">").on_press(wrap![ExplorerMessage::JumpToPage(
                        screen.current_page + 1,
                        displayed_dmis_count
                    )]);
                let last_page_button =
                    button(">>").on_press(wrap![ExplorerMessage::JumpToPage(
                        displayed_dmis_count / screen.settings.page_size,
                        displayed_dmis_count
                    )]);
                let page_text = text!(
                    "Viewing {} page from {} | DMIs {} - {} of {}",
                    screen.current_page + 1,
                    displayed_dmis_count / screen.settings.page_size + 1,
                    screen.settings.page_size * screen.current_page + 1,
                    (screen.settings.page_size * screen.current_page
                        + screen.settings.page_size
                        + 1)
                    .min(displayed_dmis_count),
                    displayed_dmis_count
                )
                .font(Font {
                    weight: Weight::Bold,
                    ..Default::default()
                });
                container(
                    row![
                        zeroth_page_button,
                        previous_page_button,
                        page_text,
                        next_page_button,
                        last_page_button
                    ]
                    .spacing(10)
                    .padding(5)
                    .align_y(Vertical::Center),
                )
                .align_x(Horizontal::Center)
            } else {
                let dmi_count_text =
                    text!("Viewing {} DMIs", displayed_dmis_count).font(Font {
                        weight: Weight::Bold,
                        ..Default::default()
                    });
                container(dmi_count_text)
                    .padding(5)
                    .align_y(Vertical::Center)
                    .align_x(Horizontal::Center)
            };

        container(
            scrollable(
                column![
                    input_controls,
                    output_controls,
                    screen.filter_view(),
                    settings_bar,
                    upper_page_controls,
                    parsed_dmis_column,
                    lower_page_controls,
                ]
                .spacing(10),
            )
            .id(widget::scrollable::Id::new(MAIN_EXPLORER_SCROLLABLE_ID)),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .padding(20)
        .id(widget::container::Id::new(MAIN_EXPLORER_CONTAINER_ID))
        .into()
    }
}
