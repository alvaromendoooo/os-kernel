use std::io::{self, Read};

const PRIVILEGED: [&str; 15] = ["hlt", "cli", "sti", "in", "out", "lgdt", "lidt", "ltr", "wrmsr", "rdmsr",
    "invlpg", "wbinvd", "mov_cr0", "mov_cr3", "mov_cr4"];

// Apply one event to the CPU. Returns (new ring, faulted).
fn step(ring: u32, words: &[&str]) -> (u32, bool) {
    // TODO: SYSCALL / INTR enter ring 0, SYSRET / IRET return to ring 3, EXEC of a privileged
    // mnemonic faults in ring 3. Keep the ring unchanged for everything else.
    let _ = (words, PRIVILEGED.len());
    (ring, false)
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut ring: u32 = 3;
    let mut out = String::new();
    for line in input.lines() {
        let words: Vec<&str> = line.split_whitespace().collect();
        if words.is_empty() {
            continue;
        }
        if words[0] == "PROBE" {
            out.push_str(&format!("ring {}\n", ring));
            continue;
        }
        let (r, faulted) = step(ring, &words);
        ring = r;
        if faulted {
            out.push_str("#GP\n");
        }
    }
    print!("{}", out);
}
