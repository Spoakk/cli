use std::fmt;

// #7c5af3
const R: u8 = 124;
const G: u8 = 90;
const B: u8 = 243;
// #6448c3 — dim
const RD: u8 = 100;
const GD: u8 = 72;
const BD: u8 = 195;

pub fn init() {
    #[cfg(windows)]
    let _ = crossterm::ansi_support::supports_ansi();
    #[cfg(windows)]
    {
        use crossterm::execute;
        use crossterm::style::Print;
        let _ = execute!(std::io::stdout(), Print(""));
    }
}

pub struct Colored {
    text: String,
    fg: Option<(u8, u8, u8)>,
    bold: bool,
    dim: bool,
    reset: bool,
}

impl Colored {
    fn new(text: impl Into<String>) -> Self {
        Self { text: text.into(), fg: None, bold: false, dim: false, reset: true }
    }
}

impl fmt::Display for Colored {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut codes = String::new();
        if self.bold { codes.push_str("\x1b[1m"); }
        if self.dim  { codes.push_str("\x1b[2m"); }
        if let Some((r, g, b)) = self.fg {
            codes.push_str(&format!("\x1b[38;2;{};{};{}m", r, g, b));
        }
        if self.reset {
            write!(f, "{}{}\x1b[0m", codes, self.text)
        } else {
            write!(f, "{}{}", codes, self.text)
        }
    }
}

pub fn spoak(s: impl Into<String>) -> Colored {
    Colored { fg: Some((R, G, B)), bold: true, ..Colored::new(s) }
}

pub fn spoak_dim(s: impl Into<String>) -> Colored {
    Colored { fg: Some((RD, GD, BD)), bold: false, ..Colored::new(s) }
}

pub fn bold(s: impl Into<String>) -> Colored {
    Colored { bold: true, ..Colored::new(s) }
}

pub fn dim(s: impl Into<String>) -> Colored {
    Colored { dim: true, ..Colored::new(s) }
}

pub fn green(s: impl Into<String>) -> Colored {
    Colored { fg: Some((80, 200, 120)), ..Colored::new(s) }
}

pub fn red(s: impl Into<String>) -> Colored {
    Colored { fg: Some((220, 80, 80)), ..Colored::new(s) }
}

pub fn yellow(s: impl Into<String>) -> Colored {
    Colored { fg: Some((220, 180, 60)), ..Colored::new(s) }
}

pub fn cyan(s: impl Into<String>) -> Colored {
    Colored { fg: Some((80, 220, 220)), ..Colored::new(s) }
}

pub fn magenta(s: impl Into<String>) -> Colored {
    Colored { fg: Some((220, 80, 220)), ..Colored::new(s) }
}

pub fn orange(s: impl Into<String>) -> Colored {
    Colored { fg: Some((255, 160, 50)), ..Colored::new(s) }
}

pub fn sky(s: impl Into<String>) -> Colored {
    Colored { fg: Some((80, 180, 255)), ..Colored::new(s) }
}

pub fn mint(s: impl Into<String>) -> Colored {
    Colored { fg: Some((80, 220, 160)), ..Colored::new(s) }
}

pub fn rose(s: impl Into<String>) -> Colored {
    Colored { fg: Some((255, 100, 140)), ..Colored::new(s) }
}

pub fn motd_to_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if c == '§' || c == '&' {
            i += 1;
            if i >= chars.len() { break; }
            let code = chars[i];

            if code == '#' && i + 7 <= chars.len() {
                let hex: String = chars[i..i+7].iter().collect();
                if let Some(ansi) = hex_to_ansi(&hex) {
                    out.push_str(&ansi);
                    i += 7;
                    continue;
                }
            }

            let ansi: &str = match code.to_ascii_lowercase() {
                '0' => "\x1b[38;2;0;0;0m",
                '1' => "\x1b[38;2;0;0;170m",
                '2' => "\x1b[38;2;0;170;0m",
                '3' => "\x1b[38;2;0;170;170m",
                '4' => "\x1b[38;2;170;0;0m",
                '5' => "\x1b[38;2;170;0;170m",
                '6' => "\x1b[38;2;255;170;0m",
                '7' => "\x1b[38;2;170;170;170m",
                '8' => "\x1b[38;2;85;85;85m",
                '9' => "\x1b[38;2;85;85;255m",
                'a' => "\x1b[38;2;85;255;85m",
                'b' => "\x1b[38;2;85;255;255m",
                'c' => "\x1b[38;2;255;85;85m",
                'd' => "\x1b[38;2;255;85;255m",
                'e' => "\x1b[38;2;255;255;85m",
                'f' => "\x1b[38;2;255;255;255m",
                'l' => "\x1b[1m",
                'o' => "\x1b[3m",
                'n' => "\x1b[4m",
                'm' => "\x1b[9m",
                'r' => "\x1b[0m",
                'k' => "",
                _   => "",
            };
            out.push_str(ansi);
        } else {
            out.push(c);
        }
        i += 1;
    }
    out.push_str("\x1b[0m");
    out
}

fn hex_to_ansi(hex: &str) -> Option<String> {
    // hex = "#RRGGBB"
    let h = hex.trim_start_matches('#');
    if h.len() != 6 { return None; }
    let r = u8::from_str_radix(&h[0..2], 16).ok()?;
    let g = u8::from_str_radix(&h[2..4], 16).ok()?;
    let b = u8::from_str_radix(&h[4..6], 16).ok()?;
    Some(format!("\x1b[38;2;{};{};{}m", r, g, b))
}
