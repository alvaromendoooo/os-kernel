use std::collections::BTreeMap;
use std::io::{self, Read};

#[derive(Clone, Copy)]
struct Pte {
    frame: usize,
    w: u8,
    cow: u8,
}

struct Machine {
    frames_total: usize,
    refs: Vec<i32>,
    procs: BTreeMap<i64, BTreeMap<i64, Pte>>,
}

impl Machine {
    fn lowest_free(&self) -> Option<usize> {
        for f in 0..self.frames_total {
            if self.refs[f] == 0 {
                return Some(f);
            }
        }
        None
    }

    // Share the parent's pages with a new child. False if parent is missing or child exists.
    fn fork(&mut self, parent: i64, child: i64) -> bool {
        // Check parent missing or child already exists
        if self.procs.contains_key(&child) {
            return false;
        }

        // Get parent page table
        let ptable = match self.procs.get_mut(&parent) {
            Some(t) => t,
            None => return false, // Parent doesn't register ptable
        };

        // Writable pages become read_only + COW.
        for pte in ptable.values_mut() {
            if pte.w == 1 {
                pte.w = 0;
                pte.cow = 1;
            }
        }

        let child_table = ptable.clone();

        // Each shared frames gains one more reference
        for pte in child_table.values() {
            self.refs[pte.frame] += 1;
        }

        // Registry of child
        self.procs.insert(child, child_table);
        
        true
    }

    // Handle a write. Returns the output line.
    fn write(&mut self, pid: i64, vpn: i64) -> String {
        let entry = match self.procs.get(&pid).and_then(|t| t.get(&vpn)) {
            Some(e) => *e,
            None => return "SEGV".to_string(),
        };
        
        if entry.w == 1 {
            return format!("OK {}", entry.frame);
        }

        if entry.cow == 0 {
            return "SEGV".to_string();
        }

        let old = entry.frame;
        if self.refs[old] > 1 {
            let new = match self.lowest_free() {
                Some(f) => f,
                None => return "OOM".to_string()
            };
            self.refs[new] = 1;
            self.refs[old] -= 1;
            let pte = self.procs.get_mut(&pid).unwrap().get_mut(&vpn).unwrap();
            pte.frame = new;
            pte.w = 1;
            pte.cow = 0;
            format!("COPY {} -> {}", old, new)
        } else {
            let pte = self.procs.get_mut(&pid).unwrap().get_mut(&vpn).unwrap();
            pte.w = 1;
            pte.cow = 0;
            format!("REUSE {}", old)
        }
    }
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut m = Machine { frames_total: 0, refs: Vec::new(), procs: BTreeMap::new() };
    m.procs.insert(1, BTreeMap::new());
    let mut out = String::new();
    for line in input.lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        if w.is_empty() {
            continue;
        }
        match w[0] {
            "FRAMES" => {
                m.frames_total = w[1].parse().unwrap();
                m.refs = vec![0; m.frames_total];
            }
            "MAP" => {
                let pid: i64 = w[1].parse().unwrap();
                let vpn: i64 = w[2].parse().unwrap();
                let exists = match m.procs.get(&pid) {
                    Some(t) => t.contains_key(&vpn),
                    None => true,
                };
                if exists {
                    out.push_str("ERROR\n");
                    continue;
                }
                match m.lowest_free() {
                    None => out.push_str("OOM\n"),
                    Some(f) => {
                        m.refs[f] = 1;
                        let e = Pte { frame: f, w: if w[3] == "rw" { 1 } else { 0 }, cow: 0 };
                        m.procs.get_mut(&pid).unwrap().insert(vpn, e);
                        out.push_str(&format!("frame {}\n", f));
                    }
                }
            }
            "FORK" => {
                let ok = m.fork(w[1].parse().unwrap(), w[2].parse().unwrap());
                out.push_str(if ok { "OK\n" } else { "ERROR\n" });
            }
            "READ" => {
                let pid: i64 = w[1].parse().unwrap();
                let vpn: i64 = w[2].parse().unwrap();
                match m.procs.get(&pid).and_then(|t| t.get(&vpn)) {
                    Some(e) => out.push_str(&format!("frame {}\n", e.frame)),
                    None => out.push_str("SEGV\n"),
                }
            }
            "WRITE" => {
                let line = m.write(w[1].parse().unwrap(), w[2].parse().unwrap());
                out.push_str(&line);
                out.push('\n');
            }
            "EXIT" => {
                let pid: i64 = w[1].parse().unwrap();
                match m.procs.remove(&pid) {
                    None => out.push_str("ERROR\n"),
                    Some(t) => {
                        for (_, e) in t.iter() {
                            m.refs[e.frame] -= 1;
                        }
                        out.push_str("OK\n");
                    }
                }
            }
            "PTE" => {
                let pid: i64 = w[1].parse().unwrap();
                let vpn: i64 = w[2].parse().unwrap();
                match m.procs.get(&pid).and_then(|t| t.get(&vpn)) {
                    Some(e) => out.push_str(&format!("frame={} w={} cow={} refs={}\n", e.frame, e.w, e.cow, m.refs[e.frame])),
                    None => out.push_str("UNMAPPED\n"),
                }
            }
            "STAT" => {
                let used = m.refs.iter().filter(|&&r| r > 0).count();
                out.push_str(&format!("used={} free={}\n", used, m.frames_total - used));
            }
            _ => {}
        }
    }
    print!("{}", out);
}