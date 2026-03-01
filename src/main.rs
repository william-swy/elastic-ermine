use elastic_ermine::{es,util};

mod assets;
mod widget;
mod dev_tools;
mod search;
mod settings;
mod sidebar;


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
    DevToolsView(dev_tools::Message),
    SettingsView(settings::Message),
    SearchView(search::Message),
    Sidebar(sidebar::Message),
}

#[derive(Debug, Default)]
struct MyApp{
    dev_tools_view: dev_tools::View,
    settings_view: settings::View,
    search_view: search::View,
    sidebar_view: sidebar::View,
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
                    search::Action::None => iced::Task::none(),
                    search::Action::TryClientInvoke(context) => {
                        let client_res = self.settings_view.get_client();
                        search::View::try_invoke_with_client(client_res, context).map(Message::SearchView)
                    } 
                }
            }
            Message::DevToolsView(message) => {
                match self.dev_tools_view.update(message) {
                    dev_tools::Action::None => iced::Task::none(),
                    dev_tools::Action::InvokeOperation { method, path, body } => {
                        let client_res = self.settings_view.get_client();
                        dev_tools::View::try_invoke_es_operation_with_client(
                            client_res, method, path, body
                        ).map(Message::DevToolsView)
                    },
                }                
            },
            Message::SettingsView(message) => {
                match self.settings_view.update(message) {
                    settings::Action::Run(task) => task.map(Message::SettingsView),
                    settings::Action::None => iced::Task::none(),
                }
            },
            Message::Sidebar(message) => {
                match self.sidebar_view.update(message) {
                    sidebar::Action::None => iced::Task::none(),
                }
            },
        }
    }

    fn view(&self) -> iced::Element<'_, Message> {
        iced::widget::row![
            self.sidebar_view.view().map(Message::Sidebar),
            self.main_window()
                .align_x(iced::alignment::Horizontal::Center)
                .width(iced::Fill)
                .height(iced::Fill),
        ]
        .spacing(10)
        .width(iced::Fill)
        .into()
    }

    fn main_window(&self) -> iced::widget::Container<'_, Message> {
        iced::widget::container(
            match self.sidebar_view.current_page() {
                sidebar::Page::Search => self.search_view.view().map(Message::SearchView),
                sidebar::Page::Connection => self.settings_view.view().map(Message::SettingsView),
                sidebar::Page::Logs => self.logs_section(),
                sidebar::Page::DevConsole => self.dev_tools_view.view().map(Message::DevToolsView),
            }
        )
    }

    fn logs_section(&self) -> iced::Element<'_, Message> {
        iced::widget::text("Logs WIP").into()
    }

}
