use std::collections::HashMap;
use std::io::{self, Read};

struct Mmu {
    mem: HashMap<u32, u32>,
    cr3: u32,
}

impl Mmu {
    fn rd(&self, a: u32) -> u32 {
        *self.mem.get(&a).unwrap_or(&0)
    }

    // Walk the tables. Ok(physical address) or Err(page-fault error code).
    fn translate(&mut self, vaddr: u32, write: bool, user: bool) -> Result<u32, u32> {
        let pde_mem = (self.cr3 & 0xFFFFF000) + 4 * (vaddr >> 22);
        let mut pde = self.rd(pde_mem);

        let p_pde = (pde & 0x01) != 0;
        let rw_pde = (pde & 0x02) != 0; // R/W - Bit 2
        let us_pde = (pde & 0x04) != 0; // U/S - Bit 4
        let ps_pde = (pde & 0x80) != 0; // Page Size - Bit 7

        let mut base_error = 0;
        if write { base_error |= 2; }
        if user { base_error |= 4; }

        // Check Present PDE
        if !p_pde {
            return Err(base_error);
        }

        if (user && !us_pde) || (write && !rw_pde) {
            return Err(base_error | 1);
        }

        if ps_pde {
            let paddr = (pde & 0xFFC00000) | (vaddr & 0x3FFFFF);

            // Update hardware bits PDE
            pde |= 0x20; // Set A
            if write {
                pde |= 0x40; // Set B
            }
            
            self.mem.insert(pde_mem, pde);

            return Ok(paddr);
        }
        // If no 4MiB, 4KiB pages.
        let pte_addr = (pde & 0xFFFFF000) + 4 * ((vaddr >> 12) & 0x3FF);
        let mut pte = self.rd(pte_addr);

        let p_pte = (pte & 0x01) != 0;
        let rw_pte = (pte & 0x02) != 0;
        let us_pte = (pte & 0x04) != 0;

        if !p_pte {
            return Err(base_error);
        }

        if (user && !us_pte) || (write && !rw_pte) {
            return Err(base_error | 1);
        }

        let paddr = (pte & 0xFFFFF000) | (vaddr & 0xFFF);

        // Update bits hardware:
        // Set A in PTE + PDE
        // Set D in PTE if write allowed
        pde |= 0x20;
        self.mem.insert(pde_mem, pde);

        pte |= 0x20;
        if write {
            pte |= 0x40;
        }
        self.mem.insert(pte_addr, pte);

        Ok(paddr)
    }
}

fn hex(s: &str) -> u32 {
    u32::from_str_radix(s, 16).unwrap()
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut mmu = Mmu { mem: HashMap::new(), cr3: 0 };
    let mut out = String::new();
    for line in input.lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        if w.is_empty() {
            continue;
        }
        match w[0] {
            "CR3" => mmu.cr3 = hex(w[1]) & 0xFFFFF000,
            "MEM" => {
                mmu.mem.insert(hex(w[1]), hex(w[2]));
            }
            "PEEK" => out.push_str(&format!("0x{:08x}\n", mmu.rd(hex(w[1])))),
            "ACCESS" => {
                let v = hex(w[1]);
                match mmu.translate(v, w[2] == "W", w[3] == "U") {
                    Ok(p) => out.push_str(&format!("0x{:08x}\n", p)),
                    Err(e) => out.push_str(&format!("#PF cr2=0x{:08x} err=0x{:x}\n", v, e)),
                }
            }
            _ => {}
        }
    }
    print!("{}", out);
}