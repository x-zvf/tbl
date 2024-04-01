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
    if let Some(ref layout) = args.alignment {
        layout
            .delimiters
            .iter()
            .map(|x| x.clone())
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
            .join(&"|")
    }
}

pub fn display(args: Args, rows: &Vec<Vec<String>>) {
    if rows.len() == 0 {
        return;
    }
    let (header, data) = match args.headers {
        Some(ref h) => (h.clone(), &rows[..]),
        None => (rows[0].clone(), &rows[1..]),
    };

    let mut column_widths = header.iter().map(|h| h.len()).collect::<Vec<usize>>();
    for row in data {
        let nc = usize::max(column_widths.len(), row.len());
        while column_widths.len() <= nc {
            column_widths.push(0)
        }
        for (i, c) in row.iter().enumerate() {
            column_widths[i] = usize::max(column_widths[i], c.len());
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
        .map(|c| match c {
            '|' => '+',
            _ => '-',
        })
        .collect();
    let print_top_bot = || {
        if args.decoration == Decoration::Full {
            if header.len() >= 3 {
                println!(
                    "{}{}{}",
                    if header_underline[0..1] == *"+" {
                        '+'
                    } else {
                        '-'
                    },
                    "-".repeat(header_underline.len() - 2),
                    if header_underline[header_underline.len() - 1..header_underline.len()] == *"+"
                    {
                        '+'
                    } else {
                        '-'
                    }
                )
            } else {
                println!("{}", "-".repeat(header_underline.len()))
            }
        }
    };
    let print_underline = || {
        if args.decoration != Decoration::None {
            println!("{}", header_underline);
        }
    };

    print_top_bot();
    println!("{}", header_text);
    print_underline();
    for row in data {
        println!("{}", format_row(&args, &column_widths, row));
    }

    print_top_bot();
}