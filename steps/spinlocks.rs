use std::io::{self, Read};

struct Thread {
    name: String,
    prog: Vec<String>,
    pc: usize,
    r: i64,
}

struct State {
    counter: i64,
    spins: i64,
    lock: i64, // index of the holder, or -1
}

// Run one instruction of threads[i]. Returns 0 normally, 1 for a REL by a non-holder (error message
// wanted), 2 for a relock panic.
fn execute(threads: &mut Vec<Thread>, i: usize, state: &mut State) -> u8 {
    let Some(thread) = threads.get_mut(i) else { return 0 };
    let inst = match thread.prog.get(thread.pc) {
        Some(s) => s.clone(),
        None => return 0
    };

    match inst.as_str() {
        "LOAD" => { thread.r = state.counter; thread.pc += 1; },
        "INC"  => { thread.r += 1; thread.pc += 1; },
        "STORE" => { state.counter = thread.r; thread.pc += 1; },
        "ACQ" => {
            if state.lock == -1 { // Lock is free
                state.lock = i as i64;
                thread.pc += 1;
            } else if state.lock == i as i64 {
                return 2; // relock panic.
            } else {
                state.spins += 1;
            }
        },
        "REL" => {
            thread.pc += 1;
            if state.lock == i as i64 { // If thread holds the lock
                state.lock = -1; // Lock becomes free
            } else {
                return 1;
            }
        }
        _ => {},
    }
    0
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut state = State { counter: 0, spins: 0, lock: -1 };
    let mut threads: Vec<Thread> = Vec::new();
    let mut schedule: Vec<String> = Vec::new();
    for line in input.lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        if w.is_empty() {
            continue;
        }
        match w[0] {
            "COUNTER" => state.counter = w[1].parse().unwrap(),
            "THREAD" => threads.push(Thread { name: w[1].to_string(), prog: w[2..].iter().map(|s| s.to_string()).collect(), pc: 0, r: 0 }),
            "SCHEDULE" => {
                for s in w[1..].iter() {
                    schedule.push(s.to_string());
                }
            }
            _ => {}
        }
    }
    let mut out = String::new();
    for name in schedule.iter() {
        let mut ti = 0;
        for (j, t) in threads.iter().enumerate() {
            if &t.name == name {
                ti = j;
            }
        }
        if threads[ti].pc >= threads[ti].prog.len() {
            continue;
        }
        let r = execute(&mut threads, ti, &mut state);
        if r == 2 {
            out.push_str(&format!("PANIC {} relock\n", name));
            print!("{}", out);
            return;
        }
        if r == 1 {
            out.push_str(&format!("ERR {} REL\n", name));
        }
    }
    let unfinished = threads.iter().filter(|t| t.pc < t.prog.len()).count();
    let holder = if state.lock < 0 { "free".to_string() } else { threads[state.lock as usize].name.clone() };
    out.push_str(&format!("counter={}\nspins={}\nlock={}\nunfinished={}\n", state.counter, state.spins, holder, unfinished));
    print!("{}", out);
}