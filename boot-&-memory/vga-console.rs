use std::io::{self, Read};

struct Screen {
    cols: usize,
    rows: usize,
    row: usize,
    col: usize,
    attr: u32,
    cells: Vec<Vec<u32>>, // attribute << 8 | character
}

impl Screen {
    fn blank_row(&self) -> Vec<u32> {
        vec![(self.attr << 8) | 0x20; self.cols]
    }

    fn reset(&mut self) {
        self.cells = (0..self.rows).map(|_| self.blank_row()).collect();
        self.row = 0;
        self.col = 0;
    }

    // Drop the top row, append a blank row, leave the cursor on the last row.
    fn scroll(&mut self) {
        if !self.cells.is_empty() {
            self.cells.remove(0);
            let new_blank_row = self.blank_row();
            self.cells.push(new_blank_row);
        }
        
        // Leaved the cursor on the last row.
        self.row = self.rows - 1;
    }

    fn newline(&mut self) {       
        self.row += 1;
        self.col = 0;

        if self.row >= self.rows {
            self.scroll();
            
        }
    }

    fn put_char(&mut self, ch: u8) {
        if self.col >= self.cols {
            self.newline();
        }

        self.cells[self.row][self.col] = self.attr << 8 | u32::from(ch);
        self.col += 1;
    }
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut s = Screen { cols: 0, rows: 0, row: 0, col: 0, attr: 7, cells: Vec::new() };
    let mut out = String::new();
    for line in input.lines() {
        let words: Vec<&str> = line.split_whitespace().collect();
        if words.is_empty() {
            continue;
        }
        match words[0] {
            "SCREEN" => {
                s.cols = words[1].parse().unwrap();
                s.rows = words[2].parse().unwrap();
                s.attr = 7;
                s.reset();
            }
            "ATTR" => s.attr = u32::from_str_radix(words[1], 16).unwrap(),
            "PRINT" => {
                let bytes = line.as_bytes();
                if bytes.len() > 6 {
                    for &b in &bytes[6..] {
                        s.put_char(b);
                    }
                }
            }
            "NEWLINE" => s.newline(),
            "CLEAR" => s.reset(),
            "SHOW" => {
                for r in 0..s.rows {
                    let text: String = s.cells[r].iter().map(|&c| (c & 0xFF) as u8 as char).collect();
                    out.push_str(text.trim_end_matches(' '));
                    out.push('\n');
                }
            }
            "CELL" => {
                let r: usize = words[1].parse().unwrap();
                let c: usize = words[2].parse().unwrap();
                out.push_str(&format!("0x{:04x}\n", s.cells[r][c]));
            }
            "ADDR" => {
                let r: usize = words[1].parse().unwrap();
                let c: usize = words[2].parse().unwrap();
                out.push_str(&format!("0x{:x}\n", 0xB8000 + 2 * (r * s.cols + c)));
            }
            _ => {}
        }
    }
    print!("{}", out);
}
