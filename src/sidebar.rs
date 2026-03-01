use crate::{assets};

#[derive(Debug, Default, Clone)]
pub enum Page {
    #[default]
    Search,
    DevConsole,
    Connection,
    Logs
}

#[derive(Debug, Clone)]
pub enum Message {
    PageChanged(Page),
    ExpandedClicked,
}

pub enum Action {
    None,
}

#[derive(Debug, Default)]
pub struct View {
    toolbar_expanded: bool,
    current_page: Page,
}

const ICON_TEXT_SPACING: u32 = 5;

impl View {
    #[must_use]
    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::PageChanged(page) => {
                self.current_page = page;
                Action::None
            },
            Message::ExpandedClicked =>  {
                self.toolbar_expanded = !self.toolbar_expanded;
                Action::None
            },
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let sidebar = iced::widget::Column::with_children(self.body())
            .push(iced::widget::space::vertical())
            .push(self.footer());

        iced::widget::container(
            sidebar
            .width(
                if self.toolbar_expanded {
                    120.into() // Must hard set width otherwise text will not appear
                } else {
                    iced::Shrink
                }
            )
        )
        .width(iced::Shrink)
        .height(iced::Fill)
        .into()
    }

    pub fn current_page(&self) -> Page {
        self.current_page.clone()
    }

    fn body(&self) -> impl Iterator<Item = iced::Element<'_, Message>> {
        // Entries in the form of (icon, expanded text, message)
        let buttons = [
            (assets::search_icon(), "Search", Message::PageChanged(Page::Search)),
            (assets::terminal_icon(), "Dev Tools", Message::PageChanged(Page::DevConsole)),
            (assets::settings_icon(), "Settings", Message::PageChanged(Page::Connection)),
            (assets::file_icon(), "Logs", Message::PageChanged(Page::Logs)),
        ];

        buttons.into_iter().map(|(icon, expanded_text, message)| {
            if self.toolbar_expanded {
                iced::widget::button(
                    iced::widget::row![
                        icon.width(iced::Shrink),
                        iced::widget::text(expanded_text)
                    ]
                    .spacing(ICON_TEXT_SPACING)
                )
                .width(iced::Fill)
                .on_press(message)
                .into()
            } else {
                iced::widget::tooltip(
                iced::widget::button(
                    icon.width(iced::Shrink)
                )
                .on_press(message), 
                iced::widget::container(
                    iced::widget::text(expanded_text))
                    .padding(5)
                    .style(iced::widget::container::bordered_box)
                    .width(iced::Shrink)
                    .height(iced::Shrink),
                    iced::widget::tooltip::Position::Right
                )
                .into()
            }
        })
    }

    // Return type not set in stone
    fn footer(&self) -> iced::Element<'_, Message> {
        if self.toolbar_expanded {
            iced::widget::button(
                iced::widget::row![
                    assets::arrow_left_line_icon().width(iced::Shrink),
                    iced::widget::text("Collapse")
                ]
                .spacing(ICON_TEXT_SPACING)
            )
        } else {
            iced::widget::button(assets::arrow_right_line_icon().width(iced::Shrink))
        }
        .width(iced::Fill)
        .on_press(Message::ExpandedClicked)
        .into()
    }
}