use elastic_ermine::{es,util};

use iced::widget::{column, row};

mod assets;
mod view;


fn main() -> iced::Result {
    let window = iced::window::Settings {
        icon: Some(iced::window::icon::from_file_data(
            assets::APP_ICON_BUFFER, 
            Some(image::ImageFormat::Png)).map_err(|err| {
                iced::Error::WindowCreationFailed(Box::new(err))
            }
        )?),
        ..Default::default()
    };
    iced::application(MyApp::new, MyApp::update, MyApp::view)
        .title("Elastic Ermine")
        .window(window)
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    PageChanged(Page),
    APIView(view::api::Message),
    SettingsView(view::settings::Message),
    SearchView(view::search::Message),
}

#[derive(Debug, Default)]
struct MyApp{
    // general state
    current_page: Page,
    api_view: view::api::View,
    settings_view: view::settings::View,
    search_view: view::search::View,
}

#[derive(Debug, Default, Clone)]
enum Page {
    #[default]
    Search,
    API,
    Connection,
    Logs
}

impl MyApp {
    fn new() -> (Self, iced::Task<Message>) {
        (
            MyApp::default(),
            iced::Task::none()
        )
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::SearchView(message) => {
                match self.search_view.update(message) {
                    view::search::Action::None => iced::Task::none(),
                    view::search::Action::TryClientInvoke(context) => {
                        let client_res = self.settings_view.get_client();
                        view::search::View::try_invoke_with_client(client_res, context).map(Message::SearchView)
                    } 
                }
            }
            Message::APIView(message) => {
                match self.api_view.update(message) {
                    view::api::Action::None => iced::Task::none(),
                    view::api::Action::InvokeOperation { method, path, body } => {
                        let client_res = self.settings_view.get_client();
                        view::api::View::try_invoke_es_operation_with_client(
                            client_res, method, path, body
                        ).map(Message::APIView)
                    },
                }                
            },
            Message::SettingsView(message) => {
                match self.settings_view.update(message) {
                    view::settings::Action::Run(task) => task.map(Message::SettingsView),
                    view::settings::Action::None => iced::Task::none(),
                }
            },
            Message::PageChanged(page) => {
                self.current_page = page;

                iced::Task::none()
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Message> {
        column![
            self.header()
                .align_x(iced::alignment::Horizontal::Left)
                .align_y(iced::alignment::Vertical::Top)
                .width(iced::Fill)
                .height(iced::Shrink),
            iced::widget::rule::horizontal(2),
            row![
                self.window_selector()
                    .align_x(iced::alignment::Horizontal::Left)
                    .align_y(iced::alignment::Vertical::Top)
                    .width(iced::Shrink)
                    .height(iced::Fill),
                iced::widget::rule::vertical(2),    
                self.main_window()
                    .align_x(iced::alignment::Horizontal::Center)
                    .width(iced::Fill)
                    .height(iced::Fill),
            ]
            .width(iced::Fill)
            .spacing(5)
        ]
        .width(iced::Fill)
        .height(iced::Fill)
        .into()

    }

    fn header(&self) -> iced::widget::Container<'_, Message> {
        iced::widget::container(
            row![
                assets::app_icon()
                .height(50)
                .width(50),
                column![
                    iced::widget::text("Elastic Ermine")
                        .font(iced::Font { weight: iced::font::Weight::Bold, ..iced::Font::default()})
                        .size(30),
                    iced::widget::text("Search your data with Elasticsearch ... or Opensearch")
                        .font(iced::Font { weight: iced::font::Weight::Light, ..iced::Font::default()})
                        .size(14),
                ].spacing(5)
            ].spacing(10)
        )
    }

    fn window_selector(&self) -> iced::widget::Container<'_, Message> {
        iced::widget::container(
            column![
                iced::widget::tooltip(
                    iced::widget::button(assets::search_icon().width(iced::Shrink))
                        .on_press(Message::PageChanged(Page::Search))
                        .padding(iced::Padding::from([10, 10])),
                    "Search",
                    iced::widget::tooltip::Position::Right
                ).padding(10),
                iced::widget::tooltip(
                    iced::widget::button(assets::terminal_icon().width(iced::Shrink))
                        .on_press(Message::PageChanged(Page::API))
                        .padding(iced::Padding::from([10, 10])),
                    "Dev Console",
                    iced::widget::tooltip::Position::Right
                ),
                iced::widget::tooltip(
                    iced::widget::button(assets::settings_icon().width(iced::Shrink))
                        .on_press(Message::PageChanged(Page::Connection))
                        .padding(iced::Padding::from([10, 10])),
                    "Settings",
                    iced::widget::tooltip::Position::Right
                ),
                iced::widget::tooltip(
                    iced::widget::button(assets::file_icon().width(iced::Shrink))
                        .on_press(Message::PageChanged(Page::Logs))
                        .padding(iced::Padding::from([10, 10])),
                    "Logs",
                    iced::widget::tooltip::Position::Right
                )
                
            ]
        )
    }

    fn main_window(&self) -> iced::widget::Container<'_, Message> {
        iced::widget::container(
            match self.current_page {
                Page::Search => self.search_view.view().map(Message::SearchView),
                Page::Connection => self.settings_view.view().map(Message::SettingsView),
                Page::Logs => self.logs_section(),
                Page::API => self.api_view.view().map(Message::APIView),
            }
        )
    }

    fn logs_section(&self) -> iced::Element<'_, Message> {
        iced::widget::text("Logs WIP").into()
    }

}
