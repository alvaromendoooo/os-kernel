use std::collections::VecDeque;
use std::io::{self, Read};

const CLOCK: u64 = 1193182;

#[derive(Clone)]
struct Task {
    name: String,
    slice: i64,
    left: i64,
}

struct Machine {
    divisor: u64,
    state: u8, // 0 idle, 1 awaiting low byte, 2 awaiting high byte
    low_byte: u64,
    ticks: u64,
    counts: u64,
    ready: VecDeque<Task>,
    running: Option<Task>,
}

impl Machine {
    // Handle one `out`. Returns the output line, if any.
    fn out_port(&mut self, port: u64, value: u64) -> Option<String> {
        match port {
            67 => {
                let channel = (value & 0xC0) >> 6; // Bits 6-7
                let access = (value & 0x30) >> 4; // Bits 5-4

                if channel != 0 || access != 3 {
                    Some("PIT ignored".to_string())
                } else {
                    let mode = (value & 0x0E) >> 1;
                    let _mode = match mode { // Reinterpretation of mode value
                        7 => 3,
                        6 => 2,
                        other => other,
                    };

                    self.state = 1;
                    Some(format!("PIT mode={}", _mode))
                }
            }, // 43
            64 => {
                match self.state {
                    1 => {
                        self.low_byte = value;
                        self.state = 2; // Awaits high_byte
                        None
                    },
                    2 => {
                        let high = value;
                        let low = self.low_byte;
                        let mut divisor = high * 256 + low;

                        if divisor == 0 {
                            divisor = 65536;
                        }

                        self.state = 0; // Becomes idle again
                        self.divisor = divisor;

                        let hz = (CLOCK + divisor / 2) / divisor;
                        let period_us = divisor * 1000000 / CLOCK;

                        Some(format!("PIT divisor={} hz={} period_us={}", divisor, hz, period_us)) 
                    },
                    _  => None,
                }
            }, // 40
            _ => None,
        }
    }

    // One timer interrupt.
    fn tick(&mut self, out: &mut String) {
        self.ticks += 1;
        self.counts += self.divisor;

        if let Some(mut task) = self.running.take() {
            task.left -= 1;

            if task.left <= 0 {
                if !self.ready.is_empty() {
                    let old_name = task.name.clone();

                    task.left = task.slice;
                    self.ready.push_back(task);

                    let mut new_task = self.ready.pop_front().unwrap();
                    new_task.left = new_task.slice;

                    out.push_str(&format!("tick {}: {} -> {}\n", self.ticks, old_name, new_task.name));

                    // This new task becomes the new running task
                    self.running = Some(new_task);
                } else {
                    // Empty queue, running task refresh
                    task.left = task.slice;
                    self.running = Some(task);
                }
            } else {
                // Task still running
                self.running = Some(task);
            }
        }
    }
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut m = Machine { divisor: 65536, state: 0, low_byte: 0, ticks: 0, counts: 0, ready: VecDeque::new(), running: None };
    let mut out = String::new();
    for line in input.lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        if w.is_empty() {
            continue;
        }
        match w[0] {
            "OUT" => {
                let p = u64::from_str_radix(w[1], 16).unwrap();
                let v = u64::from_str_radix(w[2], 16).unwrap();
                if let Some(r) = m.out_port(p, v) {
                    out.push_str(&r);
                    out.push('\n');
                }
            }
            "TASK" => {
                let s: i64 = w[2].parse().unwrap();
                let t = Task { name: w[1].to_string(), slice: s, left: s };
                if m.running.is_none() {
                    m.running = Some(t);
                } else {
                    m.ready.push_back(t);
                }
            }
            "TICK" => {
                let n: u64 = if w.len() > 1 { w[1].parse().unwrap() } else { 1 };
                for _ in 0..n {
                    m.tick(&mut out);
                }
            }
            "UPTIME" => out.push_str(&format!("ticks={} ms={}\n", m.ticks, m.counts * 1000 / CLOCK)),
            "STATUS" => match m.running {
                None => out.push_str("running=- left=0 queue=-\n"),
                Some(ref r) => {
                    let names: Vec<String> = m.ready.iter().map(|t| t.name.clone()).collect();
                    let q = if names.is_empty() { "-".to_string() } else { names.join(" ") };
                    out.push_str(&format!("running={} left={} queue={}\n", r.name, r.left, q));
                }
            },
            _ => {}
        }
    }
    print!("{}", out);
}