use iced::theme::Container;
use iced::widget::{container, row, text};
use iced::{Alignment, Element, Length};
const CPU_INFO: &str = "/proc/cpuinfo";

const NAME_PROMOTE: &str = "model name";

const PROCESSOR_PROMOTE: &str = "processor";

const MHZ_PROMOTE: &str = "cpu MHz";
const CACHE_SIZE_PROMOTE: &str = "cache size";

pub struct CpuMessageVec {
    inner: Vec<CpuMessage>,
}

impl CpuMessageVec {
    pub fn iter(&self) -> impl Iterator<Item = &CpuMessage> {
        self.inner.iter()
    }
    pub fn refresh(&mut self) {
        self.inner = get_cpuinfo().unwrap_or(Vec::new())
    }
    pub fn new() -> Self {
        CpuMessageVec { inner: Vec::new() }
    }
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct CpuMessage {
    pub name: String,
    pub processor: usize,
    pub mhz: String, // TODO: to i32
    pub cache_size: String,
    pub show_more: bool,
}

use crate::Message;

impl CpuMessage {
    pub fn view(&self) -> Element<Message> {
        let row: Element<Message> = row![
            text(self.name.as_str()),
            text(self.processor.to_string()),
            text(self.mhz.as_str()),
            text(self.cache_size.as_str()),
        ]
        .spacing(10)
        .align_items(Alignment::Center)
        .into();

        container(row)
            .center_x()
            .center_y()
            .width(Length::Fill)
            .style(Container::Box)
            .padding(30)
            .into()
    }
}

fn get_key(line: &str) -> String {
    line.split(':').last().unwrap_or("").to_string()
}

pub(super) fn get_cpuinfo() -> Option<Vec<CpuMessage>> {
    let Ok(cpuinfo) = std::fs::read_to_string(CPU_INFO).map(|cpuinfo| cpuinfo.trim().to_string()) else {
        return None;
    };
    let mut output = Vec::new();
    let cpuinfos: Vec<&str> = cpuinfo.split("\n\n").collect();
    for cpus in cpuinfos {
        let mut name: String = "UnKnown".to_string();
        let mut processor: usize = 0;
        let mut mhz: String = "UnKnown".to_string();
        let mut cache_size: String = "UnKnown".to_string();

        for info in cpus.lines() {
            if info.starts_with(NAME_PROMOTE) {
                name = get_key(info);
            }
            if info.starts_with(PROCESSOR_PROMOTE) {
                processor = get_key(info).replace(' ', "").parse().unwrap_or(0);
            }

            if info.starts_with(MHZ_PROMOTE) {
                mhz = get_key(info);
            }
            if info.starts_with(CACHE_SIZE_PROMOTE) {
                cache_size = get_key(info);
            }
        }
        output.push(CpuMessage {
            name,
            processor,
            mhz,
            cache_size,
            show_more: false,
        });
    }
    Some(output)
}
