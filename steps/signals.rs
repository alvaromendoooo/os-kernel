use std::collections::HashMap;
use std::io::{self, Read};

struct Proc {
    disposition: HashMap<u32, &'static str>, // "handler" | "ignore"; missing means default
    blocked: u32,
    pending: u32,
    dead: bool,
}

fn name_of(sig: u32) -> Option<&'static str> {
    Some(match sig {
        1 => "SIGHUP", 2 => "SIGINT", 3 => "SIGQUIT", 4 => "SIGILL", 6 => "SIGABRT", 8 => "SIGFPE",
        9 => "SIGKILL", 10 => "SIGUSR1", 11 => "SIGSEGV", 12 => "SIGUSR2", 13 => "SIGPIPE",
        14 => "SIGALRM", 15 => "SIGTERM", 17 => "SIGCHLD", 18 => "SIGCONT", 19 => "SIGSTOP",
        20 => "SIGTSTP", 28 => "SIGWINCH",
        _ => return None,
    })
}

fn default_action(sig: u32) -> &'static str {
    match sig {
        3 | 4 | 6 | 8 | 11 => "CORE",
        17 | 28 => "IGNORED",
        18 => "CONT",
        19 | 20 => "STOP",
        _ => "TERMINATE",
    }
}

fn bit(sig: u32) -> u32 {
    1u32 << (sig - 1)
}

impl Proc {
    // Deliver `sig` now: print its line and mark the process dead if the action kills it.
    fn deliver(&mut self, sig: u32, out: &mut String) {
        let d = self.disposition.get(&sig).cloned().unwrap_or("default");
        let result = if d == "handler" {
            "HANDLED"
        } else if d == "ignore" {
            "IGNORED"
        } else {
            default_action(sig)
        };
        out.push_str(&format!("{} {}\n", name_of(sig).unwrap(), result));
        if d == "default" && (result == "TERMINATE" || result == "CORE") {
            self.dead = true;
        }
    }

    // SIGNAL n: deliver now, or hold it pending (printing PENDING / MERGED) if it is blocked.
    fn generate(&mut self, sig: u32, out: &mut String) {
        let mask = bit(sig);
        if self.blocked & mask != 0 {
            if self.pending & mask != 0 {
                out.push_str(&format!("{} MERGED\n", name_of(sig).unwrap()));
            } else {
                self.pending |= mask;
                out.push_str(&format!("{} PENDING\n", name_of(sig).unwrap()));
            }
        } else {
            self.deliver(sig, out);
        }
    }

    // Deliver every pending signal that is not blocked, lowest number first.
    fn flush(&mut self, out: &mut String) {
        for sig in 1..=31 {
            let mask = bit(sig);
            
            if self.blocked & mask != 0 {
                continue;
            }

            if (self.pending & mask) != 0 {
                self.pending &= !mask;
                self.deliver(sig, out);

                if self.dead {
                    break;
                }
            }
        }
    }
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut p = Proc { disposition: HashMap::new(), blocked: 0, pending: 0, dead: false };
    let mut out = String::new();
    for line in input.lines() {
        if p.dead {
            break;
        }
        let w: Vec<&str> = line.split_whitespace().collect();
        if w.is_empty() {
            continue;
        }
        let c = w[0];
        if c == "PENDING" {
            out.push_str(&format!("pending=0x{:08x}\n", p.pending));
            continue;
        }
        if c == "MASK" {
            out.push_str(&format!("blocked=0x{:08x}\n", p.blocked));
            continue;
        }
        if c == "SETMASK" && w.len() >= 2 {
            p.blocked = u32::from_str_radix(w[1], 16).unwrap() & !(bit(9) | bit(19));
            p.flush(&mut out);
            continue;
        }
        if w.len() < 2 {
            continue;
        }
        let sig: u32 = w[1].parse().unwrap();
        if name_of(sig).is_none() {
            out.push_str("EINVAL\n");
            continue;
        }
        if c == "HANDLE" || c == "IGNORE" {
            if sig == 9 || sig == 19 {
                out.push_str("EINVAL\n");
                continue;
            }
            if c == "HANDLE" {
                p.disposition.insert(sig, "handler");
            } else {
                p.disposition.insert(sig, "ignore");
                p.pending &= !bit(sig);
            }
            out.push_str("OK\n");
        } else if c == "DEFAULT" {
            p.disposition.remove(&sig);
            if default_action(sig) == "IGNORED" {
                p.pending &= !bit(sig);
            }
            out.push_str("OK\n");
        } else if c == "BLOCK" {
            if sig != 9 && sig != 19 {
                p.blocked |= bit(sig);
            }
        } else if c == "UNBLOCK" {
            p.blocked &= !bit(sig);
            p.flush(&mut out);
        } else if c == "SIGNAL" {
            p.generate(sig, &mut out);
        }
    }
    print!("{}", out);
}