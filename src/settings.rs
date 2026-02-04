use crate::{assets, widget, es, util};
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
    test_connection_result: Option<Result<(), String>>,
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
            test_connection_result: None,
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
    Ready,
    Waiting,
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
                        self.test_connection_button_state = TestConnectionButtonState::Ready;
                        self.test_connection_result = Some(Err(reason));
                        Action::None
                    },
                }
            },
            Message::TestConnectionButtonResultReturned(res) => {
                self.test_connection_button_state = TestConnectionButtonState::Ready;
                self.test_connection_result = Some(res);
                Action::None
            },
        }
    }

    #[must_use]
    pub fn view(&self) -> iced::Element<'_, Message> {
        iced::widget::container(
            column![
                iced::widget::text("Cluster Connection"),
                self.general_info_section(),
                self.authentication_section(),
                self.advanced_section(),
                self.test_connection_section(),
            ]
            .spacing(20)
        )
        .padding([20, 40])
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::Top)
        .into()
        
    }

    fn general_info_section(&self) -> iced::widget::Container<'_, Message> {
        Self::section(
            iced::widget::text("General").into(), 
            column![
                        iced::widget::text("Elasticsearch URL"),
                        iced::widget::text_input("https://elasticsearch.example.com:9200", &self.es_url)
                            .on_input(Message::UrlChanged),
                        self.es_url.is_empty().then_some(
                            iced::widget::text("Elasticsearch URL is required")
                        ),
                        (!self.es_url.is_empty() && !util::valid_url(&self.es_url)).then_some(
                            iced::widget::text("Invalid URL format")
                        )
                ].into()
        )
    }

    fn authentication_section(&self) -> iced::widget::Container<'_, Message> {
        Self::section(
            iced::widget::text("Authentication").into(), 
            iced::widget::column![
                row![
                    widget::RadioArea::new("Basic Auth", AuthChoice::Basic, self.auth_choice_type, Message::AuthChoiceSelected)
                        .width(iced::FillPortion(1)),
                    widget::RadioArea::new("AWS SigV4", AuthChoice::AWSSigV4, self.auth_choice_type, Message::AuthChoiceSelected)
                        .width(iced::FillPortion(1)),
                    widget::RadioArea::new("No Auth", AuthChoice::None, self.auth_choice_type, Message::AuthChoiceSelected)
                        .width(iced::FillPortion(1)),
                ].width(iced::Fill).spacing(10),
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
                }).flatten()
            ].into()
        )
    }

    fn advanced_section(&self) -> iced::widget::Container<'_, Message> {
        Self::section(
            iced::widget::text("Advanced Settings").into(), 
            iced::widget::column![
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
                })
            ].into())
    }

    fn section<'a>(header: iced::Element<'a, Message>, body: iced::Element<'a, Message>) -> iced::widget::Container<'a, Message> {
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
        ).style(iced::widget::container::bordered_box)
    }

    
    fn test_connection_section(&self) -> iced::widget::Column<'_, Message> {
        iced::widget::column![
            match &self.test_connection_button_state {
                TestConnectionButtonState::Ready => 
                    iced::widget::button("Test connection")
                        .on_press(Message::TestConnectionButtonPressed),
                TestConnectionButtonState::Waiting =>
                        iced::widget::button(
                            iced::widget::row![
                                assets::loading_icon().width(iced::Shrink),
                                "Test connection"
                            ])
            },
            // TODO: refactor using `section`` function. Also modify `section` function to accept more params
            self.test_connection_result.as_ref().map(|res| {
                match res {
                    Ok(_) => iced::widget::container(
                        iced::widget::column![
                            iced::widget::text("Connection Successful"),
                            iced::widget::text("Successfully pinged instance")
                        ]
                        .spacing(5)
                    ).style(|t| {
                        let success = iced::widget::container::success(t);
                        let border = success.border.rounded(5.0);
                        success.border(border)
                    }),
                    Err(msg) => iced::widget::container(
                        iced::widget::column![
                            iced::widget::text("Connection Failed"),
                            iced::widget::text(msg)
                        ]
                        .spacing(5)
                    ).style(|t| {
                        let danger = iced::widget::container::danger(t);
                        let border = danger.border.rounded(5.0);
                        danger.border(border)
                    }),
                }
                .width(iced::Fill)
                .padding(10)
            })
        ]
        .spacing(15)
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
