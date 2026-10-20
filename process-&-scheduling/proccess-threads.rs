use std::collections::BTreeMap;
use std::io::{self, Read};

#[derive(Clone, Copy, PartialEq)]
enum State {
    Runnable,
    Zombie,
}

#[derive(Clone, Copy)]
struct Proc {
    parent: i64,
    state: State,
    status: i64,
}

fn state_name(s: State) -> &'static str {
    if s == State::Runnable { "RUNNABLE" } else { "ZOMBIE" }
}

fn children_of(procs: &BTreeMap<i64, Proc>, pid: i64) -> Vec<i64> {
    procs.iter().filter(|&(_, p)| p.parent == pid).map(|(&c, _)| c).collect()
}

// Create a child of `parent`; return its pid, or -1 if `parent` cannot fork.
fn do_fork(procs: &mut BTreeMap<i64, Proc>, next_pid: &mut i64, parent: i64) -> i64 {
    if let Some(proc) = procs.get(&parent) {
        if proc.state == State::Runnable {
            let pid = *next_pid;
            procs.insert(*next_pid, Proc { parent, state: State::Runnable, status: 0 });
            *next_pid += 1;
            return pid;
        }
    }

    -1
}

// Turn `pid` into a zombie and reparent ALL its children to init. Returns false on error.
fn do_exit(procs: &mut BTreeMap<i64, Proc>, pid: i64, status: i64) -> bool {
    if let Some(proc) = procs.get_mut(&pid) {
        if proc.state == State::Runnable {
            proc.state = State::Zombie;
            proc.status = status;

            for (_, proc) in procs.iter_mut() {
                if proc.parent == pid {
                    proc.parent = 1;
                }
            }

            return true;
        }
    }

    false
}

// Returns (0 error | 1 reaped | 2 block | 3 echild, child, status).
fn do_wait(procs: &mut BTreeMap<i64, Proc>, pid: i64) -> (u8, i64, i64) {
    // Pid must exist and be Runnable
    match procs.get(&pid) {
        Some(p) if p.state == State::Runnable => {}
        _ => return (0,0,0),
    }

    let mut has_childrens = false;
    let mut zombie: Option<i64> = None;

    // BTreeMap iterates in ascending order so first zombie child found is the lowest
    for (&child, p) in procs.iter() {
        if p.parent != pid {
            continue;
        }

        has_childrens = true;
        if p.state == State::Zombie {
            zombie = Some(child);
            break;
        }
    }

    if let Some(child) = zombie {
        let reaped = procs.remove(&child).unwrap();
        (1, child, reaped.status)
    } else if has_childrens {
        (2, 0, 0)
    } else {
        (3, 0, 0)
    }

}

fn tree(procs: &BTreeMap<i64, Proc>, pid: i64, depth: usize, out: &mut String) {
    out.push_str(&format!("{}{} {}\n", "  ".repeat(depth), pid, state_name(procs[&pid].state)));
    for c in children_of(procs, pid) {
        tree(procs, c, depth + 1, out);
    }
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut procs: BTreeMap<i64, Proc> = BTreeMap::new();
    procs.insert(1, Proc { parent: 0, state: State::Runnable, status: 0 });
    let mut next_pid: i64 = 2;
    let mut out = String::new();
    for line in input.lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        if w.is_empty() {
            continue;
        }
        match w[0] {
            "FORK" => {
                let c = do_fork(&mut procs, &mut next_pid, w[1].parse().unwrap());
                if c < 0 { out.push_str("ERROR\n"); } else { out.push_str(&format!("{}\n", c)); }
            }
            "EXIT" => {
                let pid: i64 = w[1].parse().unwrap();
                let status: i64 = w[2].parse().unwrap();
                let init_live = match procs.get(&1) {
                    Some(p) => p.state == State::Runnable,
                    None => false,
                };
                if pid == 1 && init_live {
                    out.push_str("PANIC init exiting\n");
                    break;
                }
                out.push_str(if do_exit(&mut procs, pid, status) { "OK\n" } else { "ERROR\n" });
            }
            "WAIT" => {
                let (r, child, status) = do_wait(&mut procs, w[1].parse().unwrap());
                match r {
                    0 => out.push_str("ERROR\n"),
                    1 => out.push_str(&format!("REAPED {} {}\n", child, status)),
                    2 => out.push_str("BLOCK\n"),
                    _ => out.push_str("ECHILD\n"),
                }
            }
            "PS" => {
                for (pid, p) in procs.iter() {
                    out.push_str(&format!("{} {} {}\n", pid, p.parent, state_name(p.state)));
                }
            }
            "TREE" => {
                let pid: i64 = w[1].parse().unwrap();
                if procs.contains_key(&pid) {
                    tree(&procs, pid, 0, &mut out);
                } else {
                    out.push_str("NO_SUCH_PID\n");
                }
            }
            _ => {}
        }
    }
    print!("{}", out);
}