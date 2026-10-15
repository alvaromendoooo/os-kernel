use std::collections::HashMap;
use std::io::{self, Read};

struct Cpu {
    regs: HashMap<String, u32>,
    mem: HashMap<u32, u32>,
}

impl Cpu {
    fn reg(&self, name: &str) -> u32 {
        *self.regs.get(name).unwrap_or(&0)
    }

    fn rd(&self, a: u32) -> u32 {
        *self.mem.get(&a).unwrap_or(&0)
    }

    fn push(&mut self, v: u32) {
        let esp = self.reg("esp").wrapping_sub(4);
        self.regs.insert("esp".to_string(), esp);
        self.mem.insert(esp, v);
    }

    fn pop(&mut self) -> u32 {
        let esp = self.reg("esp");
        let v = self.rd(esp);
        self.regs.insert("esp".to_string(), esp.wrapping_add(4));
        v
    }

    // Run the context-switch routine. Returns the old context pointer (esp after the pushes).
    fn swtch(&mut self, new_ctx: u32, ret_eip: u32) -> u32 {
        // Push reg_eip
        self.push(ret_eip);

        // Push other registers
        for reg in ["ebp", "ebx", "esi", "edi"] {
            let reg_val = self.reg(reg);
            self.push(reg_val);
        }

        // Save the esp as the running context process's context pointer
        let old_ctx = self.reg("esp");

        // Set esp to the target's context pointer
        self.regs.insert("esp".to_string(), new_ctx);

        // Pop registers
        for reg in ["edi", "esi", "ebx", "ebp"] {
            let v = self.pop();
            self.regs.insert(reg.to_string(), v);
        }

        let eip = self.pop();
        self.regs.insert("eip".to_string(), eip);

        old_ctx
    }
}

fn h8(v: u32) -> String {
    format!("0x{:08x}", v)
}

fn hx(s: &str) -> u32 {
    u32::from_str_radix(s, 16).unwrap()
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut cpu = Cpu { regs: HashMap::new(), mem: HashMap::new() };
    for r in ["eax", "ebx", "esi", "edi", "ebp", "esp", "eip"].iter() {
        cpu.regs.insert(r.to_string(), 0);
    }
    let mut ctx: HashMap<String, u32> = HashMap::new();
    let mut current = String::new();
    let mut out = String::new();
    for line in input.lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        if w.is_empty() {
            continue;
        }
        match w[0] {
            "BOOT" => {
                current = w[1].to_string();
                cpu.regs.insert("esp".to_string(), hx(w[2]));
            }
            "PROC" => {
                let base = hx(w[2]).wrapping_sub(20);
                for i in 0..4u32 {
                    cpu.mem.insert(base.wrapping_add(4 * i), 0);
                }
                cpu.mem.insert(base.wrapping_add(16), hx(w[3]));
                ctx.insert(w[1].to_string(), base);
            }
            "SET" => {
                if cpu.regs.contains_key(w[1]) {
                    cpu.regs.insert(w[1].to_string(), hx(w[2]));
                }
            }
            "SWITCH" => {
                let target = w[1].to_string();
                let new_ctx = match ctx.get(&target) {
                    Some(&c) if target != current => c,
                    _ => {
                        out.push_str("ERROR\n");
                        continue;
                    }
                };
                let old = current.clone();
                let saved = cpu.swtch(new_ctx, hx(w[2]));
                ctx.insert(old.clone(), saved);
                ctx.remove(&target);
                current = target.clone();
                out.push_str(&format!("{} -> {} eip={} esp={}\n", old, target, h8(cpu.reg("eip")), h8(cpu.reg("esp"))));
            }
            "REGS" => {
                let parts: Vec<String> = ["eax", "ebx", "esi", "edi", "ebp", "esp", "eip"]
                    .iter()
                    .map(|r| format!("{}={}", r, h8(cpu.reg(r))))
                    .collect();
                out.push_str(&parts.join(" "));
                out.push('\n');
            }
            "PEEK" => out.push_str(&format!("{}\n", h8(cpu.rd(hx(w[1]))))),
            "CTX" => match ctx.get(w[1]) {
                None => out.push_str("ERROR\n"),
                Some(&p) => {
                    let names = ["edi", "esi", "ebx", "ebp", "eip"];
                    let parts: Vec<String> = (0..5u32).map(|i| format!("{}={}", names[i as usize], h8(cpu.rd(p.wrapping_add(4 * i))))).collect();
                    out.push_str(&parts.join(" "));
                    out.push('\n');
                }
            },
            _ => {}
        }
    }
    print!("{}", out);
}
