use std::io::{self, Read};

const HDR: u64 = 16;

#[derive(Debug)]
struct Block {
    addr: u64,
    size: u64,
    used: bool,
}

// Return the payload address for n bytes, or 0 if nothing fits (a payload is never at address 0
// because every block has a header in front of it).
fn alloc_block(blocks: &mut Vec<Block>, n: u64) -> u64 {
    let payload = 16 + n.next_multiple_of(16); // Rounded up + multiple of 16

    for i in 0..blocks.len() {
        // Check if processed block is FREE and is profit to be fitted
        if blocks[i].used || blocks[i].size < payload {
            continue;
        }
        let reminder = blocks[i].size - payload;
        let block_address = blocks[i].addr;
        blocks[i].used = true;

        // Splitting condition
        if reminder >= 32 {
            blocks[i].size = payload; // Update block size with needed size

            let new_block = Block {
                addr: block_address + payload,
                size: reminder,
                used: false
            };

            blocks.insert(i + 1, new_block);
        }
        return block_address + 16;
    }
    0    
}

// Free the block whose payload starts at addr. Returns true on success.
fn free_block(blocks: &mut Vec<Block>, addr: u64) -> bool {
    let Some(i) = blocks.iter().position(|b| b.used && b.addr + 16 == addr) else {
        return false;
    };
    blocks[i].used = false;

    // Merge with the following block
    if i + 1 < blocks.len() && !blocks[i + 1].used {
        let next = blocks.remove(i + 1);
        blocks[i].size += next.size;
    }
    // Merge with the preceding block
    if i > 0 && !blocks[i - 1].used {
        let cur = blocks.remove(i);
        blocks[i - 1].size += cur.size;
    }
    true
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut blocks: Vec<Block> = Vec::new();
    let mut out = String::new();
    for line in input.lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        if w.is_empty() {
            continue;
        }
        match w[0] {
            "HEAP" => {
                let start = u64::from_str_radix(w[1], 16).unwrap();
                let size = u64::from_str_radix(w[2], 16).unwrap();
                let lo = (start + 15) / 16 * 16;
                let hi = (start + size) / 16 * 16;
                blocks.clear();
                if hi > lo {
                    blocks.push(Block { addr: lo, size: hi - lo, used: false });
                }
            }
            "ALLOC" => {
                let n: u64 = w[1].parse().unwrap();
                if n == 0 {
                    out.push_str("NULL\n");
                } else {
                    let a = alloc_block(&mut blocks, n);
                    if a == 0 {
                        out.push_str("OOM\n");
                    } else {
                        out.push_str(&format!("0x{:x}\n", a));
                    }
                }
            }
            "FREE" => {
                let a = u64::from_str_radix(w[1], 16).unwrap();
                out.push_str(if free_block(&mut blocks, a) { "OK\n" } else { "ERROR\n" });
            }
            "DUMP" => {
                for b in blocks.iter() {
                    out.push_str(&format!("0x{:x} {} {}\n", b.addr, b.size, if b.used { "used" } else { "free" }));
                }
            }
            "STATS" => {
                let mut total = 0;
                let mut largest = 0;
                for b in blocks.iter() {
                    if !b.used {
                        total += b.size;
                        if b.size > largest {
                            largest = b.size;
                        }
                    }
                }
                out.push_str(&format!("free={} largest={} blocks={}\n", total, largest, blocks.len()));
            }
            _ => {}
        }
    }
    print!("{}", out);
}