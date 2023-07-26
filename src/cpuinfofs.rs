const CPU_INFO: &str = "/proc/cpuinfo";

const NAME_PROMOTE: &str = "model name";

const PROCESSOR_PROMOTE: &str = "processor";

const MHZ_PROMOTE: &str = "cpu MHz";
const CACHE_SIZE_PROMOTE: &str = "cache size";

use crate::CpuMessage;

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
                processor = get_key(info).replace(" ", "").parse().unwrap_or(0);
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
