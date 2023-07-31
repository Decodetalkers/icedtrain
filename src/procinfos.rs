use std::path::Path;

use crate::Message;
use iced::theme::{self, Container};
use iced::widget::{button, column, container, row, text, text_input};
use iced::{Alignment, Element, Length};
use once_cell::sync::Lazy;

pub static INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

const PROC_NAME_PROMOTE: &str = "Name";
const PROC_PID_PROMOTE: &str = "Pid";
const PROC_PPID_PROMOTE: &str = "PPid";
const PROC_THREADS_PROMOTE: &str = "Threads";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfoShowKind {
    Normal,
    TreeWithFullInfo,
    TreeWithLessInfo,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SortMethod {
    #[default]
    Name,
    Pid,
    PPid,
    Thread,
    CmdLine,
}

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
    fn sort_by(&mut self, sort_method: SortMethod) {
        if !self.children.is_empty() {
            self.children.sort_by(|a, b| match sort_method {
                SortMethod::Pid => a.pid.partial_cmp(&b.pid).unwrap(),
                SortMethod::PPid => a.ppid.partial_cmp(&b.ppid).unwrap(),
                SortMethod::Thread => a.threads.partial_cmp(&b.threads).unwrap(),
                SortMethod::Name => a.name.partial_cmp(&b.name).unwrap(),
                SortMethod::CmdLine => a.cmdline.partial_cmp(&b.cmdline).unwrap(),
            });
            for child in self.children.iter_mut() {
                child.sort_by(sort_method);
            }
        }
    }

    fn is_match_pattern(&self, re: regex::Regex) -> bool {
        re.is_match(&self.name.to_lowercase())
            || re.is_match(
                self.cmdline
                    .as_ref()
                    .unwrap_or(&"".to_string().to_lowercase()),
            )
            || self
                .children
                .iter()
                .any(|unit| unit.is_match_pattern(re.clone()))
    }

    fn filter_children_with_pattern(&self, re: regex::Regex) -> Self {
        if self.children.is_empty() {
            return self.clone();
        }
        let children = self
            .children
            .iter()
            .filter(|unit| unit.is_match_pattern(re.clone()))
            .cloned()
            .map(|unit| unit.filter_children_with_pattern(re.clone()))
            .collect();
        Self {
            children,
            ..self.clone()
        }
    }

    pub fn treeview(&self, tabnum: usize) -> Element<Message> {
        let ppidlen = 60_f32 + tabnum as f32 * 30_f32;
        let row: Element<Message> = row![
            text(self.name.as_str()).width(Length::Fixed(150_f32)),
            text(self.pid.to_string()).width(Length::Fixed(60_f32)),
            text(self.ppid.to_string()).width(Length::Fixed(60_f32)),
            text(self.threads.to_string()).width(Length::Fixed(ppidlen)),
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
            container(column(rows).padding(0).spacing(10))
                .width(Length::Fill)
                .style(Container::Box)
                .padding(if tabnum == 0 { 10 } else { 0 })
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
    pub infoshowkind: InfoShowKind,
    sort_method: SortMethod,
    inner: Vec<ProcInfo>,
    inner_search: Vec<ProcInfo>,
    inner_tree: Vec<ProcInfo>,
    inner_tree_search: Vec<ProcInfo>,
    searchpattern: String,
    pub showsearchbar: bool,
}

impl ProcInfoVec {
    pub fn set_sort_method(&mut self, method: SortMethod) {
        self.sort_method = method;
        self.sort_infos();
    }

    fn sort_infos(&mut self) {
        self.inner.sort_by(|a, b| match self.sort_method {
            SortMethod::Pid => a.pid.partial_cmp(&b.pid).unwrap(),
            SortMethod::PPid => a.ppid.partial_cmp(&b.ppid).unwrap(),
            SortMethod::Thread => a.threads.partial_cmp(&b.threads).unwrap(),
            SortMethod::Name => a.name.partial_cmp(&b.name).unwrap(),
            SortMethod::CmdLine => a.cmdline.partial_cmp(&b.cmdline).unwrap(),
        });
        for item in self.inner.iter_mut() {
            item.sort_by(self.sort_method);
        }

        self.inner_search.sort_by(|a, b| match self.sort_method {
            SortMethod::Pid => a.pid.partial_cmp(&b.pid).unwrap(),
            SortMethod::PPid => a.ppid.partial_cmp(&b.ppid).unwrap(),
            SortMethod::Thread => a.threads.partial_cmp(&b.threads).unwrap(),
            SortMethod::Name => a.name.partial_cmp(&b.name).unwrap(),
            SortMethod::CmdLine => a.cmdline.partial_cmp(&b.cmdline).unwrap(),
        });
        for item in self.inner_search.iter_mut() {
            item.sort_by(self.sort_method);
        }

        self.inner_tree.sort_by(|a, b| match self.sort_method {
            SortMethod::Pid => a.pid.partial_cmp(&b.pid).unwrap(),
            SortMethod::PPid => a.ppid.partial_cmp(&b.ppid).unwrap(),
            SortMethod::Thread => a.threads.partial_cmp(&b.threads).unwrap(),
            SortMethod::Name => a.name.partial_cmp(&b.name).unwrap(),
            SortMethod::CmdLine => a.cmdline.partial_cmp(&b.cmdline).unwrap(),
        });
        for item in self.inner_tree.iter_mut() {
            item.sort_by(self.sort_method);
        }

        self.inner_tree_search
            .sort_by(|a, b| match self.sort_method {
                SortMethod::Pid => a.pid.partial_cmp(&b.pid).unwrap(),
                SortMethod::PPid => a.ppid.partial_cmp(&b.ppid).unwrap(),
                SortMethod::Thread => a.threads.partial_cmp(&b.threads).unwrap(),
                SortMethod::Name => a.name.partial_cmp(&b.name).unwrap(),
                SortMethod::CmdLine => a.cmdline.partial_cmp(&b.cmdline).unwrap(),
            });
        for item in self.inner_tree_search.iter_mut() {
            item.sort_by(self.sort_method);
        }
    }

    pub fn searchbar(&self) -> Element<Message> {
        text_input("Search Pattern", self.searchpattern.as_str())
            .id(INPUT_ID.clone())
            .on_input(Message::ProcSearchPatternChanged)
            .on_submit(Message::ProcSearchBarVisibleChanged(false))
            .padding(5)
            .size(15)
            .into()
    }

    pub fn set_searchpattern(&mut self, pattern: String) {
        self.searchpattern = pattern;
    }

    pub fn title(&self) -> Element<Message> {
        let row: Element<Message> = row![
            button(text("Name"))
                .width(Length::Fixed(150_f32))
                .style({
                    if self.sort_method == SortMethod::Name {
                        theme::Button::Primary
                    } else {
                        theme::Button::Text
                    }
                })
                .on_press(Message::ProcSortMethodChanged(SortMethod::Name)),
            button(text("Pid"))
                .width(Length::Fixed(60_f32))
                .style({
                    if self.sort_method == SortMethod::Pid {
                        theme::Button::Primary
                    } else {
                        theme::Button::Text
                    }
                })
                .on_press(Message::ProcSortMethodChanged(SortMethod::Pid)),
            button(text("PPid"))
                .width(Length::Fixed(60_f32))
                .style({
                    if self.sort_method == SortMethod::PPid {
                        theme::Button::Primary
                    } else {
                        theme::Button::Text
                    }
                })
                .on_press(Message::ProcSortMethodChanged(SortMethod::PPid)),
            button(text("Threads"))
                .width(Length::Fixed(60_f32))
                .style({
                    if self.sort_method == SortMethod::Thread {
                        theme::Button::Primary
                    } else {
                        theme::Button::Text
                    }
                })
                .on_press(Message::ProcSortMethodChanged(SortMethod::Thread)),
            button(text("Cmdline"))
                .style({
                    if self.sort_method == SortMethod::CmdLine {
                        theme::Button::Primary
                    } else {
                        theme::Button::Text
                    }
                })
                .on_press(Message::ProcSortMethodChanged(SortMethod::CmdLine)),
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
                    if self.infoshowkind == InfoShowKind::Normal {
                        theme::Button::Primary
                    } else {
                        theme::Button::Text
                    }
                })
                .on_press(Message::ProcInfoShowTree(InfoShowKind::Normal))
                .padding(8),
            button(text("TreeFullInfo"))
                .style({
                    if self.infoshowkind == InfoShowKind::TreeWithFullInfo {
                        theme::Button::Primary
                    } else {
                        theme::Button::Text
                    }
                })
                .on_press(Message::ProcInfoShowTree(InfoShowKind::TreeWithFullInfo))
                .padding(8),
            button(text("TreeLessInfo"))
                .style({
                    if self.infoshowkind == InfoShowKind::TreeWithLessInfo {
                        theme::Button::Primary
                    } else {
                        theme::Button::Text
                    }
                })
                .on_press(Message::ProcInfoShowTree(InfoShowKind::TreeWithLessInfo))
                .padding(8),
        ]
        .into()
    }

    pub fn refresh(&mut self) {
        let mut procs = Vec::new();
        for pa in glob::glob("/proc/[0-9]*/status")
            .into_iter()
            .flatten()
            .flatten()
        {
            if let Some(procinfo) = ProcInfo::from_file(pa) {
                procs.push(procinfo);
            }
        }
        self.inner = procs;
        self.set_treedata();
        self.set_filiter();
        self.sort_infos();
    }

    pub fn set_filiter(&mut self) {
        let re = regex::Regex::new(&self.searchpattern.to_lowercase()).unwrap();
        self.inner_search = self
            .inner
            .iter()
            .filter(|unit| unit.is_match_pattern(re.clone()))
            .cloned()
            .map(|unit| unit.filter_children_with_pattern(re.clone()))
            .collect();
        self.inner_tree_search = self
            .inner_tree
            .iter()
            .filter(|unit| unit.is_match_pattern(re.clone()))
            .cloned()
            .map(|unit| unit.filter_children_with_pattern(re.clone()))
            .collect();
    }

    pub fn new() -> Self {
        ProcInfoVec {
            sort_method: SortMethod::default(),
            infoshowkind: InfoShowKind::Normal,
            inner: Vec::new(),
            inner_search: Vec::new(),
            inner_tree: Vec::new(),
            inner_tree_search: Vec::new(),
            searchpattern: String::new(),
            showsearchbar: false,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ProcInfo> {
        self.inner.iter()
    }

    pub fn iter_search(&self) -> impl Iterator<Item = &ProcInfo> {
        self.inner_search.iter()
    }

    pub fn iter_tree(&self) -> impl Iterator<Item = &ProcInfo> {
        self.inner_tree.iter()
    }

    pub fn iter_tree_search(&self) -> impl Iterator<Item = &ProcInfo> {
        self.inner_tree_search.iter()
    }

    pub fn set_treedata(&mut self) {
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
        self.inner_tree = procinfos
    }

    #[allow(unused)]
    pub fn to_vec(&self) -> &Vec<ProcInfo> {
        &self.inner
    }
}
