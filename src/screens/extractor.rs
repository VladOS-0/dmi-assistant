use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsStr,
    path::PathBuf,
    time::Instant,
};

use arboard::Clipboard;
use iced::{
    widget::{
        button, column, container, row, scrollable, text, text_input, Column,
        Space, TextInput,
    },
    Element, Length, Task,
};
use iced_aw::TabLabel;
use iced_toasts::ToastLevel;
use rfd::FileDialog;
use walkdir::WalkDir;

use crate::{
    dmi_utils::load_dmi,
    icon,
    screens::Screen,
    utils::{bold_text, popup},
    wrap, DMIAssistant, Message,
};

#[derive(Debug, Clone)]
pub enum ExtractorMessage {
    ChangeInputDMIPath(String),
    OpenedFileExplorer(bool),
    LoadDMI(PathBuf),
    DMILoaded((PathBuf, Result<Vec<String>, String>)),
    CopyDMI(PathBuf),
    CopyText(String),
    RemoveDMI(PathBuf),
    ClearAll,
}

#[derive(Default, Debug, Clone)]
pub struct ExtractorScreen {
    hovered_file: bool,
    path_in_input: String,
    loading_dmis: BTreeSet<PathBuf>,
    parsed_dmis: BTreeMap<PathBuf, Vec<String>>,
}

impl Screen for ExtractorScreen {
    fn label(&self) -> TabLabel {
        TabLabel::IconText('\u{1F4BE}', " Statename Extractor".to_string())
    }

    fn update(app: &mut DMIAssistant, message: Message) -> Task<Message> {
        let screen = &mut app.extractor_screen;
        if let Message::ExtractorMessage(screen_message) = message {
            match screen_message {
                ExtractorMessage::LoadDMI(path) => {
                    screen.loading_dmis.insert(path.clone());
                    Task::future(async move {
                        let load_start = Instant::now();
                        let opened_dmi = load_dmi(path.clone());
                        if opened_dmi.is_err() {
                            return wrap![ExtractorMessage::DMILoaded((
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

                        println!(
                            "DMI parsed in {}ms",
                            load_start.elapsed().as_millis()
                        );
                        wrap![ExtractorMessage::DMILoaded((
                            path,
                            Ok(existing_states)
                        ))]
                    })
                }
                ExtractorMessage::DMILoaded((path, loaded)) => {
                    if let Err(err) = loaded {
                        eprintln!("{err}");
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
                ExtractorMessage::CopyDMI(path) => {
                    let states = screen
                        .parsed_dmis
                        .get(&path)
                        .unwrap_or(&Vec::new())
                        .join(", ");
                    let _ = Clipboard::new().unwrap().set_text(states);
                    Task::done(popup(
                        "All states were copied",
                        Some("Copied"),
                        ToastLevel::Success,
                    ))
                }
                ExtractorMessage::CopyText(text) => {
                    let _ = Clipboard::new().unwrap().set_text(text);
                    Task::done(popup(
                        "Text was copied",
                        Some("Copied"),
                        ToastLevel::Success,
                    ))
                }
                ExtractorMessage::RemoveDMI(path) => {
                    screen.parsed_dmis.remove(&path);
                    Task::done(popup(
                        format!(
                            "{} was removed from extractor",
                            path.to_string_lossy()
                        ),
                        Some("Removed"),
                        ToastLevel::Success,
                    ))
                }
                ExtractorMessage::ClearAll => {
                    screen.parsed_dmis.clear();
                    screen.loading_dmis.clear();
                    Task::done(popup(
                        "Extractor was cleared",
                        Some("Removed All"),
                        ToastLevel::Success,
                    ))
                }
                ExtractorMessage::ChangeInputDMIPath(new_string) => {
                    screen.path_in_input = new_string;
                    Task::none()
                }
                ExtractorMessage::OpenedFileExplorer(browse_dirs) => {
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
                                        .max_depth(20)
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
                                                ExtractorMessage::LoadDMI(
                                                    path.to_path_buf()
                                                )
                                            ])
                                        }),
                                )
                            } else {
                                Task::done(wrap![ExtractorMessage::LoadDMI(
                                    path
                                )])
                            }
                        }))
                    } else {
                        Task::none()
                    }
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
                    let path: PathBuf = path
                        .to_str()
                        .unwrap_or("FAILED TO RESOLVE FILE")
                        .to_owned()
                        .into();

                    let dummy = PathBuf::new();

                    if path.is_dir() {
                        screen.hovered_file = false;
                        return Task::batch(
                            WalkDir::new(path)
                                .max_depth(20)
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
                                    Task::done(wrap![
                                        ExtractorMessage::LoadDMI(
                                            path.to_path_buf()
                                        )
                                    ])
                                }),
                        );
                    }
                    if screen.loading_dmis.contains(&path)
                        || screen.parsed_dmis.contains_key(&path)
                    {
                        return Task::none();
                    }
                    screen.hovered_file = false;
                    Task::done(wrap![ExtractorMessage::LoadDMI(path)])
                }

                _ => Task::none(),
            }
        } else {
            Task::none()
        }
    }

    fn view<'a>(app: &'a DMIAssistant) -> Element<'a, Message> {
        let screen = &app.extractor_screen;

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

        let input_path: TextInput<Message> =
            text_input("Input DMI path", &screen.path_in_input)
                .on_input(|input| {
                    wrap![ExtractorMessage::ChangeInputDMIPath(input)]
                })
                .on_paste(|input| {
                    wrap![ExtractorMessage::ChangeInputDMIPath(input)]
                })
                .on_submit(wrap![ExtractorMessage::LoadDMI(
                    screen.path_in_input.clone().into()
                )])
                .padding(10);

        let button_load = button(row![icon::open(), text(" Open File")])
            .on_press(wrap![ExtractorMessage::LoadDMI(
                screen.path_in_input.clone().into()
            )]);

        let button_file_explorer =
            button(row![icon::iconfile(), text(" Browse Files")])
                .on_press(wrap![ExtractorMessage::OpenedFileExplorer(false)]);

        let button_folder_explorer =
            button(row![icon::folder(), text(" Browse Folders")])
                .on_press(wrap![ExtractorMessage::OpenedFileExplorer(true)]);

        let clear_all = button(row![icon::trash(), text(" Clear All")])
            .on_press(wrap![ExtractorMessage::ClearAll])
            .style(button::danger);

        let input_controls = row![
            input_path,
            clear_all,
            button_load,
            button_file_explorer,
            button_folder_explorer
        ]
        .spacing(5);

        if !screen.loading_dmis.is_empty() {
            let mut tooltip =
                format!("Loading ({})...\n\n", screen.loading_dmis.len());
            for dmi in &screen.loading_dmis {
                tooltip += &dmi.to_string_lossy();
                tooltip += "\n";
            }
            let tooltip = column!(text(tooltip));
            return container(
                column![
                    input_controls,
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
                    container(bold_text(
                        ".. or drop your icon files or folders there!"
                    ))
                    .style(container::bordered_box)
                    .padding(50)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                ]
                .spacing(10),
            )
            .padding(50)
            .into();
        }

        let mut parsed_dmis_column: Column<Message> = Column::new();

        for (path, dmi) in &screen.parsed_dmis {
            let mut dmi_states_column: Column<Message> = Column::new();
            for state in dmi {
                dmi_states_column = dmi_states_column.push(row![
                    text!("{}  ", state),
                    button(icon::save())
                        .on_press(wrap![ExtractorMessage::CopyText(
                            state.clone()
                        )])
                        .style(button::secondary)
                ])
            }
            parsed_dmis_column = parsed_dmis_column.push(container(column![
                row![
                    bold_text(path.to_string_lossy()),
                    button(row![icon::save(), text(" Copy All")]).on_press(
                        wrap![ExtractorMessage::CopyDMI(path.clone())]
                    ),
                    button(row![icon::save(), text(" Copy Path")])
                        .on_press(wrap![ExtractorMessage::CopyText(
                            path.to_string_lossy().to_string()
                        )])
                        .style(button::secondary),
                    button(row![icon::trash(), text(" Clear")])
                        .on_press(wrap![ExtractorMessage::RemoveDMI(
                            path.clone()
                        )])
                        .style(button::danger),
                ]
                .spacing(4),
                dmi_states_column,
                Space::with_height(40)
            ]));
        }
        container(scrollable(column![
            input_controls,
            Space::with_height(50),
            row![bold_text("Parsed:    "), Space::with_height(20)],
            parsed_dmis_column
        ]))
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .padding(20)
        .into()
    }
}
