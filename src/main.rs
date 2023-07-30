use iced::{alignment, executor, Application, Command, Element, Length, Theme};

use iced::font::{self, Font};
use iced::theme;
use iced::widget::{button, column, container, row, scrollable, text, text_input, Text};

mod cpuinfo;
mod procinfos;

use cpuinfo::CpuMessageVec;
use procinfos::{InfoShowKind, ProcInfoVec};

fn main() -> iced::Result {
    env_logger::builder().format_timestamp(None).init();
    BaseTop::run(iced::Settings::default())
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    #[default]
    CpuInfoPage,
    ProcInfoPage,
}

struct BaseTop {
    page: Page,
    cpuinfos: CpuMessageVec,
    procinfos: ProcInfoVec,
}

#[derive(Clone, Debug)]
pub enum Message {
    RequestCpuInfoUpdate,
    RequestProcInfoUpdate,
    StateChanged(Page),

    ProcInfoShowTree(procinfos::InfoShowKind),
    ProcSortMethodChanged(procinfos::SortMethod),
    ProcSearchBarVisibleChanged(bool),
    ProcSearchPatternChanged(String),

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
        container(row![
            button(text("cpuInfo"))
                .style({
                    if self.page == Page::CpuInfoPage {
                        theme::Button::Primary
                    } else {
                        theme::Button::Text
                    }
                })
                .on_press(Message::StateChanged(Page::CpuInfoPage))
                .padding(8),
            button(text("top"))
                .style({
                    if self.page == Page::ProcInfoPage {
                        theme::Button::Primary
                    } else {
                        theme::Button::Text
                    }
                })
                .on_press(Message::StateChanged(Page::ProcInfoPage))
                .padding(8),
        ])
        .width(Length::Fill)
        .center_x()
        .into()
    }
}

impl Application for BaseTop {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn title(&self) -> String {
        match self.page {
            Page::CpuInfoPage => "SystemMonitor-CpuInfo".to_string(),
            Page::ProcInfoPage => "SystemMonitor-ProcInfo".to_string(),
        }
    }

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            BaseTop {
                page: Page::default(),
                cpuinfos: CpuMessageVec::new(),
                procinfos: ProcInfoVec::new(),
            },
            Command::batch(vec![
                font::load(include_bytes!("../fonts/icons.ttf").as_slice())
                    .map(|_| Message::Nothing),
                Command::perform(async {}, |_| Message::RequestCpuInfoUpdate),
                Command::perform(async {}, |_| Message::RequestProcInfoUpdate),
            ]),
        )
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let bottom: Element<_> = match self.page {
            Page::CpuInfoPage => {
                if self.cpuinfos.is_empty() {
                    container(text("None")).center_y().center_x().into()
                } else {
                    container(scrollable(
                        column(self.cpuinfos.iter().map(|cpuinfo| cpuinfo.view()).collect())
                            .spacing(20),
                    ))
                    .height(Length::Fill)
                    .into()
                }
            }
            Page::ProcInfoPage => {
                if self.procinfos.is_empty() {
                    container(text("None")).center_y().center_x().into()
                } else {
                    container(
                        column({
                            let mut col: Vec<Element<Message>> = Vec::new();
                            if self.procinfos.showsearchbar {
                                col.push(self.procinfos.searchbar());
                            }
                            col.append(&mut vec![
                                self.procinfos.top_buttons(),
                                self.procinfos.title(),
                                scrollable({
                                    if self.procinfos.showsearchbar {
                                        column(match self.procinfos.infoshowkind {
                                            InfoShowKind::Normal => self
                                                .procinfos
                                                .iter_search()
                                                .map(|procinfo| procinfo.view())
                                                .collect(),
                                            InfoShowKind::TreeWithFullInfo => self
                                                .procinfos
                                                .iter_tree_search()
                                                .map(|procinfo| procinfo.treeview(0))
                                                .collect(),
                                            InfoShowKind::TreeWithLessInfo => self
                                                .procinfos
                                                .iter_search()
                                                .map(|procinfo| procinfo.treeview(0))
                                                .collect(),
                                        })
                                        .spacing(20)
                                    } else {
                                        column(match self.procinfos.infoshowkind {
                                            InfoShowKind::Normal => self
                                                .procinfos
                                                .iter()
                                                .map(|procinfo| procinfo.view())
                                                .collect(),
                                            InfoShowKind::TreeWithFullInfo => self
                                                .procinfos
                                                .iter_tree()
                                                .map(|procinfo| procinfo.treeview(0))
                                                .collect(),
                                            InfoShowKind::TreeWithLessInfo => self
                                                .procinfos
                                                .iter()
                                                .map(|procinfo| procinfo.treeview(0))
                                                .collect(),
                                        })
                                        .spacing(20)
                                    }
                                })
                                .into(),
                            ]);
                            col
                        })
                        .spacing(10),
                    )
                    .height(Length::Fill)
                    .into()
                }
            }
        };
        column![self.buttonbox(), bottom].into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::RequestCpuInfoUpdate => self.cpuinfos.refresh(),
            Message::RequestProcInfoUpdate => self.procinfos.refresh(),
            Message::StateChanged(page) => self.page = page,
            Message::ProcInfoShowTree(state) => self.procinfos.infoshowkind = state,
            Message::ProcSortMethodChanged(method) => self.procinfos.set_sort_method(method),
            Message::ProcSearchBarVisibleChanged(visible) => {
                self.procinfos.showsearchbar = visible;
                if visible {
                    return text_input::focus(procinfos::INPUT_ID.clone());
                }
            }
            Message::ProcSearchPatternChanged(pattern) => self.procinfos.set_searchpattern(pattern),
            _ => {}
        }
        Command::none()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced::Subscription::batch([
            iced::time::every(std::time::Duration::from_secs(1))
                .map(|_| Message::RequestCpuInfoUpdate),
            iced::time::every(std::time::Duration::from_secs(2))
                .map(|_| Message::RequestProcInfoUpdate),
            iced::subscription::events_with(|event, status| {
                if let iced::event::Status::Captured = status {
                    return None;
                }
                if let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key_code: iced::keyboard::KeyCode::F,
                    modifiers: iced::keyboard::Modifiers::ALT,
                }) = event
                {
                    Some(Message::ProcSearchBarVisibleChanged(true))
                } else {
                    None
                }
            }),
        ])
    }
}
