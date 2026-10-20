use std::collections::HashMap;
use std::io::{self, Read};

struct Cpu {
    gates: HashMap<u32, u64>,
    tss_esp0: u32,
}

impl Cpu {
    // Software interrupt. Returns the output line.
    fn int_gate(&self, cpl: u32, vector: u32, esp: u32, eip: u32, eflags: u32) -> String {
        let err = (vector << 3) | 2;
        let Some(&gate) = self.gates.get(&vector) else {
            return format!("#NP err=0x{:04x}", err);
        };

        let attr = ((gate >> 40) & 0xff) as u32;
        let gate_type = attr & 0x0f;
        let gate_dpl = (attr >> 5) & 3;
        let present = attr & 0x80 != 0;
        let selector = ((gate >> 16) & 0xffff) as u32;
        let target_cpl = selector & 3;

        if !present {
            return format!("#NP err=0x{:04x}", err);
        }
        if (gate_type != 0x0e && gate_type != 0x0f) || cpl > gate_dpl {
            return format!("#GP err=0x{:04x}", err);
        }
        if target_cpl > cpl {
            return format!("#GP err=0x{:04x}", selector & !7);
        }

        let target_eip = ((gate & 0xffff) | ((gate >> 32) & 0xffff0000)) as u32;
        let switched = target_cpl < cpl;
        let new_esp = if switched {
            self.tss_esp0.wrapping_sub(20)
        } else {
            esp.wrapping_sub(12)
        };
        let new_eflags = if gate_type == 0x0e {
            eflags & !(1 << 9)
        } else {
            eflags
        };
        let old_cs = if cpl == 3 { 0x1b } else { 0x08 };
        let old_ss = if cpl == 3 { 0x23 } else { 0x10 };
        let frame = if switched {
            format!(
                "0x{:08x},0x{:08x},0x{:08x},0x{:08x},0x{:08x}",
                eip,
                old_cs,
                eflags,
                esp,
                old_ss
            )
        } else {
            format!("0x{:08x},0x{:08x},0x{:08x}", eip, old_cs, eflags)
        };
        format!(
            "OK cpl={}->{} cs=0x{:04x} eip=0x{:08x} esp=0x{:08x} if={} frame={}",
            cpl,
            target_cpl,
            selector,
            target_eip,
            new_esp,
            (new_eflags >> 9) & 1,
            frame
        )
    }
}

fn do_iret(cpl: u32, cs: u32) -> String {
    if cs == 0 || (cs & 3) < cpl {
        return format!("#GP err=0x{:04x}", (cs & 0xfffc));
    } else {
        format!("OK cpl={}->{}", cpl, (cs & 3))
    }
}

fn load_data(cpl: u32, selector: u32, dpl: u32) -> String {
    if selector != 0 {
        if !(cpl.max(selector & 3) <= dpl) {
            return  format!("#GP err=0x{:04x}", (selector & 0xfffc));
        }
    }

    format!("OK")
}

fn hx(s: &str) -> u32 {
    u32::from_str_radix(s, 16).unwrap()
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut cpu = Cpu { gates: HashMap::new(), tss_esp0: 0 };
    let mut out = String::new();
    for line in input.lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        if w.is_empty() {
            continue;
        }
        match w[0] {
            "TSS" => cpu.tss_esp0 = hx(w[1]),
            "IDT" => {
                cpu.gates.insert(w[1].parse().unwrap(), u64::from_str_radix(w[2], 16).unwrap());
            }
            "INT" => {
                let s = cpu.int_gate(w[1].parse().unwrap(), w[2].parse().unwrap(), hx(w[3]), hx(w[4]), hx(w[5]));
                out.push_str(&s);
                out.push('\n');
            }
            "IRET" => {
                out.push_str(&do_iret(w[1].parse().unwrap(), hx(w[2])));
                out.push('\n');
            }
            "DATA" => {
                out.push_str(&load_data(w[1].parse().unwrap(), hx(w[2]), w[3].parse().unwrap()));
                out.push('\n');
            }
            _ => {}
        }
    }
    print!("{}", out);
}
