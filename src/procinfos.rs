use std::path::Path;

use iced::theme::Container;
use iced::widget::{container, row, text};
use iced::{Alignment, Element, Length};

use crate::Message;

const PROC_NAME_PROMOTE: &str = "Name";
const PROC_PID_PROMOTE: &str = "Pid";
const PROC_PPID_PROMOTE: &str = "PPid";
const PROC_THREADS_PROMOTE: &str = "Threads";

fn get_key(line: &str) -> String {
    line.split(':').last().unwrap_or("").trim().to_string()
}

#[allow(unused)]
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
    pub fn view(&self) -> Element<Message> {
        let row: Element<Message> = row![
            text(self.name.as_str()),
            text(self.ppid.to_string()),
            text(self.pid.to_string()),
            text(
                self
                    .cmdline
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
            .center_x()
            .center_y()
            .width(Length::Fill)
            .style(Container::Box)
            .padding(30)
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

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct ProcInfoVec {
    is_tree: bool, // TODO: draw tree
    inner: Vec<ProcInfo>,
}

impl ProcInfoVec {
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
