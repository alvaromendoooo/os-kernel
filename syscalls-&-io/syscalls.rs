use std::collections::{HashMap, VecDeque};
use std::io::{self, Read};

struct Kernel {
    user_lo: u64,
    user_hi: u64,
    pid: u64,
    mem: HashMap<u64, u8>,
    stdin: VecDeque<u8>,
}

fn to_hex(b: &[u8]) -> String {
    let mut s = String::new();
    for x in b.iter() {
        s.push_str(&format!("{:02x}", x));
    }
    s
}

fn parse_num(s: &str) -> u64 {
    if s.starts_with("0x") {
        u64::from_str_radix(&s[2..], 16).unwrap()
    } else {
        s.parse::<u64>().unwrap()
    }
}

fn from_hex(s: &str) -> Vec<u8> {
    let mut out = Vec::new();
    let b = s.as_bytes();
    let mut i = 0;
    while i + 1 < b.len() {
        let t = String::from_utf8_lossy(&b[i..i + 2]).to_string();
        out.push(u8::from_str_radix(&t, 16).unwrap());
        i += 2;
    }
    out
}

impl Kernel {
    // True if [buf, buf + count) lies inside the user space without wrapping around.
    fn range_ok(&self, buf: u64, count: u64) -> bool {
        if count == 0 {
            return true;
        }
        match buf.checked_add(count) {
            Some(end) => buf >= self.user_lo && end <= self.user_hi,
            None => false,
        }
    }

    // Returns (signed result, extra text, keep going).
    fn syscall(&mut self, nr: u64, a1: u64, a2: u64, a3: u64) -> (i64, String, bool) {
        let mut ret: i64 = 0;
        let mut extra = String::new();
        let mut more = true; // false only for exit

        let (fd, buf, count) = (a1, a2, a3);

        match nr {
            0 => {
                if fd != 0 {
                    ret = -9;
                } else if !self.range_ok(buf, count) {
                    ret = -14;
                } else {
                    // Clamp to what stdin has left BEFORE looping, so a huge count is harmless
                    let n = count.min(self.stdin.len() as u64) as usize;
                    let mut bytes = Vec::with_capacity(n);
                    for _ in 0..n {
                        bytes.push(self.stdin.pop_front().unwrap());
                    }
                    for (i, b) in bytes.iter().enumerate() {
                        self.mem.insert(buf + i as u64, *b); // range already validated
                    }
                    ret = n as i64;
                    if n > 0 {
                        extra = format!("buf={}", to_hex(&bytes));
                    }
                }
            }
            1 => {
                if fd != 1 && fd != 2 {
                    ret = -9;
                } else if !self.range_ok(buf, count) {
                    ret = -14;
                } else {
                    ret = count as i64;
                    if count > 0 {
                        let mut bytes = Vec::new();
                        for i in 0..count {
                            bytes.push(*self.mem.get(&(buf + i)).unwrap_or(&0));
                        }
                        extra = format!("fd{}={}", fd, to_hex(&bytes));
                    }
                }
            }
            39 => ret = self.pid as i64,
            60 => {
                extra = format!("exit status={}", a1 & 255);
                more = false;
            }
            _ => ret = -38,
        }

        (ret, extra, more)
    }
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut k = Kernel { user_lo: 0, user_hi: 0, pid: 1, mem: HashMap::new(), stdin: VecDeque::new() };
    let mut out = String::new();
    for line in input.lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        if w.is_empty() {
            continue;
        }
        match w[0] {
            "USER" => {
                k.user_lo = u64::from_str_radix(w[1], 16).unwrap();
                k.user_hi = u64::from_str_radix(w[2], 16).unwrap();
            }
            "MEM" => {
                let base = u64::from_str_radix(w[1], 16).unwrap();
                for (i, b) in from_hex(w[2]).iter().enumerate() {
                    k.mem.insert(base + i as u64, *b);
                }
            }
            "STDIN" => {
                for b in from_hex(w[1]) {
                    k.stdin.push_back(b);
                }
            }
            "PID" => k.pid = w[1].parse().unwrap(),
            "MEMDUMP" => {
                let a = u64::from_str_radix(w[1], 16).unwrap();
                let n: u64 = w[2].parse().unwrap();
                let mut b = Vec::new();
                for i in 0..n {
                    b.push(*k.mem.get(&(a + i)).unwrap_or(&0));
                }
                out.push_str(&to_hex(&b));
                out.push('\n');
            }
            "CALL" => {
                let mut args = [0u64; 4];
                for i in 1..w.len() {
                    if i <= 4 {
                        args[i - 1] = parse_num(w[i]);
                    }
                }
                let (ret, extra, more) = k.syscall(args[0], args[1], args[2], args[3]);
                if !more {
                    out.push_str(&extra);
                    out.push('\n');
                    break;
                }
                if extra.is_empty() {
                    out.push_str(&format!("ret={}\n", ret));
                } else {
                    out.push_str(&format!("ret={} {}\n", ret, extra));
                }
            }
            _ => {}
        }
    }
    print!("{}", out);
}