use std::sync::LazyLock;

use iced::widget::{Svg, svg::Handle};

pub static APP_ICON_BUFFER: &[u8] = include_bytes!("../assets/logo.png");

static SETTINGS_ICON: LazyLock<Handle> = LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/settings.svg")));

static FILE_ICON: LazyLock<Handle> = LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/file-text.svg")));

static SEARCH_ICON: LazyLock<Handle> = LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/search.svg")));

static TERMINAL_ICON: LazyLock<Handle> = LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/terminal.svg")));

static REFRESH_ICON: LazyLock<Handle> = LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/refresh-cw.svg")));

static LOADING_ICON: LazyLock<Handle> = LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/loader.svg")));

static CHEVRON_DOWN_ICON: LazyLock<Handle> = LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/chevron-down.svg")));

static CHEVRON_RIGHT_ICON: LazyLock<Handle> = LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/chevron-right.svg")));

static CHEVRON_LR_ICON: LazyLock<Handle> = LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/chevrons-left-right.svg"))); 

static X_ICON: LazyLock<Handle> = LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/x.svg")));

static APP_ICON: LazyLock<iced::widget::image::Handle> = LazyLock::new(|| iced::widget::image::Handle::from_bytes(APP_ICON_BUFFER));



pub fn settings_icon() -> Svg<'static> {
    Svg::new(SETTINGS_ICON.clone())
}

pub fn file_icon() -> Svg<'static> {
    Svg::new(FILE_ICON.clone())
}

pub fn search_icon() -> Svg<'static> {
    Svg::new(SEARCH_ICON.clone())
}

pub fn terminal_icon() -> Svg<'static> {
    Svg::new(TERMINAL_ICON.clone())
}

pub fn refresh_icon() -> Svg<'static> {
    Svg::new(REFRESH_ICON.clone())
}

pub fn loading_icon() -> Svg<'static> {
    Svg::new(LOADING_ICON.clone())
}

pub fn chevron_down_icon() -> Svg<'static> {
    Svg::new(CHEVRON_DOWN_ICON.clone())
}

pub fn chevron_right_icon() -> Svg<'static> {
    Svg::new(CHEVRON_RIGHT_ICON.clone())
}

pub fn chevron_lr_icon() -> Svg<'static> {
    Svg::new(CHEVRON_LR_ICON.clone())
}

pub fn x_icon() -> Svg<'static> {
    Svg::new(X_ICON.clone())
}

pub fn app_icon() -> iced::widget::Image {
    iced::widget::Image::new(APP_ICON.clone())
}