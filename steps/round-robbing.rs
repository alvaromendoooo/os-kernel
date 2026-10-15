use std::collections::VecDeque;
use std::io::{self, Read};

struct Proc {
    name: String,
    arrival: i64,
    burst: i64,
}

// Returns (timeline of (start, end, index or -1 for idle), first start per process, finish per process).
fn simulate(quantum: i64, procs: &[Proc]) -> (Vec<(i64, i64, i64)>, Vec<i64>, Vec<i64>) {
    let n = procs.len();

    // Indices sorted by arrival; sort_by_key is stable, so ties keep input order.
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by_key(|&i| procs[i].arrival);

    let mut remaining: Vec<i64> = procs.iter().map(|p| p.burst).collect();
    let mut first = vec![-1i64; n];
    let mut finish = vec![0i64; n];
    let mut timeline: Vec<(i64, i64, i64)> = Vec::new();

    let mut queue: VecDeque<usize> = VecDeque::new();
    let mut next = 0; // next index into `order` waiting to be admitted
    let mut clock = 0i64;
    let mut done = 0;

    while done < n {
        // Admit everything that has arrived by now.
        while next < n && procs[order[next]].arrival <= clock {
            queue.push_back(order[next]);
            next += 1;
        }

        // Nothing ready: idle until the next arrival (one must exist, since done < n).
        if queue.is_empty() {
            let t = procs[order[next]].arrival;
            timeline.push((clock, t, -1));
            clock = t;
            continue;
        }

        let i = queue.pop_front().unwrap();
        if first[i] < 0 {
            first[i] = clock;
        }
        let run = quantum.min(remaining[i]);
        timeline.push((clock, clock + run, i as i64));
        clock += run;
        remaining[i] -= run;

        // Admit arrivals up to the new clock BEFORE requeueing the preempted process.
        while next < n && procs[order[next]].arrival <= clock {
            queue.push_back(order[next]);
            next += 1;
        }

        if remaining[i] > 0 {
            queue.push_back(i);
        } else {
            finish[i] = clock;
            done += 1;
        }
    }

    (timeline, first, finish)
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut quantum: i64 = 1;
    let mut procs: Vec<Proc> = Vec::new();
    for line in input.lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        if w.is_empty() {
            continue;
        }
        if w[0] == "QUANTUM" && w.len() >= 2 {
            quantum = w[1].parse().unwrap();
        } else if w[0] == "PROC" && w.len() >= 4 {
            procs.push(Proc { name: w[1].to_string(), arrival: w[2].parse().unwrap(), burst: w[3].parse().unwrap() });
        }
    }
    let (slices, first_start, finish) = simulate(quantum, &procs);
    let mut out = String::new();
    for s in slices.iter() {
        let name = if s.2 < 0 { "idle" } else { procs[s.2 as usize].name.as_str() };
        out.push_str(&format!("{}-{} {}\n", s.0, s.1, name));
    }
    let mut total = 0;
    for (i, p) in procs.iter().enumerate() {
        let turn = finish[i] - p.arrival;
        let wait = turn - p.burst;
        total += wait;
        out.push_str(&format!("{} done={} turn={} wait={} resp={}\n", p.name, finish[i], turn, wait, first_start[i] - p.arrival));
    }
    out.push_str(&format!("total_wait={}\n", total));
    print!("{}", out);
}