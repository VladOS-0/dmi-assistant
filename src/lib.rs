use iced::keyboard::{Key, Modifiers};
use iced::widget::container;
use iced::window::{Event, Id};
use iced::{Background, Element, Task, color};
use iced::{Length, Theme};
use iced_aw::time_picker::Status;
use iced_aw::{Tabs, tab_bar};
use iced_toasts::{Toast, ToastContainer, ToastId, toast_container};

pub mod config;
pub mod dmi_model;
pub mod dmi_utils;
pub mod screens;
pub mod utils;
pub mod widgets;

use crate::config::Config;
use crate::screens::Screen;
use crate::screens::explorer::{ExplorerMessage, ExplorerScreen};
use screens::Screens;
use screens::viewer::{ViewerMessage, ViewerScreen};
use utils::cleanup;

#[rustfmt::skip]
pub mod icon;

pub const DEFAULT_THEME: Theme = Theme::Nightfly;

#[derive(Debug, Clone)]
pub enum Message {
    Window(Id, Event),
    Keyboard(Key, Modifiers),

    PushToast(Box<Toast<Message>>),
    DismissToast(ToastId),

    ChangeScreen(Screens),

    ViewerMessage(ViewerMessage),
    ExplorerMessage(ExplorerMessage),
}

#[derive(Debug)]
pub struct DMIAssistant<'a> {
    pub config: Config,

    pub current_screen: Screens,

    pub viewer_screen: ViewerScreen,
    pub explorer_screen: ExplorerScreen,

    pub theme: Theme,
    pub toasts: ToastContainer<'a, Message>,
}

impl DMIAssistant<'_> {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            current_screen: Default::default(),
            viewer_screen: Default::default(),
            explorer_screen: Default::default(),
            theme: Default::default(),
            toasts: toast_container(Message::DismissToast),
        }
    }
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match &message {
            Message::Window(_id, event) => match event {
                Event::Closed | Event::CloseRequested => {
                    cleanup(&self.config);
                    iced::exit()
                }
                _ => match self.current_screen {
                    Screens::Explorer => ExplorerScreen::update(self, message),
                    Screens::Viewer => ViewerScreen::update(self, message),
                },
            },

            Message::Keyboard(_, _) => match self.current_screen {
                Screens::Explorer => ExplorerScreen::update(self, message),
                Screens::Viewer => ViewerScreen::update(self, message),
            },
            Message::PushToast(boxed_toast) => {
                self.toasts.push(boxed_toast.as_ref().clone());
                Task::none()
            }
            Message::DismissToast(id) => {
                self.toasts.dismiss(*id);
                Task::none()
            }
            Message::ChangeScreen(screen) => {
                self.current_screen = screen.clone();
                Task::none()
            }
            Message::ViewerMessage(msg) => {
                ViewerScreen::update(self, Message::ViewerMessage(msg.clone()))
            }
            Message::ExplorerMessage(msg) => ExplorerScreen::update(
                self,
                Message::ExplorerMessage(msg.clone()),
            ),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        self.toasts.view(
            container(
                Tabs::new(Message::ChangeScreen)
                    .tab_icon_position(iced_aw::tabs::Position::Left)
                    .push(
                        Screens::Explorer,
                        self.explorer_screen.label(),
                        ExplorerScreen::view(self),
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
                            tab_label_background: Background::Color(color!(
                                0x3447c7
                            )),
                            text_color: color!(0xffffff),
                            icon_color: color!(0xffffff),
                            ..Default::default()
                        },
                        Status::Hovered => tab_bar::Style {
                            tab_label_background: Background::Color(color!(
                                0x293cba
                            )),
                            text_color: color!(0xffffff),
                            icon_color: color!(0xffffff),
                            ..Default::default()
                        },
                        Status::Pressed => tab_bar::Style {
                            tab_label_background: Background::Color(color!(
                                0x132285
                            )),
                            text_color: color!(0xffffff),
                            icon_color: color!(0xffffff),
                            ..Default::default()
                        },
                        Status::Disabled => tab_bar::Style {
                            tab_label_background: Background::Color(color!(
                                0x132285
                            )),
                            text_color: color!(0xffffff),
                            icon_color: color!(0xffffff),
                            ..Default::default()
                        },
                        _ => tab_bar::Style {
                            tab_label_background: Background::Color(color!(
                                0x132285
                            )),
                            text_color: color!(0xffffff),
                            icon_color: color!(0xffffff),
                            ..Default::default()
                        },
                    }),
            )
            .padding(10),
        )
    }
}
