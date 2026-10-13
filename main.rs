use std::{io::{self, Read}};

fn le(data: &[u8], offset: usize, size: usize) -> u64 {
    let mut v: u64 = 0;
    for i in (0..size).rev() {
        v = v * 256 + data[offset + i] as u64;
    }
    v
}

fn type_name(t: u8) -> &'static str {
    match t {
        0x05 => "extended",
        0x07 => "ntfs",
        0x0B => "fat32",
        0x0C => "fat32-lba",
        0x82 => "swap",
        0x83 => "linux",
        0xEE => "gpt-protective",
        0xEF => "efi-system",
        _ => "unknown",
    }
}

// Return the report lines for a 512-byte boot sector.
fn describe(sector: &[u8]) -> Vec<String> {
    let mut report_lines: Vec<String> = Vec::new();
    
    // 1. Signature handler
    let signature_le = le(sector, 510, 2);

    if signature_le != 0xAA55 {
        report_lines.push(format!("signature BAD 0x{:04x}", signature_le));
        report_lines.push("boot none".to_string());

        return report_lines;
    } else {
        report_lines.push("signature OK".to_string());
    }

    // Disk id handler.
    let disk_id  = le(sector, 440, 4);
    report_lines.push(format!("disk id 0x{:08x}", disk_id));

    // Partition entries handler.
    let mut current_partition_offset = 446;
    let final_partition_offset = 494;
    let mut i = 1;
    let mut boot_active_partition_counter = 0;
    let mut partition_booted = 0;

    while current_partition_offset <= final_partition_offset {
        let type_offset = current_partition_offset + 4;
        let partition_offset = sector[type_offset];

        // If type is 0x00, partition is empty
        if partition_offset == 0x00 {
            current_partition_offset += 16;
            i += 1;
            continue;
        }

        let boot_flag = sector.get(current_partition_offset).copied();

        if let Some(flag_val) = boot_flag {
            if flag_val != 0x00 && flag_val != 0x80 {
                report_lines.push(format!("P{} INVALID flag 0x{:02X}", i, flag_val));
            } else {
                let active_status = if flag_val == 0x80 { "active" } else {"inactive"};
                let name = type_name(partition_offset);
                let start = le(sector, current_partition_offset + 8, 4); // LBA
                let sectors = le(sector, current_partition_offset + 12, 4); // Sector's lecture
                let last = if sectors == 0 {
                    String::from("none")
                } else {
                    format!("{}", start + sectors - 1)
                };

                report_lines.push(
                    format!("P{} {} type=0x{:02X} {} lba={} sectors={} last={}", 
                    i, active_status, partition_offset, name, start, sectors, last)
                );

                if flag_val == 0x80 {
                    boot_active_partition_counter +=1;
                    partition_booted = i;
                }

            }
        }

        current_partition_offset += 16;
        i += 1;
    }

    // Boot partition handler
    if boot_active_partition_counter >= 2 {
        report_lines.push("boot ERROR multiple active partitions".to_string());
    } else if boot_active_partition_counter == 0 {
        report_lines.push("boot none".to_string());
    } else {
        report_lines.push(format!("boot P{}", partition_booted));
    }

    report_lines
}

fn main() {
    let mut raw = Vec::new();
    io::stdin().read_to_end(&mut raw).unwrap();
    let mut sector = vec![0u8; 512];
    let mut n = 0;
    let mut hi: i32 = -1;
    for &c in raw.iter() {
        let v: i32 = match c {
            b'0'..=b'9' => (c - b'0') as i32,
            b'a'..=b'f' => (c - b'a') as i32 + 10,
            b'A'..=b'F' => (c - b'A') as i32 + 10,
            _ => -1,
        };
        if v < 0 {
            continue;
        }
        if hi < 0 {
            hi = v;
            continue;
        }
        if n < 512 {
            sector[n] = (hi * 16 + v) as u8;
            n += 1;
        }
        hi = -1;
    }
    for l in describe(&sector) {
        println!("{}", l);
    }
}