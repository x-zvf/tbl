use itertools::Itertools;

use crate::arguments::*;

fn align_and_trim(s: &String, align: &Alignment, w: usize, spec: &WidthSpecifier) -> String {
    let br = if s.len() <= w {
        s.clone()
    } else {
        match spec {
            WidthSpecifier::Indeterminate => panic!("s.len() should be <= w"),
            WidthSpecifier::Break(_) => s[..w].to_string(),
            WidthSpecifier::Cut(_) => s[..w].to_string(),
            WidthSpecifier::Ellipsis(_) => s[..w - 3].to_string() + "...",
        }
    };
    match align {
        Alignment::Left => format!("{: <width$}", br, width = w),
        Alignment::Right => format!("{: >width$}", br, width = w),
        Alignment::Center => format!("{: ^width$}", br, width = w),
    }
}

pub fn format_row(args: &Args, column_widths: &Vec<usize>, row: &Vec<String>) -> String {
    if let Some(ref layout) = args.layout {
        layout
            .delimiters
            .iter()
            .map(|d| {
                if args.ascii {
                    d.clone()
                } else {
                    d.chars()
                        .map(|c| if c == '|' { '\u{2502}' } else { c })
                        .collect()
                }
            })
            .interleave(
                row.iter()
                    .zip(column_widths)
                    .enumerate()
                    .map(|(i, (h, c))| {
                        let fw = args.fixed_width.clone().unwrap_or(vec![]);
                        let fw = fw.get(i).unwrap_or(&WidthSpecifier::Indeterminate);
                        align_and_trim(
                            h,
                            layout.column_align.get(i).unwrap_or(&Alignment::Left),
                            *c,
                            fw,
                        )
                    }),
            )
            .collect()
    } else {
        row.iter()
            .enumerate()
            .map(|(i, h)| {
                align_and_trim(
                    h,
                    &Alignment::Left,
                    column_widths[i],
                    &WidthSpecifier::Indeterminate,
                )
            })
            .join(if args.ascii { &"|" } else { &"\u{2502}" })
    }
}

/*
fn line_adjacent(to: &String, ascii: bool, below: bool) -> String {
    let mut res = "".to_string();
    if to.len() < 2 {
        res = "\u{2500}".repeat(to.len());
    }

    if ascii {
        res = res
            .chars()
            .map(|c| match c {
                '\u{2500}' => '-',
                '\u{2502}' => '|',
                '\u{250c}' | '\u{2510}' | '\u{2514}' | '\u{2518}' | '\u{251c}' | '\u{2524}' => '+',
                x => x,
            })
            .collect();
    }
    res
}*/

pub fn display(args: Args, rows: &Vec<Vec<String>>) {
    if rows.len() == 0 {
        return;
    }
    let (header, data) = match args.headers {
        Some(ref h) => (
            h.iter()
                .take(rows.get(1).unwrap_or(h).len())
                .map(|x| x.clone())
                .collect(),
            &rows[..],
        ),
        None => (rows[0].clone(), &rows[1..]),
    };

    let mut column_widths = header
        .iter()
        .map(|h| h.chars().count())
        .collect::<Vec<usize>>();
    for row in data {
        let nc = usize::max(column_widths.len(), row.len());
        while column_widths.len() <= nc {
            column_widths.push(0)
        }
        for (i, c) in row.iter().enumerate() {
            column_widths[i] = usize::max(column_widths[i], c.chars().count());
        }
    }
    if let Some(ref fws) = args.fixed_width {
        for (i, fw) in fws.iter().enumerate() {
            let mut discard: usize = 0;
            let w: &mut usize = column_widths.get_mut(i).unwrap_or(&mut discard);
            *w = match fw {
                WidthSpecifier::Indeterminate => *w,
                WidthSpecifier::Break(i) => *i,
                WidthSpecifier::Cut(i) => *i,
                WidthSpecifier::Ellipsis(i) => *i,
            };
        }
    }

    let header_text = format_row(&args, &column_widths, &header);
    let header_underline: String = header_text
        .chars()
        .map(|c| {
            if args.ascii {
                match c {
                    '|' => '+',
                    // FIXME: if a '|' is contained in the title text, it results in + being
                    // printed at the wrong place. Users shouldn't do that though
                    _ => '-',
                }
            } else {
                match c {
                    '\u{2502}' => '\u{253c}',
                    _ => '\u{2500}',
                }
            }
        })
        .collect();
    let header_overline = if args.ascii {
        header_underline.clone()
    } else {
        header_underline
            .chars()
            .map(|c| match c {
                '\u{253c}' => '\u{252c}',
                x => x,
            })
            .collect()
    };
    let footer_line = if args.ascii {
        header_underline.clone()
    } else {
        header_underline
            .chars()
            .map(|c| match c {
                '\u{253c}' => '\u{2534}',
                x => x,
            })
            .collect()
    };
    //let header_underline_len = header_underline.chars().count();
    if args.decoration == Decoration::Full {
        if args.ascii {
            println!("+{}+", header_overline);
        } else {
            println!("\u{250C}{}\u{2510}", header_overline);
        }
    }
    if args.decoration == Decoration::Full {
        if args.ascii {
            println!("|{}|", header_text);
        } else {
            println!("\u{2502}{}\u{2502}", header_text);
        }
    } else {
        println!("{}", header_text);
    }

    if args.decoration != Decoration::None {
        if args.decoration == Decoration::Full {
            if args.ascii {
                println!("+{}+", header_underline);
            } else {
                println!("\u{251c}{}\u{2524}", header_underline);
            }
        } else {
            println!("{}", header_underline);
        }
    }
    for row in data {
        let t = format_row(&args, &column_widths, row);
        if args.decoration == Decoration::Full {
            if args.ascii {
                println!("|{}|", t);
            } else {
                println!("\u{2502}{}\u{2502}", t);
            }
        } else {
            println!("{}", t);
        }
    }

    if args.decoration != Decoration::None {
        if args.decoration == Decoration::Full {
            if args.ascii {
                println!("+{}+", footer_line);
            } else {
                println!("\u{2514}{}\u{2518}", footer_line);
            }
        }
    }
}
