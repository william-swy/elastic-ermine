use elastic_ermine::{es,util};

use iced::widget::{column, row};

fn main() -> iced::Result {
    iced::application(MyApp::new, MyApp::update, MyApp::view)
        .title("Elastic Ermine")
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    ConnectionConfigVisibility(bool),
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
    connection_config_visible: bool,
    cert_selection_open: bool,

    auth_choice_type: Option<AuthChoice>,
    auth_choice_basic: es::BasicAuth,
    auth_choice_aws: es::AwsSigv4,

    es_url: (String, bool),
    selected_cert: Result<Option<(std::path::PathBuf, reqwest::Certificate)>, MyAppError>,

    test_connection_button_state: TestConnectionButtonState,
}

#[derive(Debug)]
enum TestConnectionButtonState {
    NotClicked,
    Waiting,
    Result(Result<(), MyAppError>)
}

impl Default for MyApp {
    fn default() -> Self {
        Self { 
            connection_config_visible: false, 
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
        }
    }
}

const TOP_PADDING: f32 = 32.0;
const BOT_PADDING: f32 = 32.0;
const SIDE_PADDING: f32 = 64.0;

impl MyApp {
    fn new() -> (Self, iced::Task<Message>) {
        (
            MyApp::default(),
            iced::Task::none()
        )
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::ConnectionConfigVisibility(is_visible) => {
                self.connection_config_visible = is_visible;
                return iced::Task::none();
            }
            Message::AuthSelected(auth_choice) => {
                self.auth_choice_type = Some(auth_choice);
                return iced::Task::none();
            }
            Message::UrlChanged(url) => {
                let valid = util::valid_url(&url);
                self.es_url = (url, valid);
                return iced::Task::none();
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
                return iced::Task::none();
            }
            Message::RemoveCert => {
                self.selected_cert = Ok(None);
                return iced::Task::none();
            },
            Message::BasicAuthUsernameChanged(username) => {
                self.auth_choice_basic.username = username;
                return iced::Task::none();
            },
            Message::BasicAuthPasswordChanged(password) => {
                if password.is_empty() {
                    self.auth_choice_basic.password = None;
                } else {
                    self.auth_choice_basic.password = Some(password);
                }
                return iced::Task::none();
            },
            Message::TestConnectionButtonPressed => {
                self.test_connection_button_state = TestConnectionButtonState::Waiting;
                return iced::Task::perform(
                    MyApp::test_connection(
                        self.es_url.0.clone(), 
                        self.selected_cert.clone(), 
                        self.get_es_auth()), 
                    Message::TestConnectionButtonResultReturned);
            },
            Message::TestConnectionButtonResultReturned(res) => {
                self.test_connection_button_state = TestConnectionButtonState::Result(res);
                return iced::Task::none();
            },
            Message::AWSAuthRegionChanged(new_region) => {
                self.auth_choice_aws.region = new_region;
                return iced::Task::none();
            },
            Message::AWSAuthProfileChanged(profile) => {
                if profile.is_empty() {
                    self.auth_choice_aws.profile = None;
                } else {
                    self.auth_choice_aws.profile = Some(profile);
                }
                return iced::Task::none();
            },
        }
    }

    fn view(&self) -> iced::Element<'_, Message> {
        iced::widget::container(
            iced::widget::column![
                iced::widget::container(
                    iced::widget::column![
                        iced::widget::row![
                            iced::widget::column![
                                iced::widget::text("Elastic Ermine")
                                .font(iced::Font { weight: iced::font::Weight::Bold, ..iced::Font::default()})
                                .size(30),
                                iced::widget::text("Search your data with Elasticsearch")
                                .font(iced::Font { weight: iced::font::Weight::Light, ..iced::Font::default()})
                                .size(14)
                            ],
                            iced::widget::space::horizontal(),
                            self.connection_button()
                        ].align_y(iced::alignment::Vertical::Center),
                        self.connection_config()
                    ]
                ).align_y(iced::alignment::Vertical::Top),
                self.search_section(),
            ]
        )
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .width(iced::Fill)
        .height(iced::Fill)
        .padding(iced::Padding{top: TOP_PADDING, bottom: BOT_PADDING, left: SIDE_PADDING, right: SIDE_PADDING})
        .into()
    }


    fn connection_button(&self) -> iced::widget::Button<'_, Message> {
        let text = if self.connection_config_visible {
            "Hide Connection Config"
        } else {
            "Connection Config"
        };

        return iced::widget::button(iced::widget::text(text))
            .on_press(Message::ConnectionConfigVisibility(!self.connection_config_visible));
    }

    fn connection_config(&self) -> Option<iced::widget::Column<'_, Message>> {
        let content = self.connection_config_visible
        .then_some(column![
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
        );

        return content;
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


    fn search_section(&self) -> iced::widget::Container<'_, Message> {
        let content = column![iced::widget::text("Search")];
        return iced::widget::container(content)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Top)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill);
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
}