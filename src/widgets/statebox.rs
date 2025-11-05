/*
use iced::{
    advanced::{layout, mouse, renderer, widget, Layout, Widget}, alignment::Horizontal, widget::{container, text}, Element, Length, Renderer, Size
};
use iced_aw::{Grid, GridRow};

use crate::{dmi_model::ParsedState, screens::debugger::StateboxSettings};

/// Statebox - widget, that displays one icon state according to StateboxSettings.
#[derive(Debug, Clone)]
pub struct Statebox<'a> {
    pub state: &'a ParsedState,
    pub settings: StateboxSettings,

    pub statebox_settings: StateboxSettings,
}

impl<'a, Message, Theme, Renderer> From<Statebox<'a>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn from(widget: Statebox<'a>) -> Self {
        Self::new(widget)
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Statebox<'a>
where
    Renderer: renderer::Renderer, {
    fn size(&self) -> Size<Length> {
        todo!()
    }

    fn layout(
        &self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        todo!()
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        todo!()
    }
}*/
