use std::cmp::Ordering;
use std::io::BufRead;
extern crate itertools;
use itertools::Itertools;
use std::process::exit;
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
            let ac = a
                .get(absolute_index(a.len(), &cc.column))
                .unwrap_or(&empty_string);
            let bc = b
                .get(absolute_index(b.len(), &cc.column))
                .unwrap_or(&empty_string);
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
    if *idx >= 0 {
        usize::min(*idx as usize, le)
    } else {
        isize::max(le as isize + *idx, 0) as usize
    }
}
fn absolute_index_inclusive_slice(le: usize, idx: &isize) -> usize {
    if *idx >= 0 {
        usize::min(*idx as usize, le - 1)
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
            [absolute_index_slice(cols.len(), f)..=absolute_index_inclusive_slice(cols.len(), t)]
            .iter()
            .intersperse(&j)
            .map(|x| x.clone())
            .collect(),
    }
}

fn read_inputs(delimiter: &String, input: Box<dyn BufRead>) -> Vec<Vec<String>> {
    let input_matrix = input
        .lines()
        .map(|l| l.unwrap())
        .map(|l| {
            l.split(delimiter)
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
        })
        .collect::<Vec<_>>();
    if input_matrix.len() == 0 {
        exit(0);
    }
    input_matrix
}

fn get_number_of_columns(args: &Args, input_length: usize) -> usize {
    let mut widths = vec![input_length];
    if let Some(h) = &args.headers {
        widths.push(h.len());
    };
    if let Some(c) = &args.columns {
        widths.push(c.len());
    };
    if let Some(a) = &args.layout {
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

    let mut input_matrix = read_inputs(&args.delimiter, input);
    if input_matrix.len() == 0 {
        exit(0);
    }

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

mod test {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_sort_comparator() {
        let order = Some(vec![
            SortOrder {
                column: 0,
                numeric: true,
                descending: true,
            },
            SortOrder {
                column: -1,
                numeric: false,
                descending: false,
            },
        ]);
        let sc1 = get_sort_comparator(&order);
        assert_eq!(
            sc1(&vec!["1000".to_string()], &vec!["200".to_string()]),
            Ordering::Less
        );
        assert_eq!(
            sc1(&vec!["40".to_string()], &vec!["9".to_string()]),
            Ordering::Less
        );
        assert_eq!(
            sc1(
                &vec!["a".to_string(), "40".to_string()],
                &vec!["b".to_string(), "9".to_string()]
            ),
            Ordering::Less
        );
        assert_eq!(
            sc1(
                &vec!["-03".to_string(), "Charlie".to_string()],
                &vec!["-3".to_string(), "Bravo".to_string()]
            ),
            Ordering::Greater
        );
        assert_eq!(
            sc1(
                &vec!["-0".to_string(), "Foo".to_string()],
                &vec!["0".to_string(), "Foo".to_string()]
            ),
            Ordering::Equal
        );
    }

    #[test]
    fn test_absolute_index() {
        assert_eq!(absolute_index(1, &0), 0);
        assert_eq!(absolute_index(10, &9), 9);
        assert_eq!(absolute_index(10, &10), 10); //OOB
        assert_eq!(absolute_index(10, &-1), 9);
        assert_eq!(absolute_index(10, &-4), 6);
        assert_eq!(absolute_index(10, &-10), 0);
        assert_eq!(absolute_index(10, &-1337), usize::max_value());
        assert_eq!(absolute_index(0, &-1337), usize::max_value());
    }
    #[test]
    fn test_absolute_index_slice() {
        let foo = vec![1, 2, 3, 4, 5, 6, 7];
        assert_eq!(foo[..absolute_index_slice(7, &0)], foo[0..0]);
        assert_eq!(foo[..absolute_index_slice(7, &1)], foo[0..1]);
        assert_eq!(foo[..absolute_index_slice(7, &1000)], foo[0..7]);
        assert_eq!(foo[..=absolute_index_inclusive_slice(7, &1000)], foo[0..7]);
        assert_eq!(foo[..absolute_index_slice(7, &-19)], foo[0..0]);
        assert_eq!(foo[..absolute_index_slice(7, &-3)], foo[0..4]);
        assert_eq!(
            foo[absolute_index_slice(7, &-5)..absolute_index(7, &-3)],
            foo[2..4]
        );
        assert_eq!(
            foo[absolute_index_slice(7, &-4)..absolute_index(7, &7)],
            foo[3..7]
        );
    }

    #[test]
    fn test_map_column() {
        let col = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
        ];
        assert_eq!(map_column(&ColumnMapping::Index(34), &col), "");
        assert_eq!(map_column(&ColumnMapping::Index(2), &col), "c");
        assert_eq!(map_column(&ColumnMapping::Index(-2), &col), "2");
        assert_eq!(map_column(&ColumnMapping::Index(0), &col), "a");

        assert_eq!(
            map_column(&ColumnMapping::List(vec![1, 0, -2], ",".to_string()), &col),
            "b,a,2"
        );
        assert_eq!(
            map_column(&ColumnMapping::List(vec![], ",".to_string()), &col),
            ""
        );

        assert_eq!(
            map_column(&ColumnMapping::Range(-20, 30, ",".to_string()), &col),
            "a,b,c,1,2,3"
        );
        assert_eq!(
            map_column(&ColumnMapping::Range(2, 4, ",".to_string()), &col),
            "c,1"
        );
        assert_eq!(
            map_column(&ColumnMapping::Range(2, 5, ",".to_string()), &col),
            "c,1,2"
        );
        assert_eq!(
            map_column(
                &ColumnMapping::InclusiveRange(-20, 30, ",".to_string()),
                &col
            ),
            "a,b,c,1,2,3"
        );
        assert_eq!(
            map_column(&ColumnMapping::InclusiveRange(2, 4, ",".to_string()), &col),
            "c,1,2"
        );
        assert_eq!(
            map_column(&ColumnMapping::InclusiveRange(2, 5, ",".to_string()), &col),
            "c,1,2,3"
        );

        assert_eq!(
            map_column(&ColumnMapping::InfinteRange(2, ",".to_string()), &col),
            "c,1,2,3"
        );
        assert_eq!(
            map_column(&ColumnMapping::InfinteRange(-3, ",".to_string()), &col),
            "1,2,3"
        );
        assert_eq!(
            map_column(&ColumnMapping::InfinteRange(-3000, ",".to_string()), &col),
            "a,b,c,1,2,3"
        );
    }
}
