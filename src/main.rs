use core::fmt;
extern crate regex;
use regex::{Captures, Regex};
use std::str::FromStr;

use clap::{arg, Parser};

#[derive(clap::ValueEnum, Clone, Default, Debug)]
enum Decoration {
    #[default]
    #[clap(alias = "u")]
    UnderlineHeader,
    #[clap(alias = "n")]
    None,
    #[clap(alias = "s")]
    SurroundTopBottom,
    #[clap(alias = "f")]
    Full,
}

impl fmt::Display for Decoration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Decoration::UnderlineHeader => write!(f, "underline-header"),
            Decoration::None => write!(f, "none"),
            Decoration::SurroundTopBottom => write!(f, "surround-top-bottom"),
            Decoration::Full => write!(f, "full"),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum Alignment {
    Left,
    Right,
    Center,
}
impl Alignment {
    fn from_chr(c: char) -> Self {
        match c {
            'l' => Self::Left,
            'r' => Self::Right,
            'c' => Self::Center,
            _ => panic!("should have been unreachable"),
        }
    }
}

#[derive(Debug, Clone)]
struct ColumnLayout {
    column_align: Vec<Alignment>,
    delimiters: Vec<String>,
}

fn contains_only_valid(s: &str, chars: Vec<char>) -> Result<(), char> {
    match s.chars().filter(|c| !chars.contains(c)).peekable().peek() {
        Some(invalid_char) => Err(*invalid_char),
        _ => Ok(()),
    }
}

impl FromStr for ColumnLayout {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let valid_chars = vec!['l', 'r', 'c', ' ', '|'];
        contains_only_valid(s, valid_chars).map_err(|c| format!("Invalid character: {}", c))?;
        let column_align = s
            .chars()
            .filter(|c| char::is_alphabetic(*c))
            .map(Alignment::from_chr)
            .collect();
        let delimiters = s
            .split(char::is_alphabetic)
            .map(|s| s.to_string())
            .collect();
        Ok(ColumnLayout {
            column_align,
            delimiters,
        })
    }
}

#[derive(Debug, Clone)]
enum ColumnMapping {
    Index(i16),
    List(Vec<i16>),
    Range(i16, i16),
    InfinteRange(i16),
    InclusiveRange(i16, i16),
}

impl FromStr for ColumnMapping {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(v) = i16::from_str(s) {
            return Ok(ColumnMapping::Index(v));
        }
        if s.contains(',') {
            let conv: Result<Vec<i16>, _> = s.split(|c| c == ',').map(i16::from_str).collect();
            match conv {
                Ok(v) => return Ok(ColumnMapping::List(v)),
                _ => return Err("Failed to parse list".to_string()),
            }
        }
        let range_re = Regex::new(r"^([+-]?\d+)(..=?)([+-]?\d+)?").unwrap();
        if let Some(cap) = range_re.captures(s) {
            let from_i = i16::from_str(&cap[1]).map_err(|_| "Failed to parse from".to_string())?;
            let inclusive = &cap[2] == "..=";
            if cap.get(3) == None && !inclusive {
                return Ok(ColumnMapping::InfinteRange(from_i));
            }
            let to_i = i16::from_str(&cap[3]).map_err(|_| "Failed to parse to ".to_string())?;
            if inclusive {
                return Ok(ColumnMapping::InclusiveRange(from_i, to_i));
            }
            return Ok(ColumnMapping::Range(from_i, to_i));
        }
        return Err("Invalid column specifier".to_string());
    }
}

#[derive(Debug, Clone)]
enum WidthSpecifier {
    Break(u16),
    Cut(u16),
    Ellipsis(u16),
}

impl FromStr for WidthSpecifier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() >= 2 {
            let w = &s[0..s.len() - 1];
            let n = u16::from_str(w).map_err(|_| "Failed to parse_width".to_string())?;
            match s.chars().last() {
                Some('b') => return Ok(WidthSpecifier::Break(n)),
                Some('c') => return Ok(WidthSpecifier::Cut(n)),
                Some('e') => return Ok(WidthSpecifier::Ellipsis(n)),
                _ => {}
            }
        }
        return Err("Invalid width specifier".to_string());
    }
}

#[derive(Debug, Clone)]
struct SortOrder {
    column: u16,
    descending: bool,
    numeric: bool,
}

impl FromStr for SortOrder {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() >= 2 {
            let w = &s[0..s.len() - 1];
            let n = u16::from_str(w).map_err(|_| "Failed to parse_width".to_string())?;
            match s.chars().last() {
                Some('l') => {
                    return Ok(SortOrder {
                        column: n,
                        descending: false,
                        numeric: false,
                    })
                }
                Some('L') => {
                    return Ok(SortOrder {
                        column: n,
                        descending: true,
                        numeric: false,
                    })
                }
                Some('n') => {
                    return Ok(SortOrder {
                        column: n,
                        descending: false,
                        numeric: true,
                    })
                }
                Some('N') => {
                    return Ok(SortOrder {
                        column: n,
                        descending: true,
                        numeric: true,
                    })
                }
                _ => {}
            }
        }
        return Err("Failed to parse SortOrder".to_string());
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Specify column headers. Each column header is separated by '|'
    #[arg(
        short = 't',
        long,
        help = "Headers of columns",
        value_delimiter = '|',
        verbatim_doc_comment
    )]
    headers: Option<Vec<String>>,

    /// Specify which columns of the input should be mapped to which output
    /// column. The mapping for each output column is separated by '|'.
    ///
    /// An output column can be mapped to:
    ///     ([+-]\d+)                 A single input column, negative indices
    ///                               count from the back.
    ///     (([+-]\d+),)+([+-]\d+)    A list of columns
    ///     ([+-]\d+)..=?([+-]\d+)?   A range (rust range syntax) of columns
    ///
    /// Example: TODO
    #[arg(
        short = 'c',
        long,
        help = "Column indices",
        value_delimiter = '|',
        value_parser = clap::value_parser!(ColumnMapping),
        verbatim_doc_comment
    )]
    columns: Option<Vec<ColumnMapping>>,

    /// Specify column alignment and vertical separators.
    /// Columns can be aligned to the left, to the right, or centered, which can
    /// be specified by 'l', 'r' or 'c' respectively.
    ///
    /// Each column alignment may be separated by any combination of ' ' and
    /// '|', which will be used as the column divider. Be careful, e.g. 'lll'
    /// results in all 3 columns not being separated by anything.
    ///
    /// Example: -a 'l|l|r' results in:
    /// 10000|2000|3000
    /// 40   |50  |  60
    ///
    /// Example: -a 'l    c|| r' results in:
    /// 10000    2000|| 3000
    /// 40        50 ||   60
    #[arg(short = 'a', long, verbatim_doc_comment, value_parser = clap::value_parser!(ColumnLayout))]
    alignment: Option<ColumnLayout>,

    /// Sort the output, as specified by the rules of --sort-by
    #[arg(short = 's', long)]
    sort: bool,

    /// Specify the keys by which the output should be sorted. Multiple levels
    /// of sorting can be specified, delimited by '|'. Each sort level is a
    /// column number followed by the sort type ([+-]\d+)[lLnN]
    ///
    /// Sort type:
    ///     l     lexicographic, ascending
    ///     L     lexicographic, descending
    ///     n     numeric, ascending
    ///     N     numeric, descending
    ///
    /// Note: Numeric sorting will use the raw-byte values of non-numeric
    /// characters
    ///
    /// Example: --sort-by '2n|1l'
    ///     Sorting is done numerically in ascending order, based on the second
    ///     column (zero-indexed). If the second column is equal, rows are
    ///     further sorted lexicographically, in ascending order by the first
    ///     column.
    #[arg(long, value_delimiter = '|', verbatim_doc_comment, value_parser = clap::value_parser!(SortOrder))]
    sort_by: Option<Vec<SortOrder>>,

    /// Apply sort-by to OUTPUT instead of INPUT columns.
    #[arg(long)]
    sort_by_output: bool,

    /// Remove duplicate lines from output. Duplicate removal is done based on
    /// the OUTPUT columns
    #[arg(short = 'u', long)]
    unique: bool,

    /// String by which to separate the input columns
    #[arg(short = 'd', long, default_value = ",")]
    delimiter: String,

    /// Collapse a sequence of multiple delimiters in a row into a single one.
    /// By default, empty columns are created instead.
    #[arg(short = 'D', long, default_value_t = false)]
    collapse_delimiters: bool,

    /// Specify fixed sizes for columns. Each column declaration is delimited by
    /// '|'. Each size is the width, a positive number, followed by the
    /// overflow specifier 'b','c','e', (\d+)[bce]
    ///
    /// Overflow specifiers:
    ///     b   Break table layout
    ///     c   Cut string
    ///     e   Cut string, but replace the last 3 visible characters by
    ///         ellipsis (...)
    #[arg(short = 'w', long, verbatim_doc_comment, value_parser = clap::value_parser!(WidthSpecifier))]
    fixed_width: Option<Vec<WidthSpecifier>>,

    /// How the table should look
    #[arg(long, default_value_t = Decoration::UnderlineHeader)]
    decoration: Decoration,

    /// The input file, default is STDIN
    #[arg()]
    file: Option<String>,
}

fn main() {
    let args = Args::parse();
    println!("{:?}", args);
}
