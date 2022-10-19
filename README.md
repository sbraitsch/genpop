# What this?
genpop is a tool to generate massively long insert statements to use with e.g. flyway.
Possible usecase would be 

# How to use it

Download the repository and unzip, then run
```
cargo install --path /path/to/folder
```
genpop takes 3+ arguments: a path with filename+extension, the number of rows to generate and a variable amount of template strings.
Template values are separated by |.
The first value is required to be the name of the table. Ignore at your own peril.

Supported value patterns are:

i                 - autoincrementing id, starting from 1
s[sequence_name]  - autoincrementing id starting, from the sequences current value
n[upper_bound]    - random number from 0 to exclusive upper_bound
o[o1,o2,o3]       - one of the comma-separated options provided, rotates by given order
d[rows_per_day]   - datestring and occurrence count before decrementing

For example:
```
genpop ./migration.sql 3 "mytable|i|n[4]|d[2]|o['CAT','MOUSE']"
```
Will generate:
```
INSERT INTO mytable VALUES
(1,3,'2022-10-19','CAT'),
(2,0,'2022-10-19','MOUSE'),
(3,2,'2022-10-18','CAT');
```
