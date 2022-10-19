# What this?
genpop is a tool to generate massively long SQL insert statements.<br>
You could e.g. use it to generate a migration file for flyway, inserting 50000 rows of mockdata to test the behaviour of your program heavy load.

# How to use it

Install Rust.<br>
Download the repository and unzip, then run:<br>
```
cargo install --path /path/to/folder
```
<br>
Command syntax:<br>

```
genpop [path+name] [rows] [template(s)]
```
<br>
- Template values are separated by |<br>
- The first value is required to be the name of the table. Ignore at your own peril<br>

Supported value patterns are:

| Pattern  | Definition | Example | Result |
| ------------- | ------------- | ------------- | ------------- |
| **i**  | **autoincrementing ID, starting from 1**  | **i** | **1<br>2<br>3<br>..**|
| **s[sequence_name]**| **autoincrementing ID starting from the sequences current value** | **s[some_id]** | **24<br>25<br>26<br>..**|
| **n[upper_bound]**  | **random number from 0 to exclusive upper_bound** | **n[3]** | **2<br>0<br>1<br>..** |
| **o[o1,o2,o3]** | **one of the comma-separated options provided, rotates by given order** | **o['NA','EU']** | **'NA'<br>'EU'<br>'NA'<br>..**|
| **d[rows_per_day]** | **datestring and occurrence count before decrementing** | **d[2]** | **'2022-01-02'<br>'2022-01-02'<br>'2022-01-01'<br>..**|

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
