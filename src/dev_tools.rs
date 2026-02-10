use crate::{assets, es};
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
    HTTPOperationReturned(Result<es::OperationResult, String>), // Perhaps Value should be a reference for large results
    SendButtonPressed,
}

#[derive(Debug, Default)]
pub struct View {
    request_type: es::ElasticSearchMethodType,
    request_path: String,
    request_body: iced::widget::text_editor::Content,
    result: Option<Result<es::OperationResult, String>>,

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
                match self.send_button_state {
                    SendButtonState::Ready => 
                        iced::widget::button("Send")
                            .on_press(Message::SendButtonPressed),
                    SendButtonState::Waiting => 
                        iced::widget::button(
                            iced::widget::row![
                                assets::loading_icon().width(iced::Shrink),
                                "Send"
                            ]
                        ),
                }
                .width(iced::Shrink),
            ],
            row![
                iced::widget::text("REQUEST BODY (JSON)"),
                iced::widget::space::horizontal(),
                iced::widget::button("Format"),
                iced::widget::button("Clear")
            ]
            .spacing(5),
            iced::widget::text_editor(&self.request_body)
                .on_action(Message::RequestBodyEditPerformed)
                .height(iced::Length::Fill)
                .placeholder(r#"{"size":10000,"query":{"match_all":{}}}"#),
        ]
    }

    pub fn response(&self) -> iced::Element<'_, Message> {
        column![
            iced::widget::text("Results"),
            self.result.as_ref().map(|res| {
                match res {
                    Ok(val) => match val {
                        es::OperationResult::Json(json_val) => 
                            iced::widget::text(
                                serde_json::to_string_pretty(json_val)
                                    .unwrap_or_else(|err| format!("{} Failed to deserialize {:?}", err, val))),
                        es::OperationResult::Text(text_val) => iced::widget::text(text_val),
                    },
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
