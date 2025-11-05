use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    time::Instant,
};

use arboard::Clipboard;
use iced::{
    widget::{button, column, container, row, text, Column},
    Element, Length, Task,
};
use iced_aw::TabLabel;

use crate::{
    dmi_utils::load_dmi, screens::Screen, utils::bold_text, wrap, DMIAssistant,
    Message,
};

#[derive(Debug, Clone)]
pub enum ExtractorMessage {
    LoadDMI(PathBuf),
    DMILoaded((PathBuf, Result<Vec<String>, String>)),
    CopyDMI(PathBuf),
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
                    let file: PathBuf = path
                        .to_str()
                        .unwrap_or("FAILED TO RESOLVE FILE")
                        .to_owned()
                        .into();
                    if screen.loading_dmis.contains(&file)
                        || screen.parsed_dmis.contains_key(&file)
                    {
                        return Task::none();
                    }
                    screen.loading_dmis.insert(file.clone());
                    screen.hovered_file = false;
                    Task::done(wrap![ExtractorMessage::LoadDMI(file)])
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
            return container("Drop 'em!'")
                .style(container::bordered_box)
                .padding(50)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into();
        }

        if !screen.loading_dmis.is_empty() {
            let tooltip = column!(
                text!("Loading ({})...", screen.loading_dmis.len()),
                button("Abort").on_press(wrap![ExtractorMessage::ClearAll])
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
                dmi_states_column = dmi_states_column.push(text!("{}", state))
            }
            parsed_dmis_column = parsed_dmis_column.push(container(column![
                bold_text(path.to_string_lossy()),
                dmi_states_column,
                button("Copy")
                    .on_press(wrap![ExtractorMessage::CopyDMI(path.clone())])
            ]));
        }
        container(column![
            row![
                bold_text("Parsed:    "),
                button("Clear").on_press(wrap![ExtractorMessage::ClearAll])
            ],
            parsed_dmis_column
        ])
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }
}
