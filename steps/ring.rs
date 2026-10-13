use std::io::{self, Read};

const PRIVILEGED: [&str; 15] = ["hlt", "cli", "sti", "in", "out", "lgdt", "lidt", "ltr", "wrmsr", "rdmsr",
    "invlpg", "wbinvd", "mov_cr0", "mov_cr3", "mov_cr4"];

// Apply one event to the CPU. Returns (new ring, faulted).
fn step(ring: u32, words: &[&str]) -> (u32, bool) {
    let mut ring_result = ring;

    match words {
        ["SYSCALL"] | ["INTR", ..] => ring_result = 0,
        ["SYSRET"] => {
            if ring_result == 0 {
                ring_result = 3;
            } else {
                ring_result = 0;
                return (ring_result, true);
            }
        },
        ["IRET"] if ring_result == 0 => {
            ring_result = 3;
        }
        ["EXEC", mnemonic] if ring_result == 3 && PRIVILEGED.contains(mnemonic) => {
            // Fault
            ring_result = 0;
            return (ring_result, true);
        }
        _ => {}
    }

    (ring_result, false)
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