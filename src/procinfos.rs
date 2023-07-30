use std::path::Path;

use iced::theme::{self, Container};
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Length};

use crate::Message;

const PROC_NAME_PROMOTE: &str = "Name";
const PROC_PID_PROMOTE: &str = "Pid";
const PROC_PPID_PROMOTE: &str = "PPid";
const PROC_THREADS_PROMOTE: &str = "Threads";

fn get_key(line: &str) -> String {
    line.split(':').last().unwrap_or("").trim().to_string()
}

#[derive(Clone, Debug)]
pub struct ProcInfo {
    pub name: String,
    pub pid: usize,
    pub ppid: usize,
    pub threads: usize,
    pub cmdline: Option<String>,
    pub children: Vec<ProcInfo>,
}

impl ProcInfo {
    pub fn treeview(&self, tabnum: usize) -> Element<Message> {
        let ppidlen = 60_f32 + tabnum as f32 * 30_f32;
        let row: Element<Message> = row![
            text(self.name.as_str()).width(Length::Fixed(150_f32)),
            text(self.pid.to_string()).width(Length::Fixed(60_f32)),
            text(self.ppid.to_string()).width(Length::Fixed(ppidlen)),
            text(self.threads.to_string()).width(Length::Fixed(60_f32)),
            text(
                self.cmdline
                    .as_ref()
                    .map(|name| if name.is_empty() {
                        self.name.clone()
                    } else {
                        name.to_string()
                    })
                    .unwrap_or(self.name.to_string())
                    .as_str()
            ),
        ]
        .spacing(10)
        .align_items(Alignment::Start)
        .into();
        if self.children.is_empty() {
            container(row)
                .width(Length::Fill)
                .style(Container::Box)
                .padding(if tabnum == 0 { 10 } else { 0 })
                .into()
        } else {
            let mut rows: Vec<Element<Message>> = Vec::new();
            rows.push(row);
            for child in self.children.iter() {
                rows.push(child.treeview(tabnum + 1));
            }
            container(column(rows).padding(0).spacing(5))
                .width(Length::Fill)
                .style(Container::Box)
                .padding(10)
                .into()
        }
    }

    pub fn view(&self) -> Element<Message> {
        let row: Element<Message> = row![
            text(self.name.as_str()).width(Length::Fixed(150_f32)),
            text(self.pid.to_string()).width(Length::Fixed(60_f32)),
            text(self.ppid.to_string()).width(Length::Fixed(60_f32)),
            text(self.threads.to_string()).width(Length::Fixed(60_f32)),
            text(
                self.cmdline
                    .as_ref()
                    .map(|name| if name.is_empty() {
                        self.name.clone()
                    } else {
                        name.to_string()
                    })
                    .unwrap_or(self.name.to_string())
                    .as_str()
            ),
        ]
        .spacing(10)
        .align_items(Alignment::Start)
        .into();

        container(row)
            .width(Length::Fill)
            .style(Container::Box)
            .padding(10)
            .into()
    }
    pub fn from_file<P: AsRef<Path>>(pa: P) -> Option<Self> {
        let Ok(proccontent) = std::fs::read_to_string(&pa).map(|s| s.trim().to_string()) else {
            return None;
        };
        let mut name = String::new();
        let mut pid = 0;
        let mut ppid = 0;
        let mut threads = 1;
        let mut cmdline = None;
        let mut children = Vec::new();
        for info in proccontent.lines() {
            if info.starts_with(PROC_NAME_PROMOTE) {
                name = get_key(info);
            }
            if info.starts_with(PROC_PID_PROMOTE) {
                pid = get_key(info).parse().unwrap();
            }
            if info.starts_with(PROC_PPID_PROMOTE) {
                ppid = get_key(info).parse().unwrap();
            }
            if info.starts_with(PROC_THREADS_PROMOTE) {
                threads = get_key(info).parse().unwrap();
            }
        }
        let fullpath: &Path = pa.as_ref().parent().unwrap();

        let cmdlinepa: &Path = &fullpath.join("cmdline");
        if cmdlinepa.exists() {
            if let Ok(cmdlineread) =
                std::fs::read_to_string(cmdlinepa).map(|s| s.trim().replace('\0', " ").to_string())
            {
                cmdline = Some(cmdlineread)
            }
        }

        let taskpath: &Path = &fullpath.join("task");
        if taskpath.exists() {
            let pathstr = taskpath.to_string_lossy().to_string();
            let pattern = format!("{pathstr}/*/status");
            for pa in glob::glob(&pattern).into_iter().flatten().flatten() {
                if let Some(procinfo) = ProcInfo::from_file(pa) {
                    if procinfo.pid != pid {
                        children.push(procinfo);
                    }
                }
            }
        }
        Some(ProcInfo {
            name,
            pid,
            ppid,
            threads,
            cmdline,
            children,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ProcInfoVec {
    pub is_tree: bool,
    inner: Vec<ProcInfo>,
}

impl ProcInfoVec {
    pub fn title(&self) -> Element<Message> {
        let row: Element<Message> = row![
            text("Name").width(Length::Fixed(150_f32)),
            text("Pid").width(Length::Fixed(60_f32)),
            text("PPid").width(Length::Fixed(60_f32)),
            text("Threads").width(Length::Fixed(60_f32)),
            text("Cmdline")
        ]
        .spacing(10)
        .align_items(Alignment::Start)
        .into();

        container(row)
            .width(Length::Fill)
            .style(Container::Box)
            .padding(10)
            .into()
    }

    pub fn top_buttons(&self) -> Element<Message> {
        row![
            button(text("Normal"))
                .style({
                    if !self.is_tree {
                        theme::Button::Primary
                    } else {
                        theme::Button::Text
                    }
                })
                .on_press(Message::ProcInfoShowTree(false))
                .padding(8),
            button(text("Tree"))
                .style({
                    if self.is_tree {
                        theme::Button::Primary
                    } else {
                        theme::Button::Text
                    }
                })
                .on_press(Message::ProcInfoShowTree(true))
                .padding(8),
        ]
        .into()
    }
    pub fn refresh(&mut self) {
        let mut procs = Vec::new();
        for pa in glob::glob("/proc/*/status").into_iter().flatten().flatten() {
            if let Some(procinfo) = ProcInfo::from_file(pa) {
                procs.push(procinfo);
            }
        }
        self.inner = procs;
    }

    pub fn new() -> Self {
        ProcInfoVec {
            is_tree: false,
            inner: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ProcInfo> {
        self.inner.iter()
    }

    #[allow(unused)]
    pub fn tree(&self) -> Self {
        let mut procinfos: Vec<ProcInfo> = Vec::new();
        let mut markstatus: [bool; 5000000] = [false; 5000000];
        let mut oldinfos = self.inner.clone();
        while !oldinfos.is_empty() {
            let mut nextinfos = oldinfos.clone();
            for info in oldinfos.iter() {
                markstatus[info.ppid] = true;
            }
            let mut procinfosnext = Vec::new();
            let mut procinfostatus = vec![false; procinfos.len()];
            for (index, info) in oldinfos.iter().enumerate().rev() {
                if markstatus[info.pid] {
                    markstatus[info.pid] = false;
                } else {
                    let mut thisinfo = info.clone();
                    for (procindex, prinfo) in procinfos.iter().enumerate() {
                        if prinfo.ppid == thisinfo.pid {
                            thisinfo.children.push(prinfo.clone());
                            procinfostatus[procindex] = true;
                        }
                    }
                    procinfosnext.push(thisinfo);
                    nextinfos.remove(index);
                }
            }
            for (index, procstatus) in procinfostatus.iter().enumerate() {
                if !procstatus {
                    procinfosnext.push(procinfos[index].clone());
                }
            }
            procinfos = procinfosnext;

            oldinfos = nextinfos;
        }
        ProcInfoVec {
            is_tree: true,
            inner: procinfos,
        }
    }

    #[allow(unused)]
    pub fn to_vec(&self) -> &Vec<ProcInfo> {
        &self.inner
    }
}
