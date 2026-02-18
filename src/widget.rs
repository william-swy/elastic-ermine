pub mod circle;

pub struct RadioArea<Message, V> {
    produced_message: Message,
    val: V,
    selected_val: Option<V>,
    label: String,
    spacing: f32,
    width: iced::Length,
}

impl<Message, V> RadioArea<Message, V> 
where
    Message: Clone,
    V: Eq + Copy, 
{
    const DEFAULT_SPACING: f32 = 8.0;

    pub fn new<F>(
        label: impl Into<String>,
        value: V,
        selected: Option<V>,
        on_click: F
    ) -> Self 
    where
        F: FnOnce(V) -> Message
    {
        Self { 
            produced_message: on_click(value), 
            val: value, 
            selected_val: selected, 
            label: label.into(),
            spacing: Self::DEFAULT_SPACING,
            width: iced::Shrink,
        }
    }

    pub fn width(mut self, width: impl Into<iced::Length>) -> Self {
        self.width = width.into();
        self
    }
}

impl<'a, Message, V> From<RadioArea<Message, V>> 
    for iced::Element<'a, Message, iced::Theme, iced::Renderer> 
where
    Message: 'a + Clone,
    V: 'a + Eq + Copy
{
    fn from(radio_area: RadioArea<Message, V>) -> Self {
        let is_selected = Some(radio_area.val) == radio_area.selected_val;
        iced::widget::button(
            iced::widget::row![
                circle::Radio::new(is_selected),
                iced::widget::text(radio_area.label)
            ]
            .align_y(iced::Center)
            .spacing(radio_area.spacing)
        )
        .on_press(radio_area.produced_message)
        .style(|t: &iced::Theme, _s| {
            let palette = t.extended_palette();
            let base = iced::widget::button::Style {
                background: Some(iced::Color::TRANSPARENT.into()),
                border: iced::Border {
                    width: 1.0,
                    radius: 5.0.into(),
                    color: palette.background.weak.color,
                },
                text_color: t.palette().text,
                ..Default::default()
            };
            base
        })
        .width(radio_area.width)
        .into()
    }
}

pub fn section_with_header<'a, Message: 'a>(
    header: impl Into<iced::Element<'a, Message>>, 
    body: impl Into<iced::Element<'a, Message>>
) -> iced::widget::Container<'a, Message> {
    iced::widget::container(
        iced::widget::column![
            iced::widget::container(
                header
            ).padding(10),
            iced::widget::rule::horizontal(1),
            iced::widget::container(
                body
            ).padding(10)
        ]
    )
    .style(iced::widget::container::bordered_box)
}

pub fn section<'a, Message: 'a>(
    body: impl Into<iced::Element<'a, Message>>
) -> iced::widget::Container<'a, Message> {
    iced::widget::container(body)
        .padding(10)
        .style(iced::widget::container::bordered_box)
}
