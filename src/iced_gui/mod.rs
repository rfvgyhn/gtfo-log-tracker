mod game_log_watcher;

use crate::game_data::StoryLog;
use crate::{get_logs, Options};
use iced::alignment::Horizontal;
use iced::widget::{
    checkbox, column, container, responsive, row, scrollable, text, text_input, Responsive, Text,
};
use iced::{
    alignment, executor, font, window, Alignment, Application, Command, Element, Font, Length,
    Renderer, Settings, Subscription, Theme,
};
use iced_aw::Spinner;
use iced_table::table;
use std::collections::HashSet;
use std::fmt::Write;
use std::path::PathBuf;

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
    DataLoaded(Vec<StoryLog>, HashSet<u32>, PathBuf),
    SyncHeader(scrollable::AbsoluteOffset),
    TableResizing(usize, f32),
    TableResized,
    ToggleHideRead(bool),
    FilterChanged(String),
    FontLoaded(Result<(), font::Error>),
    Error(String),
    NewLogRead(u32),
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
                font::load(include_bytes!("../../fonts/icons.ttf").as_slice())
                    .map(Message::FontLoaded),
                Command::perform(
                    get_logs(options.gtfo_path.clone(), options.only_parse_from_logs),
                    |r| {
                        r.map(|(all_logs, read_log_ids)| {
                            Message::DataLoaded(all_logs, read_log_ids, options.gtfo_path)
                        })
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
            Message::DataLoaded(all_logs, read_log_ids, gtfo_path) => {
                let view = MainView::new(all_logs, read_log_ids, gtfo_path);
                *self = GtfoLogTracker::Loaded(view);
            }
            Message::NewLogRead(log_id) => {
                if let GtfoLogTracker::Loaded(view) = self {
                    if !view.read_log_ids.contains(&log_id) {
                        view.read_log_ids.insert(log_id);
                    }
                }
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
            Message::FilterChanged(text) => {
                if let GtfoLogTracker::Loaded(view) = self {
                    view.filter = text;
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
            GtfoLogTracker::Loaded(view) => layout(column![header(view), log_table(view)]),
            GtfoLogTracker::Error(e) => layout(text(e)),
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        if let GtfoLogTracker::Loaded(view) = self {
            game_log_watcher::watch(view.gtfo_path.clone(), view.all_logs.clone())
                .map(Message::NewLogRead)
        } else {
            Subscription::none()
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

fn header(view: &MainView) -> Element<'_, Message, Renderer<Theme>> {
    row![
        container(text(format!(
            "{}/{} Read",
            view.read_log_ids.len(),
            view.all_logs.len()
        )))
        .align_x(Horizontal::Left)
        .width(Length::FillPortion(1)),
        container(
            text_input("Filter", view.filter.as_str())
                .on_input(Message::FilterChanged)
                .padding(10)
        )
        .width(Length::FillPortion(2)),
        container(checkbox(
            "Hide Read",
            view.hide_read,
            Message::ToggleHideRead
        ))
        .width(Length::FillPortion(1))
        .align_x(Horizontal::Right)
    ]
    .padding(5)
    .spacing(10)
    .align_items(Alignment::Center)
    .into()
}

fn log_table(view: &MainView) -> Responsive<'_, Message, Renderer<Theme>> {
    responsive(|size| {
        let filtered_rows: Vec<Row> = view
            .all_logs
            .iter()
            .filter_map(
                |r| match (view.hide_read, view.read_log_ids.contains(&r.id)) {
                    (true, true) => None,
                    _ => Some(map_log_to_rows(r, &view.read_log_ids)),
                },
            )
            .flatten()
            .filter(|r| {
                if view.filter.is_empty() {
                    true
                } else {
                    let f = view.filter.to_ascii_lowercase();
                    r.level.to_ascii_lowercase().contains(&f)
                        || r.name.to_ascii_lowercase().contains(&f)
                        || r.zone.to_ascii_lowercase().contains(&f)
                        || r.id.to_string().contains(&f)
                }
            })
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
    })
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

fn map_log_to_rows<'a>(
    log: &'a StoryLog,
    read_log_ids: &'a HashSet<u32>,
) -> impl Iterator<Item = Row> + 'a {
    log.locations.iter().map(|loc| Row {
        level: format!("R{}{}", loc.rundown, loc.level),
        name: loc.name.to_string(),
        id: log.id,
        read: read_log_ids.contains(&log.id),
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
    all_logs: Vec<StoryLog>,
    read_log_ids: HashSet<u32>,
    hide_read: bool,
    filter: String,
    log_table: Table,
    gtfo_path: PathBuf,
}

impl MainView {
    fn new(all_logs: Vec<StoryLog>, read_log_ids: HashSet<u32>, gtfo_path: PathBuf) -> Self {
        Self {
            all_logs,
            read_log_ids,
            gtfo_path,
            hide_read: false,
            filter: "".to_string(),
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
        }
    }
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