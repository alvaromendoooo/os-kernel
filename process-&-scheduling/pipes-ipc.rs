use std::io::{self, Read};

struct Pipe {
    cap: usize,
    buf: Vec<u8>,
    nread: usize,
    nwrite: usize,
}

impl Pipe {
    // Store as many bytes as fit. Returns the number stored.
    fn write(&mut self, data: &[u8]) -> usize {
        let space = self.cap - (self.nwrite - self.nread);
        let count = space.min(data.len()); // Length of data to be stored in self.buf
        
        for (i, byte) in data.iter().take(count).enumerate() {
            let index_to_start_fill = (self.nwrite + i) % self.cap;
            self.buf[index_to_start_fill] = *byte;
        }

        self.nwrite += count;
        count
    }

    // Remove up to n bytes from the front and return them (empty if the pipe is empty).
    fn read(&mut self, n: usize) -> Vec<u8> {
        let mut bytes_read: Vec<u8> = Vec::new();
        let take = n.min(self.nwrite - self.nread);
        if !self.buf.is_empty() {
            for (i, _) in self.buf.iter().take(take).enumerate() {
                let index_to_start_read = (self.nread + i) % self.cap;
                bytes_read.push(self.buf[index_to_start_read]);
            }
        }

        self.nread += take;
        bytes_read
    }
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut p = Pipe { cap: 1, buf: vec![0u8; 1], nread: 0, nwrite: 0 };
    let mut r_open = true;
    let mut w_open = true;
    let mut out = String::new();
    for line in input.lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        if w.is_empty() {
            continue;
        }
        match w[0] {
            "PIPE" => {
                let cap: usize = w[1].parse().unwrap();
                p = Pipe { cap: cap, buf: vec![0u8; cap], nread: 0, nwrite: 0 };
                r_open = true;
                w_open = true;
            }
            "WRITE" => {
                let mut data: Vec<u8> = Vec::new();
                if w.len() > 1 {
                    let hex = w[1].as_bytes();
                    let mut i = 0;
                    while i + 1 < hex.len() {
                        let s = String::from_utf8_lossy(&hex[i..i + 2]).to_string();
                        data.push(u8::from_str_radix(&s, 16).unwrap());
                        i += 2;
                    }
                }
                if !w_open {
                    out.push_str("EBADF\n");
                } else if !r_open {
                    out.push_str("EPIPE\n");
                } else {
                    let n = p.write(&data);
                    if n == data.len() {
                        out.push_str(&format!("WROTE {}\n", n));
                    } else {
                        out.push_str(&format!("WROTE {} BLOCK\n", n));
                    }
                }
            }
            "READ" => {
                if !r_open {
                    out.push_str("EBADF\n");
                    continue;
                }
                let got = p.read(w[1].parse().unwrap());
                if !got.is_empty() {
                    out.push_str("READ ");
                    for b in got.iter() {
                        out.push_str(&format!("{:02x}", b));
                    }
                    out.push('\n');
                } else if w_open {
                    out.push_str("BLOCK\n");
                } else {
                    out.push_str("EOF\n");
                }
            }
            "CLOSE" => {
                if w[1] == "R" {
                    r_open = false;
                } else {
                    w_open = false;
                }
                out.push_str("OK\n");
            }
            "STATE" => {
                out.push_str(&format!("nread={} nwrite={} head={} tail={} used={}\n", p.nread, p.nwrite, p.nread % p.cap, p.nwrite % p.cap, p.nwrite - p.nread));
            }
            _ => {}
        }
    }
    print!("{}", out);
}