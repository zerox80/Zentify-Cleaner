use iced::widget::{button, container, text};
use iced::{Color, Length, Theme};

use super::style;

pub fn feature_card<'a, Message>(
    title: &str,
    description: &str,
    action_text: &str,
    on_press: Message,
    is_disabled: bool,
) -> container::Container<'a, Message>
where
    Message: Clone + 'a,
{
    let title_text = text(title).size(20);
    let description_text = text(description).size(16);
    
    let action_button = if is_disabled {
        button(
            text(action_text)
                .horizontal_alignment(iced::alignment::Horizontal::Center)
        )
        .width(Length::Fill)
        .padding(10)
        .style(iced::theme::Button::Secondary)
    } else {
        button(
            text(action_text)
                .horizontal_alignment(iced::alignment::Horizontal::Center)
        )
        .width(Length::Fill)
        .padding(10)
        .style(iced::theme::Button::Primary)
        .on_press(on_press)
    };

    let content = iced::widget::column![
        title_text,
        description_text,
        iced::widget::horizontal_space(Length::Fill),
        action_button
    ]
    .spacing(10)
    .padding(20)
    .align_items(iced::Alignment::Start)
    .width(Length::Fill);

    container(content)
        .style(iced::theme::Container::Box)
        .width(Length::Fill)
}

pub fn header<'a, Message>(
    title: &str,
    subtitle: &str,
) -> iced::widget::Column<'a, Message> {
    let title_text = text(title)
        .size(42)
        .style(style::primary_color());
    
    let subtitle_text = text(subtitle).size(24);

    iced::widget::column![title_text, subtitle_text]
        .spacing(10)
        .align_items(iced::Alignment::Center)
} 