use std::io::{self, Read};

const ROWS: [(u32, &'static str, &'static str); 4] = [
    (0x02, "1234567890-=", "!@#$%^&*()_+"),
    (0x10, "qwertyuiop[]", "QWERTYUIOP{}"),
    (0x1E, "asdfghjkl;'`", "ASDFGHJKL:\"~"),
    (0x2B, "\\zxcvbnm,./", "|ZXCVBNM<>?"),
];

// Character for a key make code, or None. Letters follow shift XOR caps; others follow shift.
fn key_char(code: u32, shifted: bool, caps: bool) -> Option<char> {

    if code == 0x39 {
        return Some(' ');
    }

    for &(start, normal, shifted_chars) in &ROWS {
        let index = code.wrapping_sub(start) as usize;
        if index >= normal.len() {
            continue;
        }

        let normal_char = normal.as_bytes()[index] as char;
        let use_shifted = if normal_char.is_ascii_alphabetic() {
            shifted ^ caps
        } else {
            shifted
        };
        return Some(if use_shifted {
            shifted_chars.as_bytes()[index] as char
        } else {
            normal_char
        });
    }

    None
}

// Run the whole byte stream through the state machine, appending console lines to `out`.
fn decode(codes: &[u32], out: &mut String) {
    let mut lshift = false;
    let mut rshift = false;
    let mut lctrl = false;
    let mut rctrl = false;
    let mut caps = false;
    let mut ext = false;
    let mut buf = String::new();

    for &code in codes {
        // If extended flag, clear
        if ext {
            // Reset ext flag
            ext = false;
            // Handle extended keys
            match code {
                0x1D => rctrl = true,
                0x9D => rctrl = false,
                0x1C => {
                    let shifted = lshift || rshift;
                    if let Some(space) = key_char(code, shifted, caps) {
                        buf.push(space);
                    }
                },
                _ => {}
            }
        } else if code == 0xE0 {
            ext = true;
        } else if code >= 0x80 {
            match code {
                0xAA => lshift = false,
                0xB6 => rshift = false,
                0x9D => lctrl = false,
                _ => {} // Ignored
            }
        } else {
            let ctrl = lctrl || rctrl;
            
            match code {
                0x2A => lshift = true,
                0x36 => rshift = true,
                0x1D => lctrl = true,
                0x3A => caps = !caps,
                0x0E => { buf.pop(); },
                0x1C => {
                    out.push_str(&buf);
                    out.push('\n');
                    buf.clear();
                }
                _ if ctrl => match code {
                    0x2E => {                      // Ctrl+C
                        out.push_str("^C\n");
                        buf.clear();
                    }
                    0x20 => {                      // Ctrl+D
                        if buf.is_empty() {
                            out.push_str("<EOF>\n");
                        }                          // non-empty buffer: ignored (check spec)
                    }
                    0x16 => buf.clear(),           // Ctrl+U
                    _ => {}                        // other Ctrl+key: ignored
                },
                _ => {
                    let shifted = lshift || rshift;
                    if let Some(c) = key_char(code, shifted, caps) {
                        buf.push(c);
                    }
                }
            }
        }
    }

    out.push_str(&buf);
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let codes: Vec<u32> = input.split_whitespace().map(|t| u32::from_str_radix(t, 16).unwrap()).collect();
    let mut out = String::new();
    decode(&codes, &mut out);
    print!("{}", out);
}
