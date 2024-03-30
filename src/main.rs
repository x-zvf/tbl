use std::cmp::Ordering;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
extern crate itertools;
use clap::Parser;
use itertools::Itertools;
use std::str::FromStr;

pub mod arguments;
use crate::arguments::*;

#[allow(unstable_name_collisions)]

fn main() {
    let args = Args::parse();
    let input: Box<dyn BufRead> = match args.file {
        Some(f) => Box::new(BufReader::new(File::open(f).expect("Failed to open file"))),
        None => Box::new(BufReader::new(io::stdin())),
    };

    let empty_string = "".to_string();

    let sort_comparator = |a: &Vec<_>, b: &Vec<_>| {
        let so = args
            .sort_by
            .clone() // I suck at Rust
            .unwrap_or(vec![SortOrder {
                column: 0,
                descending: false,
                numeric: false,
            }]);
        for cc in so {
            let ac = a.get(cc.column as usize).unwrap_or(&empty_string);
            let bc = b.get(cc.column as usize).unwrap_or(&empty_string);
            let cmpres = if cc.numeric {
                let an = i64::from_str(ac).unwrap_or(0);
                let bn = i64::from_str(bc).unwrap_or(0);
                if cc.descending {
                    bn.cmp(&an)
                } else {
                    an.cmp(&bn)
                }
            } else {
                if cc.descending {
                    bc.cmp(ac)
                } else {
                    ac.cmp(bc)
                }
            };
            if cmpres != Ordering::Equal {
                return cmpres;
            }
        }
        return Ordering::Equal;
    };

    let index_into = |s: &Vec<_>, idx| {
        if idx >= 0 {
            idx as usize
        } else {
            let ri = (s.len() as isize) + idx;
            if ri >= 0 {
                ri as usize
            } else {
                usize::max_value()
            }
        }
    };

    let map_columns = |mapping: &ColumnMapping, cols: &Vec<String>| match mapping {
        ColumnMapping::Index(i) => cols
            .get(index_into(&cols, *i as isize))
            .unwrap_or(&empty_string)
            .clone(),
        ColumnMapping::List(is, j) => is
            .iter()
            .map(|i| {
                cols.get(index_into(&cols, *i as isize))
                    .unwrap_or(&empty_string)
                    .clone()
            })
            .intersperse(j.to_string())
            .collect(),
        ColumnMapping::InfinteRange(f, j) => {
            let f = *f as isize;
            let si = if f > 0 {
                f
            } else {
                isize::max(cols.len() as isize + f, 0)
            } as usize;

            cols.iter()
                .dropping(si)
                .intersperse(j)
                .map(|x| x.clone())
                .collect()
        }
        ColumnMapping::Range(f, t, j) => {
            let f = *f as isize;
            let t = *t as isize;
            let si = if f > 0 {
                f
            } else {
                isize::max(cols.len() as isize + f, 0)
            } as usize;
            let ei = if f > 0 {
                f
            } else {
                isize::max(cols.len() as isize + t, 0)
            } as usize;
            cols[si..ei]
                .iter()
                .intersperse(j)
                .map(|x| x.clone())
                .collect()
        }
        ColumnMapping::InclusiveRange(f, t, j) => {
            let f = *f as isize;
            let t = *t as isize;
            let si = if f > 0 {
                f
            } else {
                isize::max(cols.len() as isize + f, 0)
            } as usize;
            let ei = if f > 0 {
                f
            } else {
                isize::max(cols.len() as isize + t, 0)
            } as usize;
            cols[si..=ei]
                .iter()
                .intersperse(j)
                .map(|x| x.clone())
                .collect()
        }
    };

    let mut input_columns = input
        .lines()
        .map(|l| l.unwrap())
        .map(|l| {
            l.split(&args.delimiter)
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
        })
        .collect::<Vec<_>>();

    if args.sort && !args.sort_by_output {
        input_columns.sort_by(|a, b| sort_comparator(a, b));
    }

    let mut output_columns = input_columns.iter_mut().map(|cols| {
        if let Some(cms) = &args.columns {
            cms.iter()
                .map(|cm| map_columns(cm, cols))
                .collect::<Vec<_>>()
        } else {
            cols.clone() // Skill issue: I can't for the life of make rust consume the values from
                         // input_columns
        }
    });
    let mut output_columns = if args.unique {
        output_columns.unique().collect::<Vec<Vec<_>>>()
    } else {
        output_columns.collect::<Vec<Vec<_>>>()
    };

    if args.sort && !args.sort_by_output {
        output_columns.sort_by(|a, b| sort_comparator(a, b));
    }
}
