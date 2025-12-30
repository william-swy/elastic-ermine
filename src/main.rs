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
    // General type messages
    PageChanged(Page),
    APIView(view::api::Message),
    SettingsView(view::settings::Message),

    // Search related messages
    SearchTypeChanged(SearchType),
    SearchFilterRefreshPressed,
    SearchFilterRefreshed(Result<(Vec<String>, Vec<String>), MyAppError>),
    SearchFilterAddItem(String),
    SearchFilterRemoveItem(String),

    GenericSearchBodyEditorActionPerformed(iced::widget::text_editor::Action),
    GenericSearchSearchButtonPressed,
    // perhaps place OperationSearchResult in arc to prevent clone as contents can potentially
    // be quite large
    GenericSearchSearchButtonResultReturned(Result<es::types::OperationSearchResult, MyAppError>) 
    
}

#[derive(Debug, Clone)]
struct MyAppError {
    reason: String,
}

#[derive(Debug)]
struct MyApp{
    // general state
    current_page: Page,
    api_view: view::api::View,
    settings_view: view::settings::View,

    // search related state
    search_type: SearchType,
    // search_error: Option<String>,

    refresh_search_filter_button_state: RefreshSearchFilterButtonState,
    refresh_search_filter_error: Option<String>,
    indicies: Vec<String>,
    aliases: Vec<String>,
    selected_indicies_and_aliases: std::collections::HashSet<String>,
    generic_search_body_content: iced::widget::text_editor::Content,
    generic_search_search_button_state: GenericSearchSearchButtonState,
    generic_search_display_content: GenericSearchDisplaySectionValue,
    // generic_search_result: Option<es::types::OperationSearchResult>,
}

#[derive(Debug, Default)]
enum GenericSearchDisplaySectionValue {
    #[default]
    Default,
    Error(String),
    Result(es::types::OperationSearchResult)
}

#[derive(Debug, Default, Clone)]
enum Page {
    #[default]
    Search,
    API,
    Connection,
    Logs
}

#[derive(Debug, Default, Clone)]
enum SearchType {
    StringSearch,
    #[default]
    GenericSearch,
    EndpointOperations,
}

#[derive(Debug, Default)]
enum RefreshSearchFilterButtonState {
    #[default]
    Ready,
    Waiting,
}

#[derive(Debug, Default)]
enum GenericSearchSearchButtonState {
    #[default]
    Ready,
    Waiting,
}

impl Default for MyApp {
    fn default() -> Self {
        Self { 
            current_page: Default::default(),
            api_view: Default::default(),
            settings_view: Default::default(),

            search_type: Default::default(),
            // search_error: None,
            refresh_search_filter_button_state: Default::default(),
            refresh_search_filter_error: None,
            indicies: Vec::new(),
            aliases: Vec::new(),
            selected_indicies_and_aliases: std::collections::HashSet::new(),
            generic_search_body_content: Default::default(),
            generic_search_search_button_state: Default::default(),
            // generic_search_result: None, // make errors and this result into an enum
            generic_search_display_content: Default::default()
        }
    }
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
            Message::APIView(_message) => {
                // TODO
                iced::Task::none()
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
            },
            Message::SearchTypeChanged(search_type) => {
                self.search_type = search_type;
                iced::Task::none()
            },
            Message::GenericSearchBodyEditorActionPerformed(action) => {
                self.generic_search_body_content.perform(action);
                iced::Task::none()
            },
            Message::SearchFilterRefreshPressed => {
                match self.settings_view.get_client() {
                    Ok(client) => {
                        self.refresh_search_filter_button_state = RefreshSearchFilterButtonState::Waiting;
                        iced::Task::perform(
                        MyApp::get_all_indices_and_aliases(client),
                        Message::SearchFilterRefreshed
                        )
                    },
                    Err(reason) => {
                        self.aliases = Vec::new();
                        self.indicies = Vec::new();
                        self.refresh_search_filter_error = Some(format!("Failed to refresh filters: {}", reason));
                        // TODO: update selected list using the updated list of aliases and indicies. Existing
                        // selected indicies and aliases that are in the update list can remain selected. Otherwise
                        // they should be unselected. For now refreshing will remove all selected filters
                        self.selected_indicies_and_aliases.clear();
                        iced::Task::none()
                    },
                }
            },
            Message::SearchFilterRefreshed(res) => {
                self.refresh_search_filter_button_state = RefreshSearchFilterButtonState::Ready;

                match res {
                    Ok((indicies, aliases)) => {
                        self.aliases = aliases;
                        self.indicies = indicies;
                        self.refresh_search_filter_error = None;
                    },
                    Err(err) => {
                        self.aliases = Vec::new();
                        self.indicies = Vec::new();
                        self.refresh_search_filter_error = Some(format!("Failed to refresh filters: {}", err.reason)); 
                    },
                };

                // TODO: update selected list using the updated list of aliases and indicies. Existing
                // selected indicies and aliases that are in the update list can remain selected. Otherwise
                // they should be unselected. For now refreshing will remove all selected filters
                self.selected_indicies_and_aliases.clear();
                iced::Task::none()
            },
            Message::SearchFilterAddItem(filter_item) => {
                self.selected_indicies_and_aliases.insert(filter_item);
                iced::Task::none()
            },
            Message::SearchFilterRemoveItem(filter_item) => {
                self.selected_indicies_and_aliases.remove(&filter_item);
                iced::Task::none()
            },
            Message::GenericSearchSearchButtonPressed => {
                match self.settings_view.get_client() {
                    Ok(client) => {
                        self.generic_search_search_button_state = GenericSearchSearchButtonState::Waiting;
                        iced::Task::perform(
                        MyApp::perform_generic_search(
                            client,
                            self.generic_search_body_content.text(),
                            self.selected_indicies_and_aliases.iter().map(String::to_owned).collect(),
                        ),
                        Message::GenericSearchSearchButtonResultReturned
                    )
                    },
                    Err(reason) => {
                        self.generic_search_display_content = GenericSearchDisplaySectionValue::Error(format!("Failed to search: {}", reason));
                        iced::Task::none()
                    },
                }
                
            },
            Message::GenericSearchSearchButtonResultReturned(operation_search_result) => {
                self.generic_search_search_button_state = GenericSearchSearchButtonState::Ready;
                match operation_search_result {
                    Ok(res) => {
                        self.generic_search_display_content = GenericSearchDisplaySectionValue::Result(res);
                    },
                    Err(err) => {
                        self.generic_search_display_content = GenericSearchDisplaySectionValue::Error(format!("Failed to search: {}", err.reason));
                    },
                }

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
            column![
                iced::widget::text("Elastic Ermine")
                    .font(iced::Font { weight: iced::font::Weight::Bold, ..iced::Font::default()})
                    .size(30),
                iced::widget::text("Search your data with Elasticsearch ... or Opensearch")
                    .font(iced::Font { weight: iced::font::Weight::Light, ..iced::Font::default()})
                    .size(14),
            ].spacing(5)
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
                    "HTTP",
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
                Page::Search => self.search_section(),
                Page::Connection => self.settings_view.view().map(Message::SettingsView),
                Page::Logs => self.logs_section(),
                Page::API => self.api_view.view().map(Message::APIView),
            }
        )
    }

    fn search_section(&self) -> iced::Element<'_, Message> {
        column![
            self.search_options(),
            match self.search_type {
                SearchType::StringSearch => self.search_string_search(),
                SearchType::GenericSearch => self.generic_search_view(),
                SearchType::EndpointOperations => self.search_endpoint_operations(),
            }
            .align_x(iced::alignment::Horizontal::Center)
            .width(iced::Fill)
            .height(iced::Fill),
        ].spacing(10)
        .into()
    }

    fn search_options(&self) -> iced::widget::Row<'_, Message> {
        row![
            iced::widget::button("String Search")
                .on_press(Message::SearchTypeChanged(SearchType::StringSearch)),
            iced::widget::button("Generic Search")
                .on_press(Message::SearchTypeChanged(SearchType::GenericSearch)),
            iced::widget::button("Endpoint Operations")
                .on_press(Message::SearchTypeChanged(SearchType::EndpointOperations)),
        ]
    }

    fn search_string_search(&self) -> iced::widget::Container<'_, Message> {
        iced::widget::container(
            iced::widget::text("String Search WIP")
        )
    }

    fn generic_search_view(&self) -> iced::widget::Container<'_, Message> {
        iced::widget::container(
            row![
                self.search_filters()
                    .align_x(iced::alignment::Horizontal::Left)
                    .width(iced::Shrink)
                    .height(iced::Shrink),
                column![
                    iced::widget::text_editor(&self.generic_search_body_content) // perhaps make this scrollable
                        .on_action(Message::GenericSearchBodyEditorActionPerformed)
                        .height(iced::Length::FillPortion(3)),
                    self.generic_search_search_button(),
                    self.generic_search_result_view() // perhaps make this scrollable
                        .width(iced::Fill)
                        .height(iced::Length::FillPortion(2))
                ]
                .width(iced::Fill)
                .height(iced::Fill)
            ]
        )
    }

    fn generic_search_result_view(&self) -> iced::widget::Container<'_, Message> {
        iced::widget::container(
            match &self.generic_search_display_content {
                GenericSearchDisplaySectionValue::Default => column![
                    iced::widget::text(
                        "Enter a query above to search your Elasticsearch cluster. Use the filters on the left to refine your results."
                    )
                    .align_x(iced::Center)
                    .align_y(iced::Center),
                ],
                GenericSearchDisplaySectionValue::Error(err) => column![
                    iced::widget::text(
                        format!("ERROR\nSearch failed: {}", err)
                    )
                    .align_x(iced::Center)
                    .align_y(iced::Center),
                ],
                GenericSearchDisplaySectionValue::Result(res) => column![
                    iced::widget::text(format!("Results")),
                    iced::widget::text(format!("Number of hits: {}", res.hits.hits.len())),
                    iced::widget::scrollable(
                        column(
                        res.hits.hits.iter().map(|item|
                            iced::widget::text(
                                serde_json::to_string_pretty(item).unwrap_or(format!("Failed to display {:?}", item))
                            ).into()
                        ))
                    )
                    .width(iced::Fill)
                    .height(iced::Fill)
                ],
            }
            
        )
    }

    fn generic_search_search_button(&self) -> iced::widget::Button<'_, Message> {
        let produced_message = match self.generic_search_search_button_state {
            GenericSearchSearchButtonState::Ready => Some(Message::GenericSearchSearchButtonPressed),
            GenericSearchSearchButtonState::Waiting => None,
        };
        iced::widget::button("Search")
            .on_press_maybe(produced_message)
            .width(iced::Shrink)
            .height(iced::Shrink)
    }

    fn search_endpoint_operations(&self) -> iced::widget::Container<'_, Message> {
        iced::widget::container(
            iced::widget::text("Endpoint operations WIP")
        )
    }

    fn search_filters(&self) -> iced::widget::Column<'_, Message> {
        let filters = column![
            row![
                iced::widget::text("Filters"),
                match self.refresh_search_filter_button_state {
                    RefreshSearchFilterButtonState::Ready => iced::widget::button("Refresh")
                        .on_press(Message::SearchFilterRefreshPressed),
                    RefreshSearchFilterButtonState::Waiting => iced::widget::button("Refreshing..."),
                },
            ],
            self.refresh_search_filter_error.as_ref()
                .map(|err| iced::widget::text(format!("Failed to refresh filters: {}", err))),
            iced::widget::text("Indicies"),
        ];

        let filters = filters.extend(
            self.indicies
                .iter()
                .map(|index|
                    iced::widget::checkbox(self.selected_indicies_and_aliases.contains(index))
                        .label(index)
                        .on_toggle(|toggled| {
                            if toggled {
                                Message::SearchFilterAddItem(index.to_owned())
                            } else {
                                Message::SearchFilterRemoveItem(index.to_owned())
                            }
                        })
                        .into()));

        let filters = filters.push(iced::widget::text("Aliases"));

        filters.extend(
            self.aliases
                .iter()
                .map(|alias|
                    iced::widget::checkbox(self.selected_indicies_and_aliases.contains(alias))
                        .label(alias)
                        .on_toggle(|toggled| {
                            if toggled {
                                Message::SearchFilterAddItem(alias.to_owned())
                            } else {
                                Message::SearchFilterRemoveItem(alias.to_owned())
                            }
                        })
                        .into()))
    }

    fn logs_section(&self) -> iced::Element<'_, Message> {
        iced::widget::text("Logs WIP").into()
    }

    async fn get_all_indices_and_aliases(
        client: es::ElasticsearchClient
    ) -> Result<(Vec<String>, Vec<String>), MyAppError> {
        let indicies = client.get_indicies().await
            .map_err(|err| MyAppError{reason: err.to_string()})?
            .into_iter()
            .map(|idx| idx.name)
            .collect::<Vec<String>>();

        let aliases = client.get_aliases().await
            .map_err(|err| MyAppError{reason: err.to_string()})?
            .into_iter()
            .map(|alias| alias.name)
            .collect::<Vec<String>>();

        Ok((indicies, aliases))
    }

    async fn perform_generic_search(
        client: es::ElasticsearchClient,
        body: String,
        indicies_and_aliases: Vec<String>
    ) -> Result<es::types::OperationSearchResult, MyAppError> {
        let search_body = if body.is_empty() {
            None
        } else {
            Some(serde_json::from_str::<serde_json::Value>(&body)
                    .map_err(|err| MyAppError{reason: err.to_string()})?)
        };

        let res = client.search(&indicies_and_aliases, search_body.as_ref()).await
            .map_err(|err| MyAppError{reason: err.to_string()})?;

        Ok(res)
    }

}
