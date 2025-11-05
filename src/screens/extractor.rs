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
                    text!("{}", state),
                    button(row![icon::save(), text(" Copy")])
                        .on_press(wrap![ExtractorMessage::CopyText(
                            state.clone()
                        )])
                        .style(button::secondary)
                ])
            }
            parsed_dmis_column = parsed_dmis_column.push(container(column![
                row![
                    bold_text(path.to_string_lossy()),
                    button(row![icon::save(), text(" Copy Path")])
                        .on_press(wrap![ExtractorMessage::CopyText(
                            path.to_string_lossy().to_string()
                        )])
                        .style(button::secondary)
                ],
                dmi_states_column,
                button(row![icon::save(), text(" Copy All")])
                    .on_press(wrap![ExtractorMessage::CopyDMI(path.clone())])
                    .style(button::secondary),
                Space::with_height(20)
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
