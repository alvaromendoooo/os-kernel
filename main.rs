use std::collections::HashMap;
use std::io::{self, Read};

struct Pic {
    irr: [u32; 2],
    isr: [u32; 2],
    imr: [u32; 2],
    base: [u32; 2],
    names: HashMap<u32, String>,
}

fn bit_of(v: u32, n: u32) -> bool {
    (v >> n) & 1 == 1
}

impl Pic {
    // Winning line of one chip under the delivery rule, or None.
    fn best(&self, chip: usize) -> Option<u32> {
        for line in 0..8 {
            if bit_of(self.isr[chip], line) {
                break;
            }
            if bit_of(self.irr[chip], line) && !bit_of(self.imr[chip], line) {
                return Some(line);
            }
        }
        None
    }

    // Deliver everything the rule allows right now.
    fn poll(&mut self, out: &mut String) {
        loop {
            let slave_line = self.best(1);
            let had_cascade = bit_of(self.irr[0], 2);
            if slave_line.is_some() {
                self.irr[0] |= 1 << 2;
            }
            let master_line = self.best(0);
            if !had_cascade {
                self.irr[0] &= !(1 << 2);
            }

            if master_line == Some(2) && slave_line.is_some() {
                self.irr[1] &= !(1 << slave_line.unwrap());
                self.isr[1] |= 1 << slave_line.unwrap();
                self.isr[0] |= 1 << 2;
                let line = slave_line.unwrap();
                let irq = 8 + line;
                let vector = self.base[1] + line;
                match self.names.get(&vector) {
                    Some(name) => out.push_str(&format!("IRQ{} vec=0x{:02x} {}\n", irq, vector, name)),
                    None => out.push_str(&format!("IRQ{} vec=0x{:02x} unhandled\n", irq, vector)),
                }
                continue;
            }

            let Some(line) = master_line else { break };
            self.irr[0] &= !(1 << line);
            self.isr[0] |= 1 << line;
            let irq = line;
            let vector = self.base[0] + line;
            match self.names.get(&vector) {
                Some(name) => out.push_str(&format!("IRQ{} vec=0x{:02x} {}\n", irq, vector, name)),
                None => out.push_str(&format!("IRQ{} vec=0x{:02x} unhandled\n", irq, vector)),
            }
        }
    }
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut p = Pic { irr: [0, 0], isr: [0, 0], imr: [0xFF, 0xFF], base: [0x08, 0x70], names: HashMap::new() };
    let mut out = String::new();
    for line in input.lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        if w.is_empty() {
            continue;
        }
        let c = w[0];
        if c == "REMAP" && w.len() >= 3 {
            p.base[0] = u32::from_str_radix(w[1], 16).unwrap();
            p.base[1] = u32::from_str_radix(w[2], 16).unwrap();
        } else if c == "IDT" && w.len() >= 3 {
            p.names.insert(u32::from_str_radix(w[1], 16).unwrap(), w[2].to_string());
        } else if (c == "MASK" || c == "UNMASK" || c == "RAISE") && w.len() >= 2 {
            let irq: u32 = w[1].parse().unwrap();
            let chip = (irq / 8) as usize;
            let ln = irq % 8;
            if c == "MASK" {
                p.imr[chip] |= 1u32 << ln;
            } else if c == "UNMASK" {
                p.imr[chip] &= !(1u32 << ln);
                p.poll(&mut out);
            } else if irq != 2 {
                p.irr[chip] |= 1u32 << ln;
                p.poll(&mut out);
            }
        } else if c == "EOI" && w.len() >= 2 {
            let chip = if w[1] == "M" { 0 } else { 1 };
            for ln in 0..8u32 {
                if bit_of(p.isr[chip], ln) {
                    p.isr[chip] &= !(1u32 << ln);
                    break;
                }
            }
            p.poll(&mut out);
        } else if c == "STATE" {
            out.push_str(&format!("m irr={:02x} isr={:02x} imr={:02x} s irr={:02x} isr={:02x} imr={:02x}\n",
                p.irr[0], p.isr[0], p.imr[0], p.irr[1], p.isr[1], p.imr[1]));
        }
    }
    print!("{}", out);
}
