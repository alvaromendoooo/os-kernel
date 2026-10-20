use std::collections::HashMap;
use std::io::{self, Read};

// (mode switches, address-space switches) for one transfer from domain x to domain y.
fn hop_cost(x: &str, y: &str) -> (i64, i64) {
    if x == y {
        (0, 0)
    } else if (x == "K") != (y == "K") {
        (1, 0)
    } else {
        (2, 1)
    }
}

// Architecture name for the chain's domains (in chain order).
fn classify(domains: &[String]) -> &'static str {
    let initial_domain = domains[0].as_str();

    let all_same = domains.iter().all(|d| d == initial_domain);
    if all_same {
        return "unikernel";
    }

    let mut counter_k_domain = 0;
    for service in 1..domains.len() {
        if domains[service] == "K".to_string() {
            counter_k_domain += 1;
        }
    }

    if counter_k_domain == domains.len() - 1 {
        return "monolithic";
    } else if counter_k_domain == 0 {
        return "microkernel";
    } else {
        return "hybrid";
    }
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut domain_of: HashMap<String, String> = HashMap::new();
    let mut mode_ns: i64 = 0;
    let mut as_ns: i64 = 0;
    let mut out = String::new();
    for line in input.lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        if w.is_empty() {
            continue;
        }
        if w[0] == "DOMAIN" && w.len() >= 3 {
            domain_of.insert(w[1].to_string(), w[2].to_string());
        } else if w[0] == "COST" && w.len() >= 3 {
            mode_ns = w[1].parse().unwrap();
            as_ns = w[2].parse().unwrap();
        } else if w[0] == "CHAIN" {
            let chain: Vec<&str> = w[1..].to_vec();
            let mut transfers: Vec<(&str, &str)> = Vec::new();
            for i in 0..chain.len() - 1 {
                transfers.push((chain[i], chain[i + 1]));
            }
            for i in (1..chain.len()).rev() {
                transfers.push((chain[i], chain[i - 1]));
            }
            let mut tm = 0;
            let mut ta = 0;
            for &(a, b) in transfers.iter() {
                let (m, s) = hop_cost(&domain_of[a], &domain_of[b]);
                tm += m;
                ta += s;
                out.push_str(&format!("{} -> {} mode={} as={}\n", a, b, m, s));
            }
            out.push_str(&format!("total mode={} as={} ns={}\n", tm, ta, tm * mode_ns + ta * as_ns));
            let doms: Vec<String> = chain.iter().map(|c| domain_of[*c].clone()).collect();
            out.push_str(&format!("arch={}\n", classify(&doms)));
        }
    }
    print!("{}", out);
}