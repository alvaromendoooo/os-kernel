use std::collections::HashMap;
use std::io::{self, Read};

struct Entry {
    ino: u32,
    name: String,
}

fn decode_entries(data: &[u8]) -> Vec<Entry> {
    let mut out = Vec::new();
    let mut off = 0;
    while off + 16 <= data.len() {
        let ino = data[off] as u32 | ((data[off + 1] as u32) << 8);
        if ino != 0 {
            let raw = &data[off + 2..off + 16];
            let mut n = 0;
            while n < 14 && raw[n] != 0 {
                n += 1;
            }
            out.push(Entry { ino: ino, name: String::from_utf8_lossy(&raw[..n]).to_string() });
        }
        off += 16;
    }
    out
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

// Inode number of `name` in directory `dir_ino`, or 0.
fn lookup(dirs: &HashMap<u32, Vec<u8>>, dir_ino: u32, name: &str) -> u32 {
    if dir_ino == 0 {
        return 0;
    }

    let Some(content) = dirs.get(&dir_ino) else {
        return 0;
    };

    for entry in decode_entries(content) {
        if entry.name == name {
            return entry.ino;
        }
    }

    0
}

// Returns an inode number (> 0) or, if negative, an error: -1 EINVAL, -2 ENAMETOOLONG, -3 ENOTDIR, -4 ENOENT.
fn resolve(dirs: &HashMap<u32, Vec<u8>>, path: &str) -> i64 {
    if !path.starts_with("/") {
        return -1;
    }
    let mut current_ino = 1; // root ino allways 1
    let components: Vec<&str> = path.split("/").filter(|&p| !p.is_empty()).collect();

    for pat in &components {
        if pat.len() > 14 {
            return -2;
        }

        // Check if the current inode is a directory
        if current_ino != 1 && !dirs.contains_key(&current_ino) {
            return -3;
        } 

        let next_ino = lookup(dirs, current_ino, pat);

        if next_ino == 0 {
            return -4;
        }

        // Forward to the next inode
        current_ino = next_ino;
    }

    if path.ends_with('/') && !components.is_empty() {
        // Check if final directory is valid
        if current_ino != 1 && !dirs.contains_key(&current_ino) {
            return -3;
        }
    }
    
    current_ino as i64
}

fn err_name(e: i64) -> &'static str {
    match e {
        -1 => "EINVAL",
        -2 => "ENAMETOOLONG",
        -3 => "ENOTDIR",
        _ => "ENOENT",
    }
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut dirs: HashMap<u32, Vec<u8>> = HashMap::new();
    let mut out = String::new();
    for line in input.lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        if w.is_empty() {
            continue;
        }
        match w[0] {
            "DIR" => {
                let data = if w.len() > 2 { from_hex(w[2]) } else { Vec::new() };
                dirs.insert(w[1].parse().unwrap(), data);
            }
            "RESOLVE" => {
                let r = resolve(&dirs, w[1]);
                if r > 0 {
                    out.push_str(&format!("{}\n", r));
                } else {
                    out.push_str(&format!("{}\n", err_name(r)));
                }
            }
            "LS" => {
                let r = resolve(&dirs, w[1]);
                if r <= 0 {
                    out.push_str(&format!("{}\n", err_name(r)));
                    continue;
                }
                match dirs.get(&(r as u32)) {
                    None => out.push_str("ENOTDIR\n"),
                    Some(data) => {
                        let es = decode_entries(data);
                        if es.is_empty() {
                            out.push_str("(empty)\n");
                        }
                        for e in es.iter() {
                            out.push_str(&format!("{} {}\n", e.ino, e.name));
                        }
                    }
                }
            }
            _ => {}
        }
    }
    print!("{}", out);
}