use std::collections::HashMap;

use crate::{assets, es, widget};
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
    GenericSearchBodyEditorActionPerformed(iced::widget::text_editor::Action),
    QueryStringUpdated(String),
    ResultsAccordianClicked(usize)
}

pub enum Action {
    None,
    TryClientInvoke(Context),
}

pub enum Context {
    AllIndiciesAndAliases,
    GenericSearch{
        body: String,
        indices: Vec<String>,
        aliases: Vec<String>,
    },
    QueryStringSearch {
        query_string: String,
        indices: Vec<String>,
        aliases: Vec<String>,
    }
}

#[derive(Debug, Default)]
pub struct View {
    search_type: SearchType,
    refresh_filter_button_state: RefreshFilterButtonState,
    refresh_filter_errors: Option<String>,
    known_indicies_selected: std::collections::HashMap<String, bool>,
    known_aliases_selected: std::collections::HashMap<String, bool>,

    query_string: String,

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
    #[default]
    StringSearch,
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
    Result{
        res: es::OperationSearchResult,
        expanded_hits: Vec<bool>
    }
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

                let indices = self.known_indicies_selected.iter()
                        .filter_map(|(index, selected)| selected.then_some(index.to_owned()))
                        .collect::<Vec<String>>();

                let aliases = self.known_aliases_selected.iter()
                        .filter_map(|(alias, selected)| selected.then_some(alias.to_owned()))
                        .collect::<Vec<String>>();

                Action::TryClientInvoke(
                    match self.search_type {
                        SearchType::StringSearch => Context::QueryStringSearch { query_string: self.query_string.clone(), indices, aliases },
                        SearchType::GenericSearch => Context::GenericSearch { body: self.generic_search_body_content.text(), indices, aliases},
                    }
                )
            },
            Message::SearchResultsReturned(operation_search_result) => {
                self.generic_search_search_button_state = GenericSearchSearchButtonState::Ready;
                match operation_search_result {
                    Ok(res) => {
                        let num_hits = res.hits.hits.len();
                        self.generic_search_display_content = GenericSearchDisplaySectionValue::Result{res, expanded_hits: vec![false; num_hits]};
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
            Message::QueryStringUpdated(query_string) => {
                self.query_string = query_string;
                Action::None
            },
            Message::ResultsAccordianClicked(idx) => {
                if let GenericSearchDisplaySectionValue::Result { res: _res, expanded_hits} = &mut self.generic_search_display_content {
                    if expanded_hits.len() > idx {
                        let is_expanded = expanded_hits[idx];
                        expanded_hits[idx] = !is_expanded;
                    }
                }

                Action::None
            },
        }
    }

    #[must_use]
    pub fn view(&self) -> iced::Element<'_, Message> {
        row![
            self.search_filters()
                .align_x(iced::alignment::Horizontal::Left)
                .width(iced::FillPortion(1))
                .height(iced::Shrink),
            match self.search_type {
                SearchType::StringSearch => iced::widget::column![
                    widget::section(
                        iced::widget::column![
                            self.choose_search_type_section(),
                            self.query_string_search_view(),
                        ]
                        .spacing(10)
                    ),
                    self.generic_search_result_view()
                        .width(iced::Fill)
                        .height(iced::Fill)
                ],
                SearchType::GenericSearch => iced::widget::column![
                    widget::section(
                        iced::widget::column![
                            self.choose_search_type_section(),
                            self.generic_search_view(),
                        ]
                        .spacing(10)
                    ).height(iced::FillPortion(1)),
                    self.generic_search_result_view()
                        .width(iced::Fill)
                        .height(iced::FillPortion(1))
                ],
            }
            .spacing(10)
            .align_x(iced::alignment::Horizontal::Center)
            .width(iced::FillPortion(4))
            .height(iced::Fill),
        ]
        .spacing(10)
        .into()
    }

    fn choose_search_type_section(&self) -> iced::widget::Row<'_, Message> {
        row![
            iced::widget::button("Query String")
                .on_press(Message::SearchTypeChanged(SearchType::StringSearch)),
            iced::widget::button("Search Payload")
                .on_press(Message::SearchTypeChanged(SearchType::GenericSearch)),
            iced::widget::space::horizontal(),
            self.generic_search_search_button()
        ]
    }

    fn search_filters(&self) -> iced::widget::Container<'_, Message> {
        let filters = column![
            row![
                iced::widget::text("Filters"),
                iced::widget::space::horizontal(),
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

        let filters = filters.extend(
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
                        .into()));

        iced::widget::container(filters)
            .style(iced::widget::container::bordered_box)
            .padding(10)
    }

    fn query_string_search_view(&self) -> iced::widget::Container<'_, Message> {
        iced::widget::container(
            iced::widget::text_input("title:(quick OR brown)", &self.query_string)
                .on_input(Message::QueryStringUpdated)
        )
    }

    fn generic_search_view(&self) -> iced::widget::Container<'_, Message> {
        iced::widget::container(
            iced::widget::text_editor(&self.generic_search_body_content)
                .on_action(Message::GenericSearchBodyEditorActionPerformed)
                .height(iced::Fill)
        )
    }

    fn generic_search_result_view(&self) -> iced::widget::Container<'_, Message> {
        match &self.generic_search_display_content {
            GenericSearchDisplaySectionValue::Default => widget::section(
                iced::widget::text(
                    "Enter a query above to search your Elasticsearch cluster. Use the filters on the left to refine your results."
                )
                .align_x(iced::Center)
                .align_y(iced::Center)
            ),
            GenericSearchDisplaySectionValue::Error(err) => widget::section_with_header(
                iced::widget::text("ERROR"),
                iced_selection::text(
                    format!("Search failed: {}", err)
                )
                .align_x(iced::Center)
                .align_y(iced::Center)
            ),
            GenericSearchDisplaySectionValue::Result{res, expanded_hits} => widget::section_with_header(
                iced::widget::row![
                    iced::widget::text(format!("Results")).align_y(iced::Center),
                    iced::widget::space::horizontal(),
                    self.result_stats(res)
                ],
                iced::widget::scrollable(
                    column(
                    res.hits.hits.iter().zip(expanded_hits.iter()).enumerate().map(|(index, (item, expanded))|
                        self.hit_item(item, *expanded, index)
                            .width(iced::Fill)
                            .into()
                    ))
                )
                .width(iced::Fill)
                .height(iced::Fill)
            ),
        }
    }

    // TODO: rename as search button. Should also submit query based on the search mode
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

    // TODO: refactor widget::section function to capture this case too
    fn result_stats(&self, res: &es::OperationSearchResult) -> iced::widget::Container<'_, Message> {
        if res.timed_out {
            iced::widget::container(
                iced::widget::text(format!("Timed out | {} results | {} ms", res.hits.hits.len(), res.time_took_ms))
            )
            .style(|t| {
                let danger = iced::widget::container::danger(t);
                let border = danger.border.rounded(5.0);
                danger.border(border)
            })
        } else {
            iced::widget::container(
                iced::widget::text(format!("{} results | {} ms", res.hits.hits.len(), res.time_took_ms))
            )
            .style(|t| {
                let success = iced::widget::container::success(t);
                let border = success.border.rounded(5.0);
                success.border(border)
            })
        }
        .padding(5)
    }

    // TODO: consider allow the display of multiple fields based on selection
    fn hit_item<'a>(&'a self, item: &'a serde_json::Value, expanded: bool, index: usize) -> iced::widget::Container<'a, Message> {
        let expand_button = iced::widget::button(
            if expanded {
                assets::chevron_down()
            } else {
                assets::chevron_right()
            }
            .height(15)
            .width(15)
        )
        .width(iced::Shrink)
        .height(iced::Shrink)
        .on_press(Message::ResultsAccordianClicked(index));
        
        // TODO: make this formatted with key hightlighted, kibana for reference
        let header = iced::widget::row![
            expand_button,
            iced_selection::text(
                item.get("_id")
                    .map(|val| val.as_str())
                    .flatten()
                    .unwrap_or("_id field missing")
            )
            .align_y(iced::Center)
        ].spacing(10);

        if expanded {
            widget::section_with_header(
                header, 
                iced_selection::text(
                    serde_json::to_string_pretty(item)
                        .unwrap_or(format!("Failed to display {:?}", item)))
                )
        } else {
            widget::section(header)
        }
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
            Context::GenericSearch { body, indices, aliases } => iced::Task::perform(
                Self::generic_search(client_res, body, indices, aliases),
                Message::SearchResultsReturned
            ),
            Context::QueryStringSearch { query_string, indices, aliases } => iced::Task::perform(
                Self::query_string_search(client_res, query_string, indices, aliases), Message::SearchResultsReturned),
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
        mut indices: Vec<String>,
        mut aliases: Vec<String>
    ) -> Result<es::OperationSearchResult, String> {
        let client = client_res?;

        let search_body = (!body.is_empty()).then(|| {
            serde_json::from_str::<serde_json::Value>(&body)
        })
        .transpose()
        .map_err(|err| err.to_string())?;

        indices.append(&mut aliases);

        client.search(&indices, search_body.as_ref()).await
            .map_err(|err| err.to_string())
    }

    async fn query_string_search(
        client_res: Result<es::ElasticsearchClient, String>,
        query_string: String,
        mut indices: Vec<String>,
        mut aliases: Vec<String>
    ) -> Result<es::OperationSearchResult, String> {
        let client = client_res?;
        indices.append(&mut aliases);
        client.query_string(&indices, query_string).await
            .map_err(|err| err.to_string())
    }

}
