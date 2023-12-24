use crate::game_data::StoryLog;
use crate::{get_logs, Options};
use iced::alignment::Horizontal;
use iced::widget::{checkbox, column, container, responsive, row, scrollable, text, Text};
use iced::{
    alignment, executor, font, window, Alignment, Application, Command, Element, Font, Length,
    Renderer, Settings, Theme,
};
use iced_aw::Spinner;
use iced_table::table;
use std::fmt::Write;

pub enum GtfoLogTracker {
    Loading,
    Loaded(MainView),
    Error(String),
}

impl GtfoLogTracker {
    pub fn settings(options: Options) -> Settings<Options> {
        Settings::<Options> {
            window: window::Settings {
                size: (500, 600),
                ..window::Settings::default()
            },
            flags: options,
            default_font: Default::default(),
            ..Settings::<Options>::default()
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    DataLoaded(Vec<StoryLog>),
    SyncHeader(scrollable::AbsoluteOffset),
    TableResizing(usize, f32),
    TableResized,
    ToggleHideRead(bool),
    FontLoaded(Result<(), font::Error>),
    Error(String),
}

impl Application for GtfoLogTracker {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = Options;

    fn new(options: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            GtfoLogTracker::Loading,
            Command::batch(vec![
                font::load(include_bytes!("../fonts/icons.ttf").as_slice())
                    .map(Message::FontLoaded),
                Command::perform(
                    get_logs(options.gtfo_path, options.only_parse_from_logs),
                    |r| {
                        r.map(Message::DataLoaded)
                            .unwrap_or_else(|e| Message::Error(e.to_string()))
                    },
                ),
            ]),
        )
    }

    fn title(&self) -> String {
        "GTFO Log Tracker".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::DataLoaded(logs) => {
                let view = MainView {
                    logs,
                    hide_read: false,
                    log_table: Table {
                        columns: vec![
                            TableColumn::new("", 40.0),
                            TableColumn::new("Level", 60.0),
                            TableColumn::new("Zone", 90.0),
                            TableColumn::new("Name", 130.0),
                            TableColumn::new("Id", 130.0),
                        ],
                        header: scrollable::Id::unique(),
                        body: scrollable::Id::unique(),
                    },
                };
                *self = GtfoLogTracker::Loaded(view);
            }
            Message::SyncHeader(offset) => {
                if let GtfoLogTracker::Loaded(view) = self {
                    return scrollable::scroll_to(view.log_table.header.clone(), offset);
                }
            }
            Message::TableResizing(index, offset) => {
                if let GtfoLogTracker::Loaded(view) = self {
                    if let Some(column) = view.log_table.columns.get_mut(index) {
                        column.resize_offset = Some(offset);
                    }
                }
            }
            Message::TableResized => {
                if let GtfoLogTracker::Loaded(view) = self {
                    view.log_table.columns.iter_mut().for_each(|column| {
                        if let Some(offset) = column.resize_offset.take() {
                            column.width += offset;
                        }
                    })
                }
            }
            Message::ToggleHideRead(hide) => {
                if let GtfoLogTracker::Loaded(view) = self {
                    view.hide_read = hide
                }
            }
            Message::Error(e) => *self = GtfoLogTracker::Error(e),
            Message::FontLoaded(Err(e)) => eprintln!("{:?}", e),
            Message::FontLoaded(Ok(())) => {}
        }

        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        match self {
            GtfoLogTracker::Loading => layout(
                column![text("Loading your progress"), Spinner::new()]
                    .align_items(Alignment::Center),
            ),
            GtfoLogTracker::Loaded(view) => {
                let header: Element<_> = row![
                    container(text(format!(
                        "{}/{} Read",
                        view.logs.iter().filter(|l| l.read).count(),
                        view.logs.len()
                    )))
                    .align_x(Horizontal::Left)
                    .width(Length::Fill),
                    container(checkbox(
                        "Hide Read",
                        view.hide_read,
                        Message::ToggleHideRead
                    ))
                    .width(Length::Fill)
                    .align_x(Horizontal::Right)
                ]
                .padding(5)
                .spacing(10)
                .into();
                let log_table = responsive(|size| {
                    let filtered_rows: Vec<Row> = view
                        .logs
                        .iter()
                        .filter_map(|r| match (view.hide_read, r.read) {
                            (true, true) => None,
                            _ => Some(map_log_to_rows(r)),
                        })
                        .flatten()
                        .collect();

                    table(
                        view.log_table.header.clone(),
                        view.log_table.body.clone(),
                        &view.log_table.columns,
                        &filtered_rows,
                        Message::SyncHeader,
                    )
                    .on_column_resize(Message::TableResizing, Message::TableResized)
                    .min_width(size.width)
                    .into()
                });

                let content = column![header, log_table];

                layout(container(content))
            }
            GtfoLogTracker::Error(e) => layout(text(e)),
        }
    }
}

fn layout<'a>(
    el: impl Into<Element<'a, Message, Renderer<Theme>>>,
) -> Element<'a, Message, Renderer<Theme>> {
    container(el)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_y()
        .center_x()
        .into()
}

impl<'a, 'b> table::Column<'a, 'b, Message, Renderer> for TableColumn {
    type Row = Row;

    fn header(&'b self, _: usize) -> Element<'a, Message, Renderer> {
        container(text(&self.title)).height(24).center_y().into()
    }

    fn cell(
        &'b self,
        col_index: usize,
        _: usize,
        row: &'b Self::Row,
    ) -> Element<'a, Message, Renderer> {
        match col_index {
            0 => icon_read(row.read),
            1 => text(&row.level),
            2 => text(&row.zone),
            3 => text(&row.name),
            4 => text(row.id),
            _ => text("?"),
        }
        .into()
    }

    fn width(&self) -> f32 {
        self.width
    }

    fn resize_offset(&self) -> Option<f32> {
        self.resize_offset
    }
}

fn map_log_to_rows(log: &StoryLog) -> impl Iterator<Item = Row> + '_ {
    log.locations.iter().map(|loc| Row {
        level: format!("R{}{}", loc.rundown, loc.level),
        name: loc.name.to_string(),
        id: log.id,
        read: log.read, //read_log_ids.contains(&log.id),
        zone: if loc.zones == vec![0] {
            "Outside".to_string()
        } else {
            comma_join(&loc.zones)
        },
    })
}

fn comma_join(nums: &[u16]) -> String {
    nums.iter()
        .enumerate()
        .fold(String::new(), |mut output, (i, num)| {
            if i == 0 {
                let _ = write!(output, "{num}");
            } else {
                let _ = write!(output, ", {num}");
            }
            output
        })
}

const ICONS: Font = Font::with_name("gtfo-tracker-icons");
fn icon(unicode: char) -> Text<'static> {
    text(unicode.to_string())
        .font(ICONS)
        .width(20)
        .horizontal_alignment(alignment::Horizontal::Center)
}

fn icon_read(read: bool) -> Text<'static> {
    if read {
        icon('\u{ea52}')
    } else {
        icon('\u{ea53}')
    }
}

pub struct MainView {
    logs: Vec<StoryLog>,
    hide_read: bool,
    log_table: Table,
}

struct Table {
    columns: Vec<TableColumn>,
    header: scrollable::Id,
    body: scrollable::Id,
}

struct TableColumn {
    title: String,
    width: f32,
    resize_offset: Option<f32>,
}
impl TableColumn {
    fn new(title: impl Into<String>, width: f32) -> Self {
        Self {
            title: title.into(),
            width,
            resize_offset: None,
        }
    }
}

struct Row {
    level: String,
    zone: String,
    name: String,
    id: u32,
    read: bool,
}
