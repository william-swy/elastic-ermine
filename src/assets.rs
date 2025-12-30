use std::sync::LazyLock;

use iced::widget::{Svg, svg::Handle};

pub static APP_ICON_BUFFER: &[u8] = include_bytes!("../assets/logo.png");

static SETTINGS_ICON: LazyLock<Handle> = LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/settings.svg")));

static FILE_ICON: LazyLock<Handle> = LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/file-text.svg")));

static SEARCH_ICON: LazyLock<Handle> = LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/search.svg")));

static TERMINAL_ICON: LazyLock<Handle> = LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/terminal.svg")));

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