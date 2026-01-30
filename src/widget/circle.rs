// Created based off of iced::widget::radio
// such that the radio widget can have padding and the padding area can also be clicked
use iced::{Background, Border, Color, Element, Length, Rectangle, Size};
use iced::advanced::layout;
use iced::advanced::widget;
use iced::advanced::renderer;

pub struct Radio<'class, Theme = iced::Theme>
where
    Theme: Catalog
{
    selected: bool,
    radius: f32,
    class: Theme::Class<'class>,
}

impl<'class, Theme> Radio<'class, Theme>
where
    Theme: Catalog
{
    const DEFAULT_SIZE: f32 = 16.0;

    pub fn new(selected: bool) -> Self {
        Self { 
            selected, 
            radius: Self::DEFAULT_SIZE,
            class: Theme::default()
        }
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }
}

impl<Message, Theme, Renderer> widget::Widget<Message, Theme, Renderer> for Radio<'_, Theme> 
where
    Theme: Catalog,
    Renderer: renderer::Renderer
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn layout(
        &mut self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(Size::new(self.radius, self.radius))
    }

    fn draw(
        &self,
        _tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: iced::advanced::Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {
        let style = theme.style(&self.class);

        let bounds = layout.bounds();
        let size = bounds.width;
        let dot_size = size / 2.0;

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    radius: (size / 2.0).into(),
                    width: style.border_width,
                    color: style.border_color,
                },
                ..renderer::Quad::default()
            },
            style.background,
        );


        if self.selected {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: bounds.x + dot_size / 2.0,
                        y: bounds.y + dot_size / 2.0,
                        width: bounds.width - dot_size,
                        height: bounds.height - dot_size,
                    },
                    border: iced::border::rounded(dot_size / 2.0),
                    ..renderer::Quad::default()
                },
                style.dot_color,
            )
        }
    }
}

impl<'class, Message, Theme, Renderer> From<Radio<'class, Theme>> for Element<'class, Message, Theme, Renderer>
where
    Theme: Catalog + 'class,
    Renderer: renderer::Renderer + 'class
{
    fn from(value: Radio<'class, Theme>) -> Element<'class, Message, Theme, Renderer> {
        Self::new(value)
    }
}

pub struct Style {
    pub background: Background,
    pub dot_color: Color,
    pub border_width: f32,
    pub border_color: Color,
}

pub trait Catalog {
    type Class<'class>;

    fn default<'class>() -> Self::Class<'class>;

    fn style(&self, class: &Self::Class<'_>) -> Style;
}

pub type StyleFn<'func, Theme> = Box<dyn Fn(&Theme) -> Style + 'func>;

impl Catalog for iced::Theme {
    type Class<'style_func> = StyleFn<'style_func, Self>;

    fn default<'style_func>() -> Self::Class<'style_func> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

pub fn default(theme: &iced::Theme) -> Style {
    let palette = theme.extended_palette();

    Style {
        background: Color::TRANSPARENT.into(),
        dot_color: palette.primary.strong.color,
        border_width: 1.0,
        border_color: palette.primary.strong.color,
    }
}