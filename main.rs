use std::io::{self, Read};

const USER_MAX: u64 = 0x7FFF_FFFF_FFFF;

// Little-endian unsigned integer of n bytes at off.
fn u(data: &[u8], off: u64, n: usize) -> u64 {
    let mut v: u64 = 0;
    for i in (0..n).rev() {
        let pos = off.wrapping_add(i as u64);
        let b = if pos < data.len() as u64 { data[pos as usize] } else { 0 };
        v = v * 256 + b as u64;
    }
    v
}

fn flag_letters(f: u64) -> String {
    let mut s = String::new();
    s.push(if f & 4 != 0 { 'R' } else { '-' });
    s.push(if f & 2 != 0 { 'W' } else { '-' });
    s.push(if f & 1 != 0 { 'X' } else { '-' });
    s
}

struct Load {
    idx: u64,
    lo: u64,
    hi: u64,
    vaddr: u64,
    memsz: u64,
    flags: u64,
}

// Return the report lines for the ELF file `data`.
fn analyze(data: &[u8]) -> Vec<String> {
    let mut analysis: Vec<String> = Vec::new();
    let mut accepted_loads: Vec<Load> = Vec::new();
    let mut has_error = false;
    let mut total_pages = 0;
    
    // HEADER validation
    let l = data.len() as u64;

    if l < 64 {
        analysis.push("ERROR truncated header".to_string());
        return analysis;
    }

    let magic_numbers: Vec<u8> = data.iter().take(4).copied().collect();

    if magic_numbers != [0x7f, 0x45, 0x4c, 0x46] {
        analysis.push("ERROR not an ELF file".to_string());
        return analysis;
    }

    if data.get(4) != Some(&2) {
            let class = data.get(4).copied().unwrap_or_default();
            analysis.push(format!("ERROR unsupported class {}", class));
            return analysis;
    }    

    if data.get(5) != Some(&1) {
        let dat = data.get(5).copied().unwrap_or_default();
        analysis.push(format!("ERROR unsupported byte order {}", dat));
        return analysis;
    }

    let machine_bytes: [u8; 2] = [data[18], data[19]];
    let machine = u16::from_le_bytes(machine_bytes);
    if machine != 0x3e {
        analysis.push(format!("ERROR unsupported machine 0x{:04x}", machine));
        return analysis;
    }

    let type_h_bytes: [u8; 2] = [data[16], data[17]];
    let type_h = u16::from_le_bytes(type_h_bytes);
    if type_h != 2 && type_h != 3 {
        analysis.push(format!("ERROR unsupported type {}", type_h));
        return analysis;
    }

    let phentsize_bytes: [u8; 2] = [data[54], data[55]];
    let phentsize = u16::from_le_bytes(phentsize_bytes);
    if phentsize != 56 {
        analysis.push(format!("ERROR bad phentsize {}", phentsize));
        return analysis;
    }

    let phoff = u(data, 32, 8);
    let phnum_bytes: [u8; 2] = [data[56], data[57]];
    let phnum = u16::from_le_bytes(phnum_bytes) as u64;
    let header_end = phoff.wrapping_add(phnum.wrapping_mul(56));
    if header_end > l {
        analysis.push("ERROR program headers outside file".to_string());
        return analysis;
    }

    let entry = u(data, 24, 8);
    analysis.push(format!("ELF64 little-endian x86-64 type={} entry=0x{:x}", 
        if type_h == 2 { "EXEC" } else { "DYN" },
        entry,
    ));

    // PROGRAM HEADERS //
    for i in 0..phnum {
        // Start position of current program header
        let ph_start = phoff.wrapping_add(i.wrapping_mul(56));

        let p_type = u(data, ph_start, 4);
        let p_flags = u(data, ph_start + 4, 4);
        let p_offset = u(data, ph_start + 8, 8);
        let p_vaddr = u(data, ph_start + 16, 8);
        let p_filesz = u(data, ph_start + 32, 8);
        let p_memsz = u(data, ph_start + 40, 8);
        let p_align = u(data, ph_start + 48, 8);

        match p_type {
            3 => { //  INTERP
                let end_range = p_offset.wrapping_add(p_filesz);
                if end_range > l {
                    analysis.push(format!("ERROR segment {}: data outside file", i));
                    has_error = true;
                    break;
                }

                let start_idx = p_offset as usize;
                let end_idx = (p_offset + p_filesz) as usize;
                let slice = &data[start_idx..end_idx];
                let interp_bytes: Vec<u8> = slice.iter().copied().take_while(|&b| b != 0).collect();
                let interp_str = String::from_utf8_lossy(&interp_bytes);
                analysis.push(format!("INTERP {}", interp_str));
            },
            0x6474e551 => { // GNU_STACK
                let flag = flag_letters(p_flags);
                analysis.push(format!("STACK {}", flag));
            },
            1 => { // LOAD
                if p_filesz > p_memsz {
                    analysis.push(format!("ERROR segment {}: filesz exceeds memsz", i));
                    has_error = true;
                    break;
                }

                if p_offset.wrapping_add(p_filesz) > l {
                    analysis.push(format!("ERROR segment {}: data outside file", i));
                    has_error = true;
                    break;
                }

                if p_align > 1 && (p_vaddr % p_align) != (p_offset % p_align) {
                    analysis.push(format!("ERROR segment {}: misaligned", i));
                    has_error = true;
                    break;
                }

                let (vaddr_end, overflow) = p_vaddr.overflowing_add(p_memsz);
                if overflow || vaddr_end > USER_MAX {
                    analysis.push(format!("ERROR segment {}: outside user space", i));
                    has_error = true;
                    break;
                }

                if p_memsz > 0 {
                    let page_start = p_vaddr / 4096 * 4096;
                    let page_end = (p_vaddr.wrapping_add(p_memsz).wrapping_add(4095)) / 4096 * 4096;

                    let mut overlap = false;
                    let mut prev_idx = 0;

                    for load in &accepted_loads {
                        let l_start = load.vaddr / 4096 * 4096;
                        let l_end = (load.vaddr.wrapping_add(load.memsz).wrapping_add(4095)) / 4096 * 4096;

                        // Compare range intersections
                        if page_start < l_end && l_start < page_end {
                            overlap = true;
                            prev_idx = load.idx;
                            break;
                        }
                    }

                    if overlap {
                        analysis.push(format!("ERROR segment {}: overlaps segment {}", i, prev_idx));
                        has_error = true;
                        break;
                    }
                }

                // All validated
                let bss = p_memsz - p_filesz;
                analysis.push(format!(
                    "LOAD vaddr=0x{:x} memsz={} filesz={} offset=0x{:x} flags={} align=0x{:x} bss={}",
                    p_vaddr, p_memsz, p_filesz, p_offset, flag_letters(p_flags), p_align, bss
                ));

                // Calculte pages for this segment
                if p_memsz > 0 {
                    let pages_for_seg = (p_vaddr % 4096 + p_memsz + 4095) / 4096;
                    total_pages += pages_for_seg;
                }

                accepted_loads.push(Load {
                    idx: i,
                    lo: p_vaddr,
                    hi: p_vaddr.wrapping_add(p_memsz),
                    vaddr: p_vaddr,
                    memsz: p_memsz,
                    flags: p_flags,
                });

            },
            _ => {},
        }
    }

    if !has_error {
        let entry = u(data, 24, 8);
        let mut entry_in_exec = false;

        for load in &accepted_loads {
            // Flags include X: p_flags & 1 != 0
            if (load.flags & 1) != 0 && entry >= load.lo && entry < load.hi {
                entry_in_exec = true;
                break;
            }
        }

        if !entry_in_exec {
            analysis.push("ERROR entry not in an executable segment".to_string());
        } else {
            analysis.push(format!("PAGES {}", total_pages));
            analysis.push("OK".to_string());
        }
    }

    analysis

}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let digits: Vec<u8> = input.bytes().filter(|c| (*c as char).is_ascii_hexdigit()).collect();
    let mut data: Vec<u8> = Vec::new();
    let mut i = 0;
    while i + 1 < digits.len() {
        let s = String::from_utf8_lossy(&digits[i..i + 2]).to_string();
        data.push(u8::from_str_radix(&s, 16).unwrap());
        i += 2;
    }
    for l in analyze(&data) {
        println!("{}", l);
    }
}