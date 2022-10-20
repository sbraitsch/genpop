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
genpop [path+name] [rows] [template(s)]*
```
<br>
- Template values are separated by |<br>
- The first value is required to be the name of the table. Ignore at your own peril<br>

Supported value patterns are:

| Pattern  | Definition | Example | Result |
| ------------- | ------------- | ------------- | ------------- |
| **i(x)**  | **number from 1 to x (inclusive), repeating. unbound for x=0 or no x given**  | **i()** | **1<br>2<br>3<br>..**|
| **s(x)**| **number based on the current value of an existing sequence with name x** | **s(some_id)** | **24<br>25<br>26<br>..**|
| **r(x)**  | **random number from 0 to x (exclusive)** | **r(3)** | **2<br>0<br>1<br>..** |
| **u(x)**  | **a unique string with length x** | **u(3))** | **'aaa'<br>'baa'<br>'caa'<br>..** |
| **o(a,..,z)** | **one of the comma-separated options provided. resets to a after reaching z** | **o('NA','EU')** | **'NA'<br>'EU'<br>'NA'<br>..**|
| **d(x)** | **datestring with x as the number of rows with each date before decrementing** | **d(2)** | **'2022-01-02'<br>'2022-01-02'<br>'2022-01-01'<br>..**|

For example:
```
genpop ./migration.sql 3 "mytable|i()|r(4)|d(2)]|o('CAT','MOUSE')|u(3)"
```
Will generate:
```
INSERT INTO mytable VALUES
(1,3,'2022-10-19','CAT','aaa'),
(2,0,'2022-10-19','MOUSE','baa'),
(3,2,'2022-10-18','CAT'),'caa';
```
