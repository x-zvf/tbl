# tbl (tabulate) - Make pretty tables in the command line
---

Tbl enables you to quickly format textual data (e.g. CSVs) into tables.

## Installation
### Building from source
```sh
git clone https://github.com/x-zvf/tbl
cd tbl
cargo build --release
```


## Examples

1. Quickly view a CSV

```
~$ tbl constants.csv
Name              │Symbol│Value           │Unit│Info       
──────────────────┼──────┼────────────────┼────┼───────────
Speed of light    │c     │299792458       │m/s │in vacuum  
Plank constant    │h     │6.62607015×10−34│eVs │           
Euler's number    │e     │2.71828         │    │approximate
Boltzmann constant│k     │1.380649×10−23  │J/K │           
```

2. You can specify titles, select a subset of columns, sort by columns, align values and more.
```
~$ tbl users.csv -t Birthday,Name,Email -s --sort-by 1l -c '1,2;3,4' -l 'l | c | l' --decoration full
┌───────────┬──────────────────┬────────────────────────────────┐
│Birthday   │       Name       │ Email                          │
├───────────┼──────────────────┼────────────────────────────────┤
│1973-04-23 │  Holley Garland  │ matilde_pinkerton0738@gmail.com│
│1980-03-05 │   Taylor Wendt   │ lincoln580@nav.com             │
│1991-05-08 │  Shaunda Keegan  │ sooames74@gmail.com            │
│1997-08-13 │   Loraine Yoo    │ marquerite4@hotmail.com        │
│2000-08-01 │ Dominique Alonso │ clarinda-nance@hotmail.com     │
│2006-01-01 │    Lorie Chin    │ vicente.donovan@movie.com      │
│2008-12-22 │   Lauralee Bey   │ kenisha483@constraints.com     │
│2011-07-19 │  Mohammed Horn   │ lizethbolin@yahoo.com          │
│2018-09-11 │   Edythe Frey    │ misti.franco61@bidder.com      │
│2021-02-28 │    Star Bolt     │ rodrick703@yahoo.com           │
└───────────┴──────────────────┴────────────────────────────────┘
```

## Usage

```
Usage: tbl [OPTIONS] [FILE]

Arguments:
  [FILE]
          The input file, default is STDIN

Options:
  -t, --headers <HEADERS>
          Headers of columns

  -c, --columns <COLUMNS>
          Specify input should be mapped to which output columns.
          
          The mapping for each output column is separated by ','.
          
          An output column can be mapped to:
              ([+-]\d+)                 A single input column, negative indices
                                        count from the back.
              (([+-]\d+);)+([+-]\d+)    A list of columns, separated by ';'
              ([+-]\d+)..=?([+-]\d+)?   A range (rust range syntax) of columns
          
          Optionally, after a non-single mapping a '>' character can be put,
          the string after which will be used to join the columns. If this
          is not specified, " " is used.
          
          Example: -c '3,0..=2>/,-3;5;4,-2..'
          The 3rd (zero-indexed) column should become the 0th, columns 0,1,2
          should be joined together by '/' to create the first column, the 3rd-to
          -last column and columns 5 and 4 joined by ' ' will be the second, and
          the second-to-last and last columns joined by ' ' will be the third
          column.

  -l, --layout <LAYOUT>
          Specify column alignment and vertical separators.
          
          Columns can be aligned to the left, to the right, or centered, which can
          be specified by 'l', 'r' or 'c' respectively.
          
          Each column alignment may be separated by any combination of ' ' and
          '|', which will be used as the column divider. Be careful, e.g. 'lll'
          results in all 3 columns not being separated by anything.
          
          Example: -a 'l|l|r' results in:
          10000|2000|3000
          40   |50  |  60
          
          Example: -a 'l    c|| r' results in:
          10000    2000|| 3000
          40        50 ||   60

  -w, --fixed-width <FIXED_WIDTH>
          Specify fixed sizes for columns.
          
          Each column declaration is delimited by ','. Each size is the width, a
          positive number, followed by the overflow specifier 'b','c', or 'e'.
          
          Overflow specifiers:
              b   Break table layout
              c   Cut string
              e   Cut string, but replace the last 3 visible characters by
                  ellipsis (...)

      --decoration <DECORATION>
          How the table should look
          
          [default: underline-header]
          [possible values: underline-header, none, full]

  -a, --ascii
          Do not use Unicode characters for displaying table borders

  -s, --sort
          Sort the output, as specified by the rules of --sort-by

      --sort-by <SORT_BY>
          Specify the keys by which the output should be sorted.
          
          Multiple levels of sorting can be specified, delimited by ','. Each sort
          level is a column number followed by the sort type ([+-]\d+)[lLnN]
          
          Sort type:
              l     lexicographic, ascending
              L     lexicographic, descending
              n     numeric, ascending
              N     numeric, descending
          
          
          Example: --sort-by '2n,1l'
              Sorting is done numerically in ascending order, based on the second
              column (zero-indexed). If the second column is equal, rows are
              further sorted lexicographically, in ascending order by the first
              column.

      --sort-by-output
          Apply sort-by to OUTPUT instead of INPUT columns

      --sort-ignore-first
          Ignore the first line when sorting.
          
          This is useful, if the first row contains column headers.

  -u, --unique
          Remove duplicate lines from output.
          
          Duplicate removal is done based on the OUTPUT columns

  -d, --delimiter <DELIMITER>
          String by which to separate the input columns
          
          [default: ,]

  -D, --collapse-delimiters
          Collapse a sequence of multiple delimiters in a row into a single one.
          
          By default, empty columns are created instead.

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Contributing

Contributions are welcome. In particular, I would appreciate more thorough unit
and integration testing, documentation and packaging for distributions.
New features are also welcome, but please keep in mind, that tbl should not become
a replacement for AWK, Perl, Python, and others.


## License
This project is licensed under the MIT license. See LICENSE for a copy.
