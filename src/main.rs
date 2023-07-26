use std::sync::Arc;

use iced::{alignment, executor, Alignment, Application, Command, Element, Length, Theme};

use iced::font::{self, Font};
use iced::theme::Container;
use iced::widget::{button, column, container, row, text, Text};
use tokio::sync::Mutex;

mod cpuinfofs;

fn main() -> iced::Result {
    env_logger::builder().format_timestamp(None).init();
    BaseTop::run(iced::Settings::default())
}

struct BaseTop {
    cpuinfos: Vec<CpuMessage>,
}

#[allow(unused)]
#[derive(Clone, Debug)]
struct CpuMessage {
    name: String,
    processor: usize,
    mhz: String, // TODO: to i32
    cache_size: String,
    show_more: bool,
}

#[allow(unused)]
#[derive(Clone, Debug)]
enum Message {
    RequestUpdate,
    Nothing,
    CpuMessageStateChanged(Arc<Mutex<CpuMessage>>),
}

impl CpuMessage {
    fn view(&self) -> Element<Message> {
        let row: Element<Message> = row![
            text(self.name.as_str()),
            text(self.processor.to_string()),
            text(self.mhz.as_str()),
            text(self.cache_size.as_str()),
            button(edit_icon()).padding(10)
        ]
        .spacing(10)
        .align_items(Alignment::Center)
        .into();

        container(row)
            .center_x()
            .center_y()
            .width(Length::Fill)
            .style(Container::Box)
            .into()
    }
}
const ICONS: Font = Font::with_name("Iced-Todos-Icons");

fn icon(unicode: char) -> Text<'static> {
    text(unicode.to_string())
        .font(ICONS)
        .width(20)
        .horizontal_alignment(alignment::Horizontal::Center)
}

fn edit_icon() -> Text<'static> {
    icon('\u{F303}')
}

impl Application for BaseTop {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn title(&self) -> String {
        "CpuInfos".to_string()
    }

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            BaseTop {
                cpuinfos: Vec::new(),
            },
            Command::batch(vec![
                font::load(include_bytes!("../fonts/icons.ttf").as_slice())
                    .map(|_| Message::Nothing),
                Command::perform(async {}, |_| Message::RequestUpdate),
            ]),
        )
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        if self.cpuinfos.is_empty() {
            container(text("None")).center_y().center_x().into()
        } else {
            column(self.cpuinfos.iter().map(|cpuinfo| cpuinfo.view()).collect())
                .spacing(20)
                .into()
        }
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        if let Message::RequestUpdate = message {
            self.cpuinfos = cpuinfofs::get_cpuinfo().unwrap_or(vec![]);
        }
        Command::none()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::RequestUpdate)
    }
}
