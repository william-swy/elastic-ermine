use elastic_ermine::{es,util};

use iced::widget::{column, row};

fn main() -> iced::Result {
    let window = iced::window::Settings {
        icon: Some(iced::window::icon::from_file_data(
            include_bytes!("../icons/logo.png"), 
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

    // Connection related messages
    AuthSelected(AuthChoice),
    UrlChanged(String),
    
    SelectCert,
    ObtainCert(Result<Option<(std::path::PathBuf, reqwest::Certificate)>, MyAppError>), // might need to rc certificate to avoid clone
    RemoveCert,

    BasicAuthUsernameChanged(String),
    BasicAuthPasswordChanged(String),

    AWSAuthRegionChanged(String),
    AWSAuthProfileChanged(String),

    TestConnectionButtonPressed,
    TestConnectionButtonResultReturned(Result<(), MyAppError>),

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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum AuthChoice {
    Basic,
    AWSSigV4,
    None,
}

#[derive(Debug)]
struct MyApp{
    // general state
    current_page: Page,

    // auth related state
    cert_selection_open: bool,

    auth_choice_type: Option<AuthChoice>,
    auth_choice_basic: es::BasicAuth,
    auth_choice_aws: es::AwsSigv4,

    es_url: (String, bool),
    selected_cert: Result<Option<(std::path::PathBuf, reqwest::Certificate)>, MyAppError>,

    test_connection_button_state: TestConnectionButtonState,

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

#[derive(Debug)]
enum TestConnectionButtonState {
    NotClicked,
    Waiting,
    Result(Result<(), MyAppError>)
}

#[derive(Debug, Default, Clone)]
enum Page {
    #[default]
    Search,
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

            auth_choice_type: Some(AuthChoice::None), 
            auth_choice_basic: es::BasicAuth {
                username: String::from("elastic"),
                password: None
            },
            auth_choice_aws: es::AwsSigv4 {
                region: String::from("us-east-1"),
                profile: None
            },
            es_url: (Default::default(), false),
            selected_cert: Ok(None),
            cert_selection_open: false,

            test_connection_button_state: TestConnectionButtonState::NotClicked,

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
            Message::AuthSelected(auth_choice) => {
                self.auth_choice_type = Some(auth_choice);
                iced::Task::none()
            }
            Message::UrlChanged(url) => {
                let valid = util::valid_url(&url);
                self.es_url = (url, valid);
                iced::Task::none()
            }
            Message::SelectCert => {
                if self.cert_selection_open {
                    iced::Task::none()
                } else {
                    self.cert_selection_open = true;
                    iced::window::oldest()
                        .and_then(|id| iced::window::run(id, MyApp::get_cert_from_file))
                        .then(iced::Task::future)
                        .map(Message::ObtainCert)
                }
            }
            Message::ObtainCert(res) => {
                self.cert_selection_open = false;
                self.selected_cert = res;
                iced::Task::none()
            }
            Message::RemoveCert => {
                self.selected_cert = Ok(None);
                iced::Task::none()
            },
            Message::BasicAuthUsernameChanged(username) => {
                self.auth_choice_basic.username = username;
                iced::Task::none()
            },
            Message::BasicAuthPasswordChanged(password) => {
                if password.is_empty() {
                    self.auth_choice_basic.password = None;
                } else {
                    self.auth_choice_basic.password = Some(password);
                }
                iced::Task::none()
            },
            Message::TestConnectionButtonPressed => {
                self.test_connection_button_state = TestConnectionButtonState::Waiting;
                iced::Task::perform(
                    MyApp::test_connection(
                        self.es_url.0.clone(), 
                        self.selected_cert.clone(), 
                        self.get_es_auth()), 
                    Message::TestConnectionButtonResultReturned)
            },
            Message::TestConnectionButtonResultReturned(res) => {
                self.test_connection_button_state = TestConnectionButtonState::Result(res);
                iced::Task::none()
            },
            Message::AWSAuthRegionChanged(new_region) => {
                self.auth_choice_aws.region = new_region;
                iced::Task::none()
            },
            Message::AWSAuthProfileChanged(profile) => {
                if profile.is_empty() {
                    self.auth_choice_aws.profile = None;
                } else {
                    self.auth_choice_aws.profile = Some(profile);
                }
                iced::Task::none()
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
                self.refresh_search_filter_button_state = RefreshSearchFilterButtonState::Waiting;
                iced::Task::perform(
                    MyApp::get_all_indices_and_aliases(
                        self.es_url.0.clone(), 
                        self.selected_cert.clone(), 
                        self.get_es_auth()
                    ),
                    Message::SearchFilterRefreshed
                )
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
                self.generic_search_search_button_state = GenericSearchSearchButtonState::Waiting;
                iced::Task::perform(
                    MyApp::perform_generic_search(
                        self.es_url.0.clone(), 
                        self.selected_cert.clone(), 
                        self.get_es_auth(),
                        self.generic_search_body_content.text(),
                        self.selected_indicies_and_aliases.iter().map(String::to_owned).collect(),
                    ),
                    Message::GenericSearchSearchButtonResultReturned
                )
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
            },
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
                iced::widget::button("Search")
                    .on_press(Message::PageChanged(Page::Search)),
                iced::widget::button("Connection")
                    .on_press(Message::PageChanged(Page::Connection)),
                iced::widget::button("Logs")
                    .on_press(Message::PageChanged(Page::Logs)),
            ].spacing(5)
        )
    }

    fn main_window(&self) -> iced::widget::Container<'_, Message> {
        iced::widget::container(
            match self.current_page {
                Page::Search => self.search_section(),
                Page::Connection => self.connection_config_section(),
                Page::Logs => self.logs_section(),
            }
        )
    }

    fn connection_config_section(&self) -> iced::widget::Column<'_, Message> {
        column![
            iced::widget::text("Elasticsearch Connection"),
            iced::widget::text("Elasticsearch URL"),
            iced::widget::text_input("https://elasticsearch.example.com:9200", &self.es_url.0)
                .on_input(Message::UrlChanged),
            self.es_url.0.is_empty().then(|| {
                iced::widget::text("Elasticsearch URL is required")
            }),
            (!self.es_url.0.is_empty() && !self.es_url.1).then(|| {
                iced::widget::text("Invalid URL format")
            }),
            iced::widget::text("Authentication Method"),
            row![
                iced::widget::radio("Basic Auth", AuthChoice::Basic, self.auth_choice_type, Message::AuthSelected),
                iced::widget::radio("AWS SigV4", AuthChoice::AWSSigV4, self.auth_choice_type, Message::AuthSelected),
                iced::widget::radio("None", AuthChoice::None, self.auth_choice_type, Message::AuthSelected)
            ],
            self.auth_choice_type.map(|choice| {
                match choice {
                    AuthChoice::Basic => Some(column![
                        iced::widget::text("Username"),
                        iced::widget::text_input("username", &self.auth_choice_basic.username)
                            .on_input(Message::BasicAuthUsernameChanged),
                        iced::widget::text("Password (Optional)"),
                        iced::widget::text_input("", self.auth_choice_basic.password.as_ref().map(|s| s.as_str()).unwrap_or(""))
                            .on_input(Message::BasicAuthPasswordChanged),
                    ]),
                    AuthChoice::AWSSigV4 => Some(column![
                        iced::widget::text("AWS Region"),
                        iced::widget::text_input("us-east-1", &self.auth_choice_aws.region)
                            .on_input(Message::AWSAuthRegionChanged),
                        iced::widget::text("AWS Profile"),
                        iced::widget::text_input("default", self.auth_choice_aws.profile.as_ref().map(|s| s.as_str()).unwrap_or(""))
                            .on_input(Message::AWSAuthProfileChanged)
                    ]),
                    AuthChoice::None => None,
                }
            }).flatten(),
            iced::widget::text("CA certificate file (optional)"),
            self.selected_cert.as_ref().ok().and_then(Option::as_ref)
                .map(|(cert_path, _)| {
                    row![
                        iced::widget::text(
                            cert_path.file_name()
                                .expect("Unable to get file name of cert")
                                .to_string_lossy()
                        ),
                        iced::widget::button(iced::widget::text("x"))
                            .on_press(Message::RemoveCert)
                    ]
                })
                .unwrap_or(
                    row![iced::widget::button("Upload Certificate (.pem or .der)")
                            .on_press_maybe((!self.cert_selection_open).then(|| Message::SelectCert))]
                ),
            self.selected_cert.as_ref().err().map(|x| {
                iced::widget::text(format!("Failed to get certificate\n {}", x.reason))
            }),
            self.test_connection_button(),
        ]
    }

    fn test_connection_button(&self) -> iced::widget::Column<'_, Message> {
        match &self.test_connection_button_state {
            TestConnectionButtonState::NotClicked => {
                return column![iced::widget::button("Test connection")
                    .on_press(Message::TestConnectionButtonPressed)];
            },
            TestConnectionButtonState::Waiting => {
                return column![iced::widget::button("Test connection\t waiting...")];
            },
            TestConnectionButtonState::Result(res) => {
                return column![
                    iced::widget::button("Test connection")
                        .on_press(Message::TestConnectionButtonPressed),
                    res.as_ref().map(|_| {
                        iced::widget::text("Connection successful")
                    }).unwrap_or_else(|err| {
                        iced::widget::text(format!("Connection failed\n{}", err.reason))
                    })
                ];
            },
        }
    }


    fn search_section(&self) -> iced::widget::Column<'_, Message> {
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
                    iced::widget::text(
                        format!("{:?}", res)
                    )
                    .align_x(iced::Center)
                    .align_y(iced::Center),
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

    fn logs_section(&self) -> iced::widget::Column<'_, Message> {
        column![iced::widget::text("Logs WIP")]
    }

    fn get_es_auth(&self) -> Option<es::Auth> {
        self.auth_choice_type.map(|auth_type| {
            match auth_type {
                AuthChoice::Basic => Some(es::Auth::BASIC(self.auth_choice_basic.clone())),
                AuthChoice::AWSSigV4 => Some(es::Auth::AWS(self.auth_choice_aws.clone())),
                AuthChoice::None => None,
            }
        }).flatten()
    }

    fn get_cert_from_file(window: &dyn iced::Window) -> impl Future<Output = Result<Option<(std::path::PathBuf, reqwest::Certificate)>, MyAppError>> + use<> {
        let current_dir = std::env::current_dir()
            .expect("Unable to open current working directory");

        let dialog = rfd::AsyncFileDialog::new()
            .add_filter("CA", &["pem", "der"])
            .set_directory(current_dir)
            .set_title("Select CA Cert")
            .set_parent(&window);
        
        async move {
            let file = dialog.pick_file().await;

            let res = match file {
                Some(file_handle) => {
                    let path: std::path::PathBuf = file_handle.into();
                    let contents = tokio::fs::read_to_string(&path).await
                        .map_err(|err| MyAppError{reason: format!("Unable to read {}, error: {}", path.to_string_lossy(), err.kind())})?;
                    let certificate = if util::path_has_extension(&path,"pem") {
                        reqwest::Certificate::from_pem(contents.as_bytes())
                            .map_err(|err| 
                                MyAppError{
                                    reason: format!("Unable to interpret {} as pem, error: {}", path.to_string_lossy(), err.to_string())
                                })
                    } else if util::path_has_extension(&path,"der") {
                        reqwest::Certificate::from_der(contents.as_bytes())
                            .map_err(|err|
                                MyAppError{
                                    reason: format!("Unable to interpret {} as der, error: {}", path.to_string_lossy(), err.to_string())
                                })
                    } else {
                        Err(MyAppError{
                            reason: format!("{} is unsupported file type", path.to_string_lossy())
                        })
                    }?;

                    Some((path, certificate))
                },
                None => None,
            };

            return Ok(res);
        }
        
    }

    async fn test_connection(
        url: String, 
        selected_cert: Result<Option<(std::path::PathBuf, reqwest::Certificate)>, MyAppError>, 
        auth: Option<es::Auth>
    ) -> Result<(), MyAppError> {
        let mut client = es::ElasticsearchClient::new(url)
            .map_err(|err| MyAppError{reason: err.to_string()})?;

        if let Some((_, cert)) = selected_cert? {
            client.use_custom_certificate(cert)
                .map_err(|err| MyAppError{reason: err.to_string()})?;
        }

        if let Some(auth_choice) = auth {
            client.use_auth(auth_choice);
        }

        return client.test_connection().await
            .map_err(|err| MyAppError{reason: err.to_string()});
    }

    async fn get_all_indices_and_aliases(
        url: String, 
        selected_cert: Result<Option<(std::path::PathBuf, reqwest::Certificate)>, MyAppError>, 
        auth: Option<es::Auth>
    ) -> Result<(Vec<String>, Vec<String>), MyAppError> {
        let mut client = es::ElasticsearchClient::new(url)
            .map_err(|err| MyAppError{reason: err.to_string()})?;

        if let Some((_, cert)) = selected_cert? {
            client.use_custom_certificate(cert)
                .map_err(|err| MyAppError{reason: err.to_string()})?;
        }

        if let Some(auth_choice) = auth {
            client.use_auth(auth_choice);
        }

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
        url: String, 
        selected_cert: Result<Option<(std::path::PathBuf, reqwest::Certificate)>, MyAppError>, 
        auth: Option<es::Auth>,
        body: String,
        indicies_and_aliases: Vec<String>
    ) -> Result<es::types::OperationSearchResult, MyAppError> {
        let mut client = es::ElasticsearchClient::new(url)
            .map_err(|err| MyAppError{reason: err.to_string()})?;

        if let Some((_, cert)) = selected_cert? {
            client.use_custom_certificate(cert)
                .map_err(|err| MyAppError{reason: err.to_string()})?;
        }

        if let Some(auth_choice) = auth {
            client.use_auth(auth_choice);
        }

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
