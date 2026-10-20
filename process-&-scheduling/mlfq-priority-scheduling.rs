use std::collections::VecDeque;
use std::io::{self, Read};

// jobs: (pid, burst). Appends the schedule lines to `out`.
fn simulate(quanta: &[i64], allots: &[i64], boost: i64, jobs: &[(String, i64)], out: &mut String) {
    let levels = quanta.len();
    let n_jobs = jobs.len();
    let mut jobs_remaining: Vec<i64> = jobs.iter().map(|j| j.1).collect();
    let mut usage = vec![0i64; n_jobs];
    let mut queues: Vec<VecDeque<usize>> = vec![VecDeque::new(); levels];
    
    // All jobs start at level 0
    for i in 0..n_jobs {
        queues[0].push_back(i);
    }

    let mut clock = 0i64;
    let mut next_boost = boost;
    let mut done = 0;

    while done < n_jobs {
        // Head of the highest non-empty level
        let Some(l) = (0..levels).find(|&l| !queues[l].is_empty()) else { break; };
        let job = queues[l].pop_front().unwrap();

        let run = quanta[l].min(jobs_remaining[job]);

        out.push_str(format!("{} L{} {}\n", jobs[job].0, l, run).as_str());

        clock += run;
        jobs_remaining[job] -= run;
        usage[job] += run;

        if jobs_remaining[job] == 0 {
            done += 1;
            out.push_str(format!("{} done t={}\n", jobs[job].0, clock).as_str());
        } else if usage[job] >= allots[l] && l + 1 < levels {
            usage[job] = 0;
            queues[l + 1].push_back(job);
        } else {
            queues[l].push_back(job);
        }

        // Priority boost.
        if boost > 0 && clock >= next_boost {
            for l in 1..levels {
                while let Some(k) = queues[l].pop_front() {
                    queues[0].push_back(k);
                }
            }
            for k in 0..n_jobs {
                usage[k] = 0;
            }
            while next_boost <= clock {
                next_boost += boost;
            }
        }
    }
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut quanta: Vec<i64> = Vec::new();
    let mut allots: Vec<i64> = Vec::new();
    let mut boost: i64 = 0;
    let mut jobs: Vec<(String, i64)> = Vec::new();
    for line in input.lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        if w.is_empty() || w[0] == "LEVELS" {
            continue;
        }
        match w[0] {
            "QUANTUM" => quanta = w[1..].iter().map(|x| x.parse().unwrap()).collect(),
            "ALLOTMENT" => allots = w[1..].iter().map(|x| x.parse().unwrap()).collect(),
            "BOOST" => boost = w[1].parse().unwrap(),
            _ => {
                if w.len() >= 2 {
                    jobs.push((w[0].to_string(), w[1].parse().unwrap()));
                }
            }
        }
    }
    let mut out = String::new();
    simulate(&quanta, &allots, boost, &jobs, &mut out);
    print!("{}", out);
}
