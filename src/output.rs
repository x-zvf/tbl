use itertools::Itertools;

use crate::arguments::*;

fn align_and_trim(s: &String, align: &Alignment, w: usize, spec: &WidthSpecifier) -> String {
    let br = if s.chars().count() <= w {
        s.clone()
    } else {
        match spec {
            WidthSpecifier::Indeterminate => panic!("s.chars().count() should be <= w"),
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

fn replace_with_if<FA, FB>(cond: &bool, s: &String, fa: FA, fb: FB) -> String
where
    FA: FnMut(char) -> char,
    FB: FnMut(char) -> char,
{
    if *cond {
        s.chars().map(fa).collect()
    } else {
        s.chars().map(fb).collect()
    }
}

fn calculate_column_widths(args: &Args, header: &Vec<String>, data: &[Vec<String>]) -> Vec<usize> {
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
    column_widths
}

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

    let column_widths = calculate_column_widths(&args, &header, data);
    let header_text = format_row(&args, &column_widths, &header);
    let header_underline = replace_with_if(
        &args.ascii,
        &header_text,
        |c| {
            // FIXME: if a '|' is contained in the title text, it results in + being
            // printed at the wrong place. Users shouldn't do that though
            match c {
                '|' => '+',
                _ => '-',
            }
        },
        |c| match c {
            '\u{2502}' => '\u{253c}',
            _ => '\u{2500}',
        },
    );
    let header_overline = replace_with_if(
        &args.ascii,
        &header_underline,
        |c| c,
        |c| match c {
            '\u{253c}' => '\u{252c}',
            x => x,
        },
    );
    let footer_line = replace_with_if(
        &args.ascii,
        &header_underline,
        |c| c,
        |c| match c {
            '\u{253c}' => '\u{2534}',
            x => x,
        },
    );
    let print_a_u = |al, ar, ul, ur, s| {
        if args.ascii {
            println!("{}{}{}", al, s, ar);
        } else {
            println!("{}{}{}", ul, s, ur);
        }
    };
    let print_f_a_u = |al, ar, ul, ur, s| {
        if args.decoration == Decoration::Full {
            print_a_u(al, ar, ul, ur, s);
        } else {
            println!("{}", s);
        }
    };

    if args.decoration == Decoration::Full {
        print_a_u("+", "+", "\u{250c}", "\u{2510}", header_overline);
    }
    print_f_a_u("|", "|", "\u{2502}", "\u{2502}", header_text);

    if args.decoration != Decoration::None {
        print_f_a_u("+", "+", "\u{251c}", "\u{2524}", header_underline);
    }
    for row in data {
        let t = format_row(&args, &column_widths, row);
        print_f_a_u("|", "|", "\u{2502}", "\u{2502}", t);
    }

    if args.decoration == Decoration::Full {
        print_a_u("+", "+", "\u{2514}", "\u{2518}", footer_line);
    }
}
