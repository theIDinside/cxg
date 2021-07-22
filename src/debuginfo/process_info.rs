use std::io::Read;

#[derive(Debug)]
pub struct ProcessInfo {
    // name
    pub name: String,
    // process id
    pub pid: usize,
    // virtual memory usage, peak
    pub virtual_mem_usage_peak: usize,
    // virtual memory usage
    pub virtual_mem_usage: usize,
    /// Resident set size
    pub rss: usize,
    // shared library code size
    pub shared_lib_code: usize,
}

impl ProcessInfo {
    pub fn new() -> std::io::Result<ProcessInfo> {
        let rss = {
            let mut file = std::fs::File::open("/proc/self/smaps_rollup").expect("failed to open smaps_rollup");
            let mut buf = String::with_capacity(1024);
            file.read_to_string(&mut buf)?;

            let data: String = buf
                .lines()
                .skip(1)
                .take(1)
                .flat_map(|line| line.chars().filter(|c| c.is_digit(10)))
                .collect();
            data.parse().unwrap()
        };

        let mut file = std::fs::File::open("/proc/self/status").expect("failed to open statm");
        let mut buf = String::with_capacity(1024);
        file.read_to_string(&mut buf)?; // .expect("failed to read data");
        let to_find = vec![0, 5, 16, 17, 28];
        let mut items: Vec<String> = buf
            .lines()
            .enumerate()
            .filter(|(line_no, _)| to_find.contains(line_no))
            .map(|(i, line)| line.chars().filter(|c| if i == 0 { true } else { c.is_digit(10) }).collect())
            .collect();
        let name = items.remove(0).chars().skip(6).collect();
        Ok(ProcessInfo {
            name,
            pid: items.remove(0).parse().expect("failed to parse pid"),
            virtual_mem_usage_peak: items.remove(0).parse().expect("failed to parse peak virtual memory usage"),
            virtual_mem_usage: items.remove(0).parse().expect("failed to parse virtual memory usage"),
            rss,
            shared_lib_code: items.remove(0).parse().expect("failed to parse shared library code size"),
        })
    }
}
