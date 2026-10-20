use std::collections::HashMap;
use std::io::{self, Read};

const NDIRECT: usize = 12;

struct Fs {
    bsize: usize,
    itype: u64,
    nlink: u64,
    size: u64,
    addrs: Vec<u64>,
    blocks: HashMap<u64, Vec<u8>>,
}

fn from_hex(s: &str) -> Vec<u8> {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 1 < b.len() {
        let t = String::from_utf8_lossy(&b[i..i + 2]).to_string();
        out.push(u8::from_str_radix(&t, 16).unwrap());
        i += 2;
    }
    out
}

fn le(raw: &[u8], off: usize, n: usize) -> u64 {
    let mut v: u64 = 0;
    for i in (0..n).rev() {
        v = v * 256 + raw[off + i] as u64;
    }
    v
}

fn blk_name(b: i64) -> String {
    if b < 0 {
        "EFBIG".to_string()
    } else if b == 0 {
        "HOLE".to_string()
    } else {
        b.to_string()
    }
}

impl Fs {
    // The index-th 32-bit little-endian pointer inside disk block `block_no` (0 if absent).
    fn ptr(&self, block_no: u64, index: usize) -> u64 {
        let empty: Vec<u8> = Vec::new();
        let data = self.blocks.get(&block_no).unwrap_or(&empty);
        let mut v: u64 = 0;
        for i in (0..4).rev() {
            let pos = index * 4 + i;
            let b = if pos < data.len() { data[pos] } else { 0 };
            v = v * 256 + b as u64;
        }
        v
    }

    // Disk block for logical block n: >= 0 (0 = hole) or -1 if beyond the inode's reach.
    fn bmap(&self, n: usize) -> i64 {
        if n < NDIRECT {
            self.addrs[n] as i64
        } else if n < NDIRECT + self.bsize / 4 {
            let indirect = self.addrs[NDIRECT];
            if indirect == 0 {
                0
            } else {
                self.ptr(indirect, n - NDIRECT) as i64
            }
        } else {
            -1
        }
    }

    // Number of disk blocks the file occupies, including the indirect block.
    fn count_blocks(&self) -> usize {
        let mut counter = 0;
        for i in 0..self.addrs.len() - 1 {
            if self.addrs[i] != 0 {
                counter += 1;
            }
        }

        // Indirect address treatment
        let indirect_block = self.addrs[12];
        if indirect_block != 0 {
            counter += 1; // Count the indirect

            if let Some(indirect_entries) = self.blocks.get(&indirect_block) {
                for &i_e in indirect_entries {
                    if i_e != 0 {
                        counter += 1;
                    }
                }
            }
        }
        
        counter
    }

    // Append one line per logical block touched by [offset, offset + length), clipped to the size.
    fn span(&self, offset: u64, length: u64, out: &mut String) {
        if length == 0 || offset >= self.size || self.bsize == 0 {
            out.push_str("EOF\n");
            return;
        }

        let bsize = self.bsize as u64;
        let end = offset.saturating_add(length).min(self.size);
        let first_lbn = offset / bsize;
        let last_lbn = (end - 1) / bsize;
        for lbn in first_lbn..=last_lbn {
            let blk = blk_name(self.bmap(lbn as usize));
            let block_start = lbn * bsize;
            let from_bytes = offset.max(block_start) - block_start;
            let to_bytes = end.min(block_start + bsize) - block_start;
            out.push_str(&format!("lbn={} blk={} bytes={}-{}\n", lbn, blk, from_bytes, to_bytes));
        }
    }
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut fs = Fs { bsize: 1024, itype: 0, nlink: 0, size: 0, addrs: vec![0; 13], blocks: HashMap::new() };
    let mut out = String::new();
    for line in input.lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        if w.is_empty() {
            continue;
        }
        match w[0] {
            "FS" => fs.bsize = w[1].parse().unwrap(),
            "INODE" => {
                let raw = from_hex(w[1]);
                fs.itype = le(&raw, 0, 2);
                fs.nlink = le(&raw, 6, 2);
                fs.size = le(&raw, 8, 4);
                for i in 0..13 {
                    fs.addrs[i] = le(&raw, 12 + 4 * i, 4);
                }
            }
            "BLOCK" => {
                let d = if w.len() > 2 { from_hex(w[2]) } else { Vec::new() };
                fs.blocks.insert(w[1].parse().unwrap(), d);
            }
            "STAT" => {
                let nm = match fs.itype {
                    0 => "free",
                    1 => "dir",
                    2 => "file",
                    3 => "dev",
                    _ => "unknown",
                };
                out.push_str(&format!("type={} nlink={} size={} blocks={}\n", nm, fs.nlink, fs.size, fs.count_blocks()));
            }
            "BMAP" => out.push_str(&format!("{}\n", blk_name(fs.bmap(w[1].parse().unwrap())))),
            "MAXSIZE" => out.push_str(&format!("{}\n", (NDIRECT + fs.bsize / 4) * fs.bsize)),
            "SPAN" => fs.span(w[1].parse().unwrap(), w[2].parse().unwrap(), &mut out),
            _ => {}
        }
    }
    print!("{}", out);
}