use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    time::Instant,
};

use arboard::Clipboard;
use iced::{
    widget::{button, column, container, row, scrollable, text, Column, Space},
    Element, Length, Task,
};
use iced_aw::TabLabel;
use walkdir::WalkDir;

use crate::{
    dmi_utils::load_dmi, icon, screens::Screen, utils::bold_text, wrap,
    DMIAssistant, Message,
};

#[derive(Debug, Clone)]
pub enum ExtractorMessage {
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
    loading_dmis: BTreeSet<PathBuf>,
    parsed_dmis: BTreeMap<PathBuf, Vec<String>>,
}

impl Screen for ExtractorScreen {
    fn label(&self) -> TabLabel {
        TabLabel::IconText('\u{1F4BE}', " Extractor".to_string())
    }

    fn update(app: &mut DMIAssistant, message: Message) -> Task<Message> {
        let screen = &mut app.extractor_screen;
        if let Message::ExtractorMessage(screen_message) = message {
            match screen_message {
                ExtractorMessage::LoadDMI(path) => Task::future(async move {
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
                }),
                ExtractorMessage::DMILoaded((path, loaded)) => {
                    if let Err(err) = loaded {
                        eprintln!("{err}");
                        screen.loading_dmis.remove(&path);
                        return Task::none();
                    }
                    screen.loading_dmis.remove(&path);

                    screen.parsed_dmis.insert(path, loaded.unwrap());
                    Task::none()
                }
                ExtractorMessage::CopyDMI(path) => {
                    let states = screen
                        .parsed_dmis
                        .get(&path)
                        .unwrap_or(&Vec::new())
                        .join(", ");
                    let _ = Clipboard::new().unwrap().set_text(states);
                    Task::none()
                }
                ExtractorMessage::CopyText(text) => {
                    let _ = Clipboard::new().unwrap().set_text(text);
                    Task::none()
                }
                ExtractorMessage::RemoveDMI(path) => {
                    screen.parsed_dmis.remove(&path);
                    Task::none()
                }
                ExtractorMessage::ClearAll => {
                    screen.parsed_dmis.clear();
                    screen.loading_dmis.clear();
                    Task::none()
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
                    screen.loading_dmis.insert(path.clone());
                    screen.hovered_file = false;
                    Task::done(wrap![ExtractorMessage::LoadDMI(path)])
                }

                _ => Task::none(),
            }
        } else {
            Task::none()
        }
    }

    fn view(app: &DMIAssistant) -> Element<'_, Message> {
        let screen = &app.extractor_screen;
        /*
         *
         * PLACEHOLDERS
         *
         */
        if screen.hovered_file {
            return container("You can release the mouse now, y\'now")
                .style(container::bordered_box)
                .padding(50)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into();
        }

        if !screen.loading_dmis.is_empty() {
            let mut tooltip =
                format!("Loading ({})...\n\n", screen.loading_dmis.len());
            for dmi in &screen.loading_dmis {
                tooltip += &dmi.to_string_lossy();
                tooltip += "\n";
            }
            let tooltip = column!(
                text(tooltip),
                button(row![icon::trash(), text(" Abort")])
                    .on_press(wrap![ExtractorMessage::ClearAll])
                    .style(button::danger)
            );
            return container(tooltip)
                .style(container::bordered_box)
                .padding(50)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into();
        }

        if screen.parsed_dmis.is_empty() {
            return container(column![bold_text("Drop your files there!")])
                .style(container::bordered_box)
                .padding(50)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
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
                        .style(button::secondary),
                ]
                .spacing(4),
                dmi_states_column,
                Space::with_height(40)
            ]));
        }
        container(scrollable(column![
            row![
                bold_text("Parsed:    "),
                button(row![icon::trash(), text(" Clear All")])
                    .on_press(wrap![ExtractorMessage::ClearAll])
                    .style(button::danger)
            ],
            parsed_dmis_column
        ]))
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }
}
