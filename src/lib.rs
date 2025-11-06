use iced::widget::container;
use iced::window::{Event, Id};
use iced::{color, Background, Element, Task};
use iced::{Length, Theme};
use iced_aw::time_picker::Status;
use iced_aw::{tab_bar, Tabs};
use iced_toasts::{toast_container, Toast, ToastContainer, ToastId};

pub mod dmi_model;
pub mod dmi_utils;
pub mod screens;
pub mod utils;
pub mod widgets;

use crate::screens::extractor::{ExtractorMessage, ExtractorScreen};
use crate::screens::Screen;
use screens::viewer::{ViewerMessage, ViewerScreen};
use screens::Screens;
use utils::cleanup;

pub mod icon;

pub const DEFAULT_THEME: Theme = Theme::Nightfly;

#[derive(Debug, Clone)]
pub enum Message {
    Window(Id, Event),

    PushToast(Box<Toast<Message>>),
    DismissToast(ToastId),

    ChangeScreen(Screens),

    ViewerMessage(ViewerMessage),
    ExtractorMessage(ExtractorMessage),
}

#[derive(Debug)]
pub struct DMIAssistant<'a> {
    pub current_screen: Screens,

    pub viewer_screen: ViewerScreen,
    pub extractor_screen: ExtractorScreen,

    pub theme: Theme,
    pub toasts: ToastContainer<'a, Message>,
}

impl DMIAssistant<'_> {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        if let Message::Window(_id, event) = &message {
            match event {
                Event::Closed | Event::CloseRequested => {
                    cleanup();
                    iced::exit()
                }
                _ => match self.current_screen {
                    Screens::Extractor => {
                        ExtractorScreen::update(self, message)
                    }
                    Screens::Viewer => ViewerScreen::update(self, message),
                },
            }
        } else {
            match message {
                Message::PushToast(boxed_toast) => {
                    self.toasts.push(boxed_toast.as_ref().clone());
                    Task::none()
                }
                Message::DismissToast(id) => {
                    self.toasts.dismiss(id);
                    Task::none()
                }
                Message::ChangeScreen(screen) => {
                    self.current_screen = screen;
                    Task::none()
                }
                Message::ViewerMessage(msg) => {
                    ViewerScreen::update(self, Message::ViewerMessage(msg))
                }
                Message::ExtractorMessage(msg) => ExtractorScreen::update(
                    self,
                    Message::ExtractorMessage(msg),
                ),
                _ => Task::none(),
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        self.toasts
            .view(
                container(
                    Tabs::new(Message::ChangeScreen)
                        .tab_icon_position(iced_aw::tabs::Position::Left)
                        .push(
                            Screens::Extractor,
                            self.extractor_screen.label(),
                            ExtractorScreen::view(self),
                        )
                        .push(
                            Screens::Viewer,
                            self.viewer_screen.label(),
                            ViewerScreen::view(self),
                        )
                        .set_active_tab(&self.current_screen)
                        .tab_label_spacing(20)
                        .tab_bar_height(Length::Shrink)
                        .tab_label_padding(10)
                        .tab_bar_style(|_, status| match status {
                            Status::Active => tab_bar::Style {
                                tab_label_background: Background::Color(
                                    color!(0x3447c7),
                                ),
                                text_color: color!(0xffffff),
                                icon_color: color!(0xffffff),
                                ..Default::default()
                            },
                            Status::Hovered => tab_bar::Style {
                                tab_label_background: Background::Color(
                                    color!(0x293cba),
                                ),
                                text_color: color!(0xffffff),
                                icon_color: color!(0xffffff),
                                ..Default::default()
                            },
                            Status::Pressed => tab_bar::Style {
                                tab_label_background: Background::Color(
                                    color!(0x132285),
                                ),
                                text_color: color!(0xffffff),
                                icon_color: color!(0xffffff),
                                ..Default::default()
                            },
                            Status::Disabled => tab_bar::Style {
                                tab_label_background: Background::Color(
                                    color!(0x132285),
                                ),
                                text_color: color!(0xffffff),
                                icon_color: color!(0xffffff),
                                ..Default::default()
                            },
                            _ => tab_bar::Style {
                                tab_label_background: Background::Color(
                                    color!(0x132285),
                                ),
                                text_color: color!(0xffffff),
                                icon_color: color!(0xffffff),
                                ..Default::default()
                            },
                        }),
                )
                .padding(10),
            )
            .into()
    }
}

impl Default for DMIAssistant<'_> {
    fn default() -> Self {
        Self {
            current_screen: Default::default(),
            viewer_screen: Default::default(),
            extractor_screen: Default::default(),
            theme: Default::default(),
            toasts: toast_container(Message::DismissToast),
        }
    }
}
