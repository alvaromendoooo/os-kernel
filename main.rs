use std::io::{self, Read};

const PAGE: u64 = 4096;
const FREE: u8 = 0;
const USED: u8 = 1;
const RESV: u8 = 2;

// Size the map, mark frames wholly inside usable ranges free, then reserve every frame touched by
// a non-usable range or the kernel image.
fn build_map(regions: &[(u64, u64, u32)], kernels: &[(u64, u64)]) -> Vec<u8> {
    let max_regios = regions
        .iter()
        .map(|&(base, length, _)| base.saturating_add(length))
        .max()
        .unwrap_or(0);

    let total_frames = (max_regios / PAGE) as usize;

    let mut map = vec![1u8; total_frames];

    // Free frames that completly lies inside a type-1 range
    // Rule: round start_up (base + PAGE - 1) / PAGE and end_dow (base + length) / PAGE.
    for &(base, length, type_reg) in regions {
        if type_reg == 1 {
            let start_frame = ((base + PAGE - 1) / PAGE) as usize;
            let end_frame = ((base + length) / PAGE) as usize;

            for i in start_frame..end_frame {
                if i < total_frames {
                    map[i] = FREE;
                }
            }
        }
    }

    // Reserve frames that touch non used ranges
    // Rule: round start_down (base / PAGE) and end_up (base + length + PAGE - 1) / PAGE
    for &(base, length, type_reg) in regions {
        if length == 0 { continue; } // Zero-length ranges are ignored.
        if type_reg != 1 {
            let start_frame = (base / PAGE) as usize;
            let end_frame = ((base + length + PAGE - 1) / PAGE) as usize;

            for i in start_frame..end_frame {
                if i < total_frames { 
                    map[i] = USED;
                }
            } 
        }
    }

    // Reserve frames touching kernel images
    for &(k_start, k_end) in kernels {
        if k_start >= k_end { continue; }
        let start_frame = (k_start / PAGE) as usize;
        let end_frame = ((k_end + PAGE - 1) / PAGE) as usize;

        for i in start_frame..end_frame {
            if i < total_frames {
                map[i] = USED;
            }
        }
    }

    map
}

// Lowest run of k free frames: mark it allocated and return its first frame, or -1.
fn alloc_frames(state: &mut Vec<u8>, k: u64) -> i64 {
    let mut sequential_counter = 0;
    let mut start_idx = 0;
    let mut found = false;

    for i in 0..state.len() {
        if state[i] == 0 {
            if sequential_counter == 0 {
                start_idx = i; // Save where the k frame trace starts
            }
            sequential_counter += 1;

            if sequential_counter == k {
                found = true;
                break;
            }
        } else {
            sequential_counter = 0; // Reset
        }
    }

    if !found {
        return -1; // OOM
    }

    // Mark found frames as reserved (USED)
    for i in start_idx..(start_idx + k as usize) {
        state[i] = USED;
    }

    (start_idx as u64 * PAGE) as i64
}

// Free k allocated frames starting at addr. Returns true on success.
fn free_frames(state: &mut Vec<u8>, addr: u64, k: u64) -> bool {
    if (addr % PAGE) != 0 || k == 0 {
        return false;
    }

    let start_idx = (addr / PAGE) as usize;
    let end_idx = match start_idx.checked_add(k as usize) { // Take action of overflow
        Some(val) => val,
        None => return false
    };

    // Check size overflow
    if end_idx > state.len() {
        return false;
    }

    // If a single frame of the range provided is FREE or RESV, no more computing, error
    for i in start_idx..end_idx {
        if state[i] != USED {
            return false;
        }
    }

    // All frames of the range provided are USED, we can free them
    for i in start_idx..end_idx {
        state[i] = FREE;
    }

    true    
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut regions: Vec<(u64, u64, u32)> = Vec::new();
    let mut kernels: Vec<(u64, u64)> = Vec::new();
    let mut state: Vec<u8> = Vec::new();
    let mut built = false;
    let mut out = String::new();
    for line in input.lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        if w.is_empty() {
            continue;
        }
        if w[0] == "E820" && w.len() >= 4 {
            regions.push((u64::from_str_radix(w[1], 16).unwrap(), u64::from_str_radix(w[2], 16).unwrap(), w[3].parse().unwrap()));
            continue;
        }
        if w[0] == "KERNEL" && w.len() >= 3 {
            kernels.push((u64::from_str_radix(w[1], 16).unwrap(), u64::from_str_radix(w[2], 16).unwrap()));
            continue;
        }
        if !built {
            state = build_map(&regions, &kernels);
            built = true;
        }
        match w[0] {
            "ALLOC" => {
                let k: u64 = if w.len() > 1 { w[1].parse().unwrap() } else { 1 };
                let f = alloc_frames(&mut state, k);
                if f < 0 {
                    out.push_str("OOM\n");
                } else {
                    out.push_str(&format!("0x{:x}\n", f));
                }
            }
            "FREE" => {
                let addr = u64::from_str_radix(w[1], 16).unwrap();
                let k: u64 = if w.len() > 2 { w[2].parse().unwrap() } else { 1 };
                out.push_str(if free_frames(&mut state, addr, k) { "OK\n" } else { "ERROR\n" });
            }
            "FREECOUNT" => {
                let c = state.iter().filter(|&&s| s == FREE).count();
                out.push_str(&format!("{}\n", c));
            }
            "BITS" => {
                let first = u64::from_str_radix(w[1], 16).unwrap();
                let count: u64 = w[2].parse().unwrap();
                for f in first..(first + count) {
                    out.push(if (f as usize) < state.len() && state[f as usize] == FREE { '0' } else { '1' });
                }
                out.push('\n');
            }
            _ => {}
        }
    }
    print!("{}", out);
}
