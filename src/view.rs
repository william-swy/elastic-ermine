pub mod api {
    use crate::es;
    use iced::widget::{column, row};

    pub enum Action {
        None,
        InvokeOperation{
            method: es::ElasticSearchMethodType,
            path: String,
            body: String,
        },
    }

    #[derive(Debug, Clone)]
    pub enum Message {
        RequestTypeSelected(es::ElasticSearchMethodType),
        RequestBodyEditPerformed(iced::widget::text_editor::Action),
        PathUpdated(String),
        HTTPOperationReturned(Result<serde_json::Value, String>), // Perhaps Value should be a reference for large results
        SendButtonPressed,
    }

    #[derive(Debug, Default)]
    pub struct View {
        request_type: es::ElasticSearchMethodType,
        request_path: String,
        request_body: iced::widget::text_editor::Content,
        result: Option<Result<serde_json::Value, String>>,

        send_button_state: SendButtonState,
    }

    #[derive(Debug, Default)]
    enum SendButtonState {
        #[default]
        Ready,
        Waiting,
    }

    const SUPPORTED_METHODS: [es::ElasticSearchMethodType; 5] =[
        es::ElasticSearchMethodType::GET,
        es::ElasticSearchMethodType::POST,
        es::ElasticSearchMethodType::PUT,
        es::ElasticSearchMethodType::PATCH,
        es::ElasticSearchMethodType::DELETE,
    ];

    impl View {
        pub fn view(&self) -> iced::Element<'_, Message> {
            row![
                iced::widget::container(
                    self.editor()
                )
                .width(iced::FillPortion(1))
                .height(iced::Fill),
                iced::widget::rule::vertical(2),
                iced::widget::container(
                    self.response()
                )
                .width(iced::FillPortion(1))
                .height(iced::Fill),
            ].into()
        }

        pub fn update(&mut self, message: Message) -> Action {
            match message {
                Message::RequestTypeSelected(method) => {
                    self.request_type = method;
                    Action::None
                },
                Message::RequestBodyEditPerformed(action) => {
                    self.request_body.perform(action);
                    Action::None
                },
                Message::PathUpdated(new_path) => {
                    self.request_path = new_path;
                    Action::None
                },
                Message::HTTPOperationReturned(value) => {
                    self.send_button_state = SendButtonState::Ready;
                    self.result = Some(value);
                    Action::None
                },
                Message::SendButtonPressed => {
                    self.send_button_state = SendButtonState::Waiting;
                    Action::InvokeOperation { 
                        method: self.request_type, 
                        path: self.request_path.clone(), 
                        body: self.request_body.text()
                    }
                },
            }
        }

        pub fn editor(&self) -> iced::widget::Column<'_, Message> {
            column![
                row![
                    iced::widget::pick_list(
                        SUPPORTED_METHODS,
                        Some(&self.request_type),
                        Message::RequestTypeSelected
                    ).width(iced::Shrink),
                    iced::widget::text_input("_search", &self.request_path)
                        .on_input(Message::PathUpdated)
                        .width(iced::Fill),
                    iced::widget::button("Send")
                        .on_press(Message::SendButtonPressed)
                        .width(iced::Shrink)
                ],
                row![
                    iced::widget::text("REQUEST BODY (JSON)"),
                    iced::widget::space::horizontal(),
                    iced::widget::button("Format"),
                ],
                iced::widget::text_editor(&self.request_body)
                    .on_action(Message::RequestBodyEditPerformed)
                    .height(iced::Length::FillPortion(3)),
            ]
        }

        pub fn response(&self) -> iced::Element<'_, Message> {
            column![
                iced::widget::text("Results"),
                self.result.as_ref().map(|res| {
                    match res {
                        Ok(val) => iced::widget::text(
                            serde_json::to_string_pretty(val).unwrap_or_else(|err| 
                                format!("{} Failed to deserialize {:?}", err, val)
                            )
                        ),
                        Err(msg) => iced::widget::text(msg),
                    }
                })
            ].into()
        }

        pub fn try_invoke_es_operation_with_client(
            client_res: Result<es::ElasticsearchClient, String>,
            method: es::ElasticSearchMethodType,
            path: String,
            body: String
        ) -> iced::Task<Message> {
            iced::Task::perform(
                async move {
                    let client = client_res?;

                    let body_json = (!body.is_empty()).then(|| {
                        serde_json::from_str::<serde_json::Value>(&body)
                    })
                    .transpose()
                    .map_err(|err| err.to_string())?;
                    
                    client.operation(method, path.as_ref(), body_json.as_ref()).await.map_err(|err| {
                        err.to_string()
                    })
                }, 
                Message::HTTPOperationReturned
            )
        }
    }
}

pub mod settings {
    use crate::{es, util};
    use iced::widget::{column, row};

    #[derive(Debug, Clone)]
    pub enum Message {
        UrlChanged(String),
        AuthChoiceSelected(AuthChoice),
        BasicAuthUsernameChanged(String),
        BasicAuthPasswordChanged(String),
        AwsAuthRegionChanged(String),
        AwsAuthProfileChanged(String),
        CertRemoved,
        CertSelectionClicked,
        CertObtained(Result<Option<(std::path::PathBuf, reqwest::Certificate)>, String>), // might need to rc certificate to avoid clone
        TestConnectionButtonPressed,
        TestConnectionButtonResultReturned(Result<(), String>),
    }

    pub enum Action {
        Run(iced::Task<Message>),
        None,
    }

    #[derive(Debug)]
    pub struct View {
        es_url: String,
        auth_choice_type: Option<AuthChoice>, // TODO remove Option and place the option in the radio button widget
        basic_auth_data: es::BasicAuth,
        aws_sigv4_data: es::AwsSigv4,
        selected_cert: Result<Option<(std::path::PathBuf, reqwest::Certificate)>, String>,
        cert_selection_open: bool, // can be agregatted with selected_cert via an enum
        test_connection_button_state: TestConnectionButtonState,
    }

    impl Default for View {
        fn default() -> Self {
            Self { 
                es_url: Default::default(), 
                auth_choice_type: Some(Default::default()),
                basic_auth_data: Default::default(),
                aws_sigv4_data: es::AwsSigv4 {
                    region: "us-east-1".to_owned(),
                    profile: None,
                },
                selected_cert: Ok(None),
                cert_selection_open: false,
                test_connection_button_state: Default::default(),
            }
        }
    }

    #[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
    pub enum AuthChoice {
        Basic,
        AWSSigV4,
        #[default]
        None,
    }

    #[derive(Debug, Default)]
    enum TestConnectionButtonState {
        #[default]
        NotClicked,
        Waiting,
        Result(Result<(), String>)
    }

    impl View {
        #[must_use]
        pub fn update(&mut self, message: Message) -> Action {
            match message {
                Message::UrlChanged(url) => {
                    self.es_url = url;
                    Action::None
                },
                Message::AuthChoiceSelected(auth_choice) => {
                    self.auth_choice_type = Some(auth_choice);
                    Action::None
                },
                Message::BasicAuthUsernameChanged(username) => {
                    self.basic_auth_data.username = username;
                    Action::None
                },
                Message::BasicAuthPasswordChanged(password) => {
                    if password.is_empty() {
                        self.basic_auth_data.password = None;
                    } else {
                        self.basic_auth_data.password = Some(password);
                    }
                    Action::None
                },
                Message::AwsAuthRegionChanged(region) => {
                    self.aws_sigv4_data.region = region;
                    Action::None
                },
                Message::AwsAuthProfileChanged(profile) => {
                    if profile.is_empty() {
                        self.aws_sigv4_data.profile = None;
                    } else {
                        self.aws_sigv4_data.profile = Some(profile);
                    }
                    Action::None
                },
                Message::CertRemoved => {
                    self.selected_cert = Ok(None);
                    Action::None
                },
                Message::CertSelectionClicked => {
                    if self.cert_selection_open {
                        Action::None
                    } else {
                        self.cert_selection_open = true;
                        Action::Run(Self::open_cert_selection())
                    }
                },
                Message::CertObtained(res) => {
                    self.cert_selection_open = false;
                    self.selected_cert = res;
                    Action::None
                },
                Message::TestConnectionButtonPressed => {
                    match self.get_client() {
                        Ok(client) => {
                            self.test_connection_button_state = TestConnectionButtonState::Waiting;
                            Action::Run(Self::test_connection(client))
                        },
                        Err(reason) => {
                            self.test_connection_button_state = TestConnectionButtonState::Result(Err(reason));
                            Action::None
                        },
                    }
                },
                Message::TestConnectionButtonResultReturned(res) => {
                    self.test_connection_button_state = TestConnectionButtonState::Result(res);
                    Action::None
                },
            }
        }

        #[must_use]
        pub fn view(&self) -> iced::Element<'_, Message> {
            column![
                iced::widget::text("Elasticsearch Connection"),
                iced::widget::text("Elasticsearch URL"),
                iced::widget::text_input("https://elasticsearch.example.com:9200", &self.es_url)
                    .on_input(Message::UrlChanged),
                self.es_url.is_empty().then_some(
                    iced::widget::text("Elasticsearch URL is required")
                ),
                (!self.es_url.is_empty() && !util::valid_url(&self.es_url)).then_some(
                    iced::widget::text("Invalid URL format")
                ),
                iced::widget::text("Authentication Method"),
                row![
                    iced::widget::radio("Basic Auth", AuthChoice::Basic, self.auth_choice_type, Message::AuthChoiceSelected),
                    iced::widget::radio("AWS SigV4", AuthChoice::AWSSigV4, self.auth_choice_type, Message::AuthChoiceSelected),
                    iced::widget::radio("None", AuthChoice::None, self.auth_choice_type, Message::AuthChoiceSelected)
                ],
                self.auth_choice_type.map(|choice| {
                    match choice {
                        AuthChoice::Basic => Some(column![
                            iced::widget::text("Username"),
                            iced::widget::text_input("username", &self.basic_auth_data.username)
                                .on_input(Message::BasicAuthUsernameChanged),
                            iced::widget::text("Password (Optional)"),
                            iced::widget::text_input(
                                    "password", 
                                    self.basic_auth_data.password.as_ref().map(String::as_str).unwrap_or("")
                                )
                                .on_input(Message::BasicAuthPasswordChanged),
                        ]),
                        AuthChoice::AWSSigV4 => Some(column![
                            iced::widget::text("AWS Region"),
                            iced::widget::text_input("us-east-1", &self.aws_sigv4_data.region)
                                .on_input(Message::AwsAuthRegionChanged),
                            iced::widget::text("AWS Profile"),
                            iced::widget::text_input(
                                    "default", 
                                    self.aws_sigv4_data.profile.as_ref().map(String::as_str).unwrap_or("")
                                )
                                .on_input(Message::AwsAuthProfileChanged)
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
                                .on_press(Message::CertRemoved)
                        ]
                    })
                    .unwrap_or(
                        row![iced::widget::button("Upload Certificate (.pem or .der)")
                                .on_press_maybe((!self.cert_selection_open).then(|| Message::CertSelectionClicked))]
                    ),
                self.selected_cert.as_ref().err().map(|reason| {
                    iced::widget::text(format!("Failed to get certificate\n {}", reason))
                }),
                self.test_connection_button(),
            ].into()
        }

        fn test_connection_button(&self) -> iced::widget::Column<'_, Message> {
            match &self.test_connection_button_state {
                TestConnectionButtonState::NotClicked => column![
                    iced::widget::button("Test connection")
                        .on_press(Message::TestConnectionButtonPressed)
                ],
                TestConnectionButtonState::Waiting => column![
                        iced::widget::button("Test connection\t waiting...")
                ],
                TestConnectionButtonState::Result(res) => column![
                    iced::widget::button("Test connection")
                        .on_press(Message::TestConnectionButtonPressed),
                    res.as_ref().map(|_| {
                        iced::widget::text("Connection successful")
                    }).unwrap_or_else(|reason| {
                        iced::widget::text(format!("Connection failed\n{}", reason))
                    })
                ],
            }
        }

        pub fn get_client(&self) -> Result<es::ElasticsearchClient, String> {
            let mut client = es::ElasticsearchClient::new(self.es_url.clone())
                .map_err(|err| err.to_string())?;

            if let Some((_, cert)) = self.selected_cert.as_ref()? {
                client.use_custom_certificate(cert.clone()).map_err(|err| err.to_string())?
            }

            let auth = self.auth_choice_type.map(|auth_type| {
                match auth_type {
                    AuthChoice::Basic => Some(es::Auth::BASIC(self.basic_auth_data.clone())),
                    AuthChoice::AWSSigV4 => Some(es::Auth::AWS(self.aws_sigv4_data.clone())),
                    AuthChoice::None => None,
                }
            }).flatten(); 

            if let Some(auth_choice) = auth {
                client.use_auth(auth_choice);
            }

            Ok(client)
        }

        fn test_connection(client: es::ElasticsearchClient) -> iced::Task<Message> {
            iced::Task::future(async move {
                let res = client.test_connection().await.map_err(|err| err.to_string());

                Message::TestConnectionButtonResultReturned(res)
            })
        }

        fn open_cert_selection() -> iced::Task<Message> {
            // iced::window::oldest()
            //     .and_then(|id| iced::window::run(id, MyApp::get_cert_from_file))
            //     .then(iced::Task::future)
            //     .map(Message::ObtainCert)
            iced::Task::future(
                rfd::AsyncFileDialog::new()
                    .add_filter("CA", &["pem", "der"])
                    .set_title("Select CA Cert")
                    .pick_file()
            ).then(|handle| {
                match handle {
                    Some(handle) => 
                        iced::Task::perform(async move {
                            let path: std::path::PathBuf = handle.into();
                            let contents = tokio::fs::read_to_string(&path).await
                                .map_err(|err| format!("Unable to read {}, error kind: {}", path.to_string_lossy(), err.kind()))?;

                            let certificate = if util::path_has_extension(&path,"pem") {
                                reqwest::Certificate::from_pem(contents.as_bytes())
                                    .map_err(|err| format!("Unable to interpret {} as pem, error kind: {}", path.to_string_lossy(), err.to_string()))
                            }
                            else if util::path_has_extension(&path,"der") {
                                reqwest::Certificate::from_der(contents.as_bytes())
                                    .map_err(|err| format!("Unable to interpret {} as der, error kind: {}", path.to_string_lossy(), err.to_string()))
                            } else {
                                Err( format!("{} is unsupported file type", path.to_string_lossy()))
                            }?;

                            Ok(Some((path, certificate)))
                        }, Message::CertObtained),
                    None => iced::Task::done(Message::CertObtained(Ok(None))),
                }
            })
        }
    }
}

pub mod search {
    use std::collections::HashMap;

    use elastic_ermine::es;
    use iced::widget::{column, row};

    #[derive(Debug, Clone)]
    pub enum Message {
        SearchTypeChanged(SearchType),
        FilterRefreshPressed,
        FilterRefreshResultsReturned(
            Result<
                (Vec<String>, Vec<String>), 
                (String, Option<Vec<String>>, Option<Vec<String>>)
            > // TODO: think if this should have an explicit type
        ),
        SelectedFiltersUpdated(FiltersUpdate),
        SearchPressed,
        SearchResultsReturned(Result<es::OperationSearchResult, String>),
        GenericSearchBodyEditorActionPerformed(iced::widget::text_editor::Action)
    }

    pub enum Action {
        None,
        TryClientInvoke(Context),
    }

    pub enum Context {
        AllIndiciesAndAliases,
        GenericSearch{
            body: String,
            indicies: Vec<String>,
            aliases: Vec<String>,
        },
    }

    #[derive(Debug, Default)]
    pub struct View {
        search_type: SearchType,
        refresh_filter_button_state: RefreshFilterButtonState,
        refresh_filter_errors: Option<String>,
        known_indicies_selected: std::collections::HashMap<String, bool>,
        known_aliases_selected: std::collections::HashMap<String, bool>,

        generic_search_search_button_state: GenericSearchSearchButtonState,
        generic_search_display_content: GenericSearchDisplaySectionValue,
        generic_search_body_content: iced::widget::text_editor::Content,
    }

    #[derive(Debug, Default)]
    enum RefreshFilterButtonState {
        #[default]
        Ready,
        Waiting,
    }

    #[derive(Debug, Default, Clone)]
    pub enum SearchType {
        StringSearch,
        #[default]
        GenericSearch
    }

    #[derive(Debug, Clone)]
    pub enum FiltersUpdate {
        AddIndex(String),
        RemoveIndex(String),
        AddAlias(String),
        RemoveAlias(String),
    }

    #[derive(Debug, Default)]
    enum GenericSearchSearchButtonState {
        #[default]
        Ready,
        Waiting,
    }

    #[derive(Debug, Default)]
    enum GenericSearchDisplaySectionValue {
        #[default]
        Default,
        Error(String),
        Result(es::OperationSearchResult)
    }

    impl View {
        #[must_use]
        pub fn update(&mut self, message: Message) -> Action {
            match message {
                Message::SearchTypeChanged(search_type) => {
                    self.search_type = search_type;
                    Action::None
                },
                Message::FilterRefreshPressed => {
                    self.refresh_filter_button_state = RefreshFilterButtonState::Waiting;
                    Action::TryClientInvoke(Context::AllIndiciesAndAliases)
                },
                Message::FilterRefreshResultsReturned(res) => {
                    self.refresh_filter_button_state = RefreshFilterButtonState::Ready;

                    // TODO: existing selected filters that still exist on refresh should not be unselected
                    match res {
                        Ok((indicies, aliases)) => {
                            self.known_aliases_selected = aliases.into_iter().map(|alias| (alias, false)).collect();
                            self.known_indicies_selected = indicies.into_iter().map(|index| (index, false)).collect();
                            self.refresh_filter_errors = None;
                        },
                        Err((err, obtained_indicies, obtained_aliases)) => {
                            // TODO: rethink if these partially returned results should be option or just plain list.
                            // None could just be an empty list.
                            
                            self.known_indicies_selected = match obtained_indicies {
                                Some(indicies) => 
                                    indicies.into_iter().map(|index| (index, false)).collect(),
                                None => HashMap::new(),
                            };

                            self.known_aliases_selected = match obtained_aliases {
                                Some(aliases) => aliases.into_iter().map(|alias| (alias, false)).collect(),
                                None => HashMap::new(),
                            };
                            self.refresh_filter_errors = Some(format!("Failed to refresh filters: {}", err)); 
                        },
                    };
                    Action::None
                },
                Message::SelectedFiltersUpdated(filters_update) => {
                    match filters_update {
                        FiltersUpdate::AddIndex(idx) => {
                            if self.known_indicies_selected.contains_key(&idx) {
                                self.known_indicies_selected.insert(idx, true);
                            }
                        },
                        FiltersUpdate::RemoveIndex(idx) => {
                            if self.known_indicies_selected.contains_key(&idx) {
                                self.known_indicies_selected.insert(idx, false);
                            }
                        },
                        FiltersUpdate::AddAlias(idx) => {
                            if self.known_aliases_selected.contains_key(&idx) {
                                self.known_aliases_selected.insert(idx, true);
                            }
                        },
                        FiltersUpdate::RemoveAlias(idx) => {
                            if self.known_aliases_selected.contains_key(&idx) {
                                self.known_aliases_selected.insert(idx, false);
                            }
                        },
                    }
                    Action::None
                },
                Message::SearchPressed => {
                    self.generic_search_search_button_state = GenericSearchSearchButtonState::Waiting;
                    Action::TryClientInvoke(Context::GenericSearch { 
                        body: self.generic_search_body_content.text(), 
                        indicies: self.known_indicies_selected.iter()
                            .filter_map(|(index, selected)| selected.then_some(index.to_owned()))
                            .collect(), 
                        aliases: self.known_aliases_selected.iter()
                            .filter_map(|(alias, selected)| selected.then_some(alias.to_owned()))
                            .collect() 
                    })
                },
                Message::SearchResultsReturned(operation_search_result) => {
                    self.generic_search_search_button_state = GenericSearchSearchButtonState::Ready;
                    match operation_search_result {
                        Ok(res) => {
                            self.generic_search_display_content = GenericSearchDisplaySectionValue::Result(res);
                        },
                        Err(err) => {
                            self.generic_search_display_content = GenericSearchDisplaySectionValue::Error(format!("Failed to search: {}", err));
                        },
                    }
                    Action::None
                },
                Message::GenericSearchBodyEditorActionPerformed(action) => {
                    self.generic_search_body_content.perform(action);
                    Action::None
                },
            }
        }

        #[must_use]
        pub fn view(&self) -> iced::Element<'_, Message> {
            column![
                self.choose_search_type_section(),
                match self.search_type {
                    SearchType::StringSearch => self.search_string_search(),
                    SearchType::GenericSearch => self.generic_search_view(),
                }
                .align_x(iced::alignment::Horizontal::Center)
                .width(iced::Fill)
                .height(iced::Fill),
            ].spacing(10)
            .into()
        }

        fn choose_search_type_section(&self) -> iced::widget::Row<'_, Message> {
            row![
                iced::widget::button("String Search")
                    .on_press(Message::SearchTypeChanged(SearchType::StringSearch)),
                iced::widget::button("Generic Search")
                    .on_press(Message::SearchTypeChanged(SearchType::GenericSearch)),
            ]
        }

        fn search_filters(&self) -> iced::widget::Column<'_, Message> {
            let filters = column![
                row![
                    iced::widget::text("Filters"),
                    match self.refresh_filter_button_state {
                        RefreshFilterButtonState::Ready => iced::widget::button("Refresh")
                            .on_press(Message::FilterRefreshPressed),
                        RefreshFilterButtonState::Waiting => iced::widget::button("Refreshing..."),
                    },
                ],
                self.refresh_filter_errors.as_ref()
                    .map(|err| iced::widget::text(err)),
                iced::widget::text("Indicies"),
            ];

            let filters = filters.extend(
                self.known_indicies_selected
                    .iter()
                    .map(|(index, selected)|
                        iced::widget::checkbox(*selected)
                            .label(index)
                            .on_toggle(|toggled| {
                                if toggled {
                                    Message::SelectedFiltersUpdated(FiltersUpdate::AddIndex(index.to_owned()))
                                } else {
                                    Message::SelectedFiltersUpdated(FiltersUpdate::RemoveIndex(index.to_owned()))
                                }
                            })
                            .into()));

            let filters = filters.push(iced::widget::text("Aliases"));

            filters.extend(
                self.known_aliases_selected
                    .iter()
                    .map(|(alias, selected)|
                        iced::widget::checkbox(*selected)
                            .label(alias)
                            .on_toggle(|toggled| {
                                if toggled {
                                    Message::SelectedFiltersUpdated(FiltersUpdate::AddAlias(alias.to_owned()))
                                } else {
                                    Message::SelectedFiltersUpdated(FiltersUpdate::RemoveAlias(alias.to_owned()))
                                }
                            })
                            .into()))
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
                GenericSearchSearchButtonState::Ready => Some(Message::SearchPressed),
                GenericSearchSearchButtonState::Waiting => None,
            };
            iced::widget::button("Search")
                .on_press_maybe(produced_message)
                .width(iced::Shrink)
                .height(iced::Shrink)
        }

        pub fn try_invoke_with_client(
            client_res: Result<es::ElasticsearchClient, String>,
            context: Context
        ) -> iced::Task<Message> {
            match context {
                Context::AllIndiciesAndAliases => iced::Task::perform(
                        Self::get_all_indicies_and_aliases(client_res),
                        Message::FilterRefreshResultsReturned
                ),
                Context::GenericSearch { body, indicies, aliases } => iced::Task::perform(
                    Self::generic_search(client_res, body, indicies, aliases),
                    Message::SearchResultsReturned
                ),
            }
        }

        async fn get_all_indicies(client: &es::ElasticsearchClient) -> Result<Vec<String>, String> {
            client.get_indicies().await
                .map(|indicies| 
                        indicies.into_iter()
                            .map(|idx| idx.name)
                            .collect::<Vec<String>>())
                .map_err(|err| err.to_string())
        }

        async fn get_all_aliases(client: &es::ElasticsearchClient) -> Result<Vec<String>, String> {
            client.get_aliases().await
                .map(|aliases| 
                        aliases.into_iter()
                            .map(|alias| alias.name)
                            .collect::<Vec<String>>())
                .map_err(|err| err.to_string())
        }

        async fn get_all_indicies_and_aliases(
            client_res: Result<es::ElasticsearchClient, String>
        ) -> Result<
                (Vec<String>, Vec<String>), 
                (String, Option<Vec<String>>, Option<Vec<String>>)>{
            let client = client_res.map_err(|err| (err, None, None))?;
            
            let (indicies_res, aliases_res) = iced::futures::join!(
                    Self::get_all_indicies(&client),
                    Self::get_all_aliases(&client)
            );
            
            match (indicies_res, aliases_res) {
                (Ok(indices), Ok(aliases)) => 
                    Ok((indices, aliases)),
                (Ok(indices), Err(alias_err)) => 
                    Err((format!("Failed to get aliases: {}", alias_err), Some(indices), None)),
                (Err(index_err), Ok(aliases)) => 
                    Err((format!("Failed to get indicies: {}", index_err), None, Some(aliases))),
                (Err(index_err), Err(alias_err)) => 
                    Err((format!("Failed to get indicies: {}\n Failed to get aliases: {}", index_err, alias_err), None, None)),
            }
        }

        async fn generic_search(
            client_res: Result<es::ElasticsearchClient, String>,
            body: String,
            mut indicies: Vec<String>,
            mut aliases: Vec<String>
        ) -> Result<es::OperationSearchResult, String> {
            let client = client_res?;

            let search_body = (!body.is_empty()).then(|| {
                serde_json::from_str::<serde_json::Value>(&body)
            })
            .transpose()
            .map_err(|err| err.to_string())?;

            indicies.append(&mut aliases);

            client.search(&indicies, search_body.as_ref()).await
                .map_err(|err| err.to_string())
        }
    }
}