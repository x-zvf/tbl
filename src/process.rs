use std::cmp::Ordering;
use std::io::BufRead;
extern crate itertools;
use itertools::Itertools;
use std::str::FromStr;

use crate::arguments::*;

fn get_sort_comparator(
    sort_by: &Option<Vec<SortOrder>>,
) -> impl Fn(&Vec<String>, &Vec<String>) -> std::cmp::Ordering + '_ {
    |a: &Vec<String>, b: &Vec<String>| {
        let empty_string: String = "".to_string();
        let so = sort_by
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
    }
}

fn absolute_index(le: usize, idx: &isize) -> usize {
    if *idx >= 0 {
        *idx as usize
    } else {
        let ri = (le as isize) + *idx;
        if ri >= 0 {
            ri as usize
        } else {
            usize::max_value()
        }
    }
}

fn absolute_index_slice(le: usize, idx: &isize) -> usize {
    if *idx > 0 {
        *idx as usize
    } else {
        isize::max(le as isize + *idx, 0) as usize
    }
}

#[allow(unstable_name_collisions)]
fn map_column(mapping: &ColumnMapping, cols: &Vec<String>) -> String {
    let empty_string: String = "".to_string();
    match mapping {
        ColumnMapping::Index(i) => cols
            .get(absolute_index(cols.len(), i))
            .unwrap_or(&empty_string)
            .clone(),
        ColumnMapping::List(is, j) => is
            .iter()
            .map(|i| {
                cols.get(absolute_index(cols.len(), i))
                    .unwrap_or(&empty_string)
            })
            .intersperse(&j)
            .map(|x| x.clone())
            .collect(),

        ColumnMapping::InfinteRange(f, j) => cols
            .iter()
            .dropping(absolute_index_slice(cols.len(), f))
            .intersperse(&j)
            .map(|x| x.clone())
            .collect(),

        ColumnMapping::Range(f, t, j) => cols
            [absolute_index_slice(cols.len(), f)..absolute_index_slice(cols.len(), t)]
            .iter()
            .intersperse(&j)
            .map(|x| x.clone())
            .collect(),
        ColumnMapping::InclusiveRange(f, t, j) => cols
            [absolute_index_slice(cols.len(), f)..=absolute_index_slice(cols.len(), t)]
            .iter()
            .intersperse(&j)
            .map(|x| x.clone())
            .collect(),
    }
}

fn get_number_of_columns(args: &Args, input_length: usize) -> usize {
    let mut widths = vec![input_length];
    if let Some(h) = &args.headers {
        widths.push(h.len());
    };
    if let Some(c) = &args.columns {
        widths.push(c.len());
    };
    if let Some(a) = &args.alignment {
        widths.push(a.column_align.len());
    };
    if let Some(w) = &args.fixed_width {
        widths.push(w.len());
    };
    *widths.iter().min().unwrap()
}

#[allow(unstable_name_collisions)]
pub fn process(args: &Args, input: Box<dyn BufRead>) -> Vec<Vec<String>> {
    let sort_comparator = get_sort_comparator(&args.sort_by);

    let mut input_matrix = input
        .lines()
        .map(|l| l.unwrap())
        .map(|l| {
            l.split(&args.delimiter)
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
        })
        .collect::<Vec<_>>();

    if args.sort && !args.sort_by_output {
        if args.sort_ignore_first {
            input_matrix[1..].sort_by(&sort_comparator);
        } else {
            input_matrix.sort_by(&sort_comparator);
        }
    }

    let n_columns = get_number_of_columns(
        &args,
        input_matrix.iter().map(|x| x.len()).max().unwrap_or(0),
    );

    let output_matrix = input_matrix.iter().map(|cols| {
        if let Some(cms) = &args.columns {
            cms.iter()
                .map(|cm| map_column(cm, cols))
                .collect::<Vec<_>>()
        } else {
            cols.clone()
        }
    });

    let mut output_matrix: Vec<Vec<_>> = if args.unique {
        output_matrix.unique().collect()
    } else {
        output_matrix.collect()
    };

    if args.sort && args.sort_by_output {
        if args.sort_ignore_first {
            output_matrix[1..].sort_by(&sort_comparator);
        } else {
            output_matrix.sort_by(&sort_comparator);
        }
    }

    output_matrix.iter_mut().for_each(|r| {
        r.drain(usize::min(n_columns, r.len())..);
        while r.len() < n_columns {
            r.push("".to_string())
        }
    });
    output_matrix
}
