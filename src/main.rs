use iced::{alignment, executor, Application, Command, Element, Length, Theme};

use iced::font::{self, Font};
use iced::theme;
use iced::widget::{button, column, container, row, scrollable, text, Text};

mod cpuinfo;
mod procinfos;

use cpuinfo::CpuMessage;

fn main() -> iced::Result {
    env_logger::builder().format_timestamp(None).init();
    BaseTop::run(iced::Settings::default())
}

struct BaseTop {
    cpuinfos: Vec<CpuMessage>,
}

#[allow(unused)]
#[derive(Clone, Debug)]
pub enum Message {
    RequestUpdate,
    Nothing,
}

#[allow(unused)]
const ICONS: Font = Font::with_name("Iced-Todos-Icons");

#[allow(unused)]
fn icon(unicode: char) -> Text<'static> {
    text(unicode.to_string())
        .font(ICONS)
        .width(20)
        .horizontal_alignment(alignment::Horizontal::Center)
}

#[allow(unused)]
fn edit_icon() -> Text<'static> {
    icon('\u{F303}')
}

impl BaseTop {
    fn buttonbox(&self) -> Element<Message> {
        row![
            button(text("main"))
                .style(theme::Button::Primary)
                .padding(8),
            button(text("top")).style(theme::Button::Text).padding(8),
        ]
        .into()
    }
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
        let bottom: Element<_> = if self.cpuinfos.is_empty() {
            container(text("None")).center_y().center_x().into()
        } else {
            container(scrollable(
                column(self.cpuinfos.iter().map(|cpuinfo| cpuinfo.view()).collect()).spacing(20),
            ))
            .height(Length::Fill)
            .into()
        };
        column![self.buttonbox(), bottom].into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        if let Message::RequestUpdate = message {
            self.cpuinfos = cpuinfo::get_cpuinfo().unwrap_or(vec![]);
        }
        Command::none()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::RequestUpdate)
    }
}
