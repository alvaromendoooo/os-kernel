use std::io::{self, Read};

// Pack base (32 bits), limit (20), access (8) and flags (4) into a 64-bit descriptor.
fn encode_segment(base: u64, limit: u64, access: u64, flags: u64) -> u64 {
    let mut desc: u64 = 0;

    let limit_low = limit & 0xFFFF; // 16 bits mask [15:0]
    let limit_high = (limit >> 16) & 0xFF; // 8 bits mask [23:16]

    let base_low = base & 0xFFFFFF; // 24 bits mask [23:0]
    let base_high = (base >> 24) & 0xFF; // 8 bits mask [31:24]

    // Limit bits [15:0] at bits [15:0]
    desc |= limit_low;
    // Bits 16-31: Base [15:0]
    desc |= (base_low & 0xFFFF) << 16;
    // Bits 32-39: Base [23:16]
    desc |= ((base_low >> 16) & 0xFF) << 32;
    // Access at [47:40]
    desc |= (access & 0xFF) << 40;
    // Limit bits [19:16] at bits [51:48]
    desc |= limit_high << 48;
    // Flags at [55:52]
    desc |= (flags & 0x0F) << 52;
    // Base bits [31:16] at bits [63:56]
    desc |= base_high << 56;

    desc
}

// Pack a 32-bit IDT gate.
fn encode_gate(offset: u64, selector: u64, attr: u64) -> u64 {
    let mut gate: u64 = 0;

    let offset_low = offset & 0xFFFF; // [15:0]
    let offset_high = (offset >> 16) & 0xFFFF; // [31:16]

    // Offset bits [15:0] at bits [15:0]
    gate |= offset_low;
    // Selector 16 bits at bits [31:16]
    gate |= (selector & 0xFFFF) << 16;
    // Zero bits at [39:32]

    // Attr 8 bits at bits [47:40]
    gate |= (attr & 0xFF) << 40;
    // Offset bits [31:16] at bits [63:48]
    gate |= offset_high << 48;

    gate
}

fn decode_segment(d: u64) -> String {
    let mut decoded_segment = String::new();

    // Build BASE bits
    let base_low = (d >> 16) & 0xFFFF;
    let base_mid = (d >> 32) & 0xFF;
    let base_high = (d >> 56) & 0xFF;

    let base = base_low | (base_mid << 16) | (base_high << 24);
    decoded_segment.push_str(format!("base=0x{:08x} ", base).as_str());

    // Build LIMIT bits
    let limit_low = d & 0xFFFF;
    let limit_high = (d >> 48) & 0x0F;

    let limit = limit_low | (limit_high << 16);
    decoded_segment.push_str(format!("limit=0x{:05x} ", limit).as_str());

    // Build ACCESS bits
    let access = (d >> 40) & 0xFF;
    decoded_segment.push_str(format!("access=0x{:02x} ", access).as_str());

    // Build FLAGS bits
    let flags = (d >> 52) & 0x0F;
    decoded_segment.push_str(format!("flags=0x{:01x} ", flags).as_str());

    // Business logic
    let kind = if (access & 0x10) == 0 {
        "system"
    } else if (access & 0x08) != 0 {
        "code"
    } else {
        "data"
    };
    decoded_segment.push_str(format!("kind={} ", kind).as_str());

    let dpl = (access >> 5) & 0x03;
    decoded_segment.push_str(format!("dpl={} ", dpl).as_str());

    let present = (access >> 7) & 0x01;
    decoded_segment.push_str(format!("present={} ", present).as_str());

    let bytes = if (flags & 0x08) != 0 {
        (limit + 1) * 4096
    } else {
        limit + 1
    };
    decoded_segment.push_str(format!("bytes={}", bytes).as_str());

    decoded_segment

}

fn decode_gate(d: u64) -> String {
     let mut decoded_gate = String::new();

    // Build OFFSET bits
    let offset_low = d & 0xFFFF;
    let offset_high = (d >> 48) & 0xFFFF;

    let offset = offset_low | (offset_high << 16);
    decoded_gate.push_str(format!("offset=0x{:08x} ", offset).as_str());

    // Build SELECTOR bits
    let selector = (d >> 16) & 0xFFFF;
    decoded_gate.push_str(format!("selector=0x{:04x} ", selector).as_str());

    // Business logic
    let attr = (d >> 40) & 0xFF;
    let type_gt = match attr & 0x0F {
        0xE => "interrupt32",
        0xF => "trap32",
        0x5 => "task",
        _ => "other"
    };
    decoded_gate.push_str(format!("type={} ", type_gt).as_str());

    let dpl = (attr >> 5) & 0x03;
    decoded_gate.push_str(format!("dpl={} ", dpl).as_str());

    let present = (attr >> 7) & 0x01;
    decoded_gate.push_str(format!("present={}", present).as_str());

    decoded_gate
}

fn hx(s: &str) -> u64 {
    u64::from_str_radix(s, 16).unwrap()
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut out = String::new();
    for line in input.lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        if w.is_empty() {
            continue;
        }
        if w[0] == "GDT" && w.len() >= 5 {
            out.push_str(&format!("0x{:016x}\n", encode_segment(hx(w[1]), hx(w[2]), hx(w[3]), hx(w[4]))));
        } else if w[0] == "IDT" && w.len() >= 4 {
            out.push_str(&format!("0x{:016x}\n", encode_gate(hx(w[1]), hx(w[2]), hx(w[3]))));
        } else if w[0] == "DECODE" && w.len() >= 2 {
            out.push_str(&decode_segment(hx(w[1])));
            out.push('\n');
        } else if w[0] == "GATE" && w.len() >= 2 {
            out.push_str(&decode_gate(hx(w[1])));
            out.push('\n');
        } else if w[0] == "SELECTOR" && w.len() >= 4 {
            out.push_str(&format!("0x{:04x}\n", ((hx(w[1]) << 3) | (hx(w[2]) << 2) | hx(w[3])) & 0xFFFF));
        }
    }
    print!("{}", out);
}
