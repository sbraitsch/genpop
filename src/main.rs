use std::process;
use std::env;
use std::fs::File;
use std::io::Write;
use std::time::Instant;
use chrono::{Utc, Duration};
use rand::Rng;

fn main() -> std::io::Result<()> {
    let start = Instant::now();
    let args: Vec<String> = env::args().collect();

    let mut templates = Vec::new();
    if args.len() >= 3 {
        for i in 3..args.len() {
            templates.push(&args[i]);
        }
    } else {
        print_guide();

        process::exit(0);
    }

    let mut rng = rand::thread_rng();
    let rows = &args[2].parse::<u32>().unwrap();
    let path = &args[1];

    let mut uniques:Vec<(u16, Vec<String>)> = Vec::new();
    let mut script = File::create(path)?;

    for &template in &templates {
        let token = &template.split('|');
        let mut values: Vec<(String, &str)> = Vec::new();
        for (i, t) in token.clone().enumerate() {
            let mut s = String::new();
            match i {
                0 => {
                    values.push( ("".to_string(), t));
                },
                _ => {
                    match t.chars().nth(0) {
                        Some(x) => {
                            match x {
                                'i' => {
                                    s.push(x);
                                    values.push((s, ""));
                                },
                                's' | 'r' | 'o' | 'd' | 'u' | 'n'  => {
                                    s.push(x);
                                    values.push((s, extract_param(t)));
                                },
                                _ => panic!("Invalid template.")
                            }
                        },
                        None => panic!("Invalid template.")
                    }
                }
            }
        }

        let mut unique: Vec<String>;
        for (f, p) in &values {
            match (f.as_str(),*p) {
                ("u", param) => {
                    let len = param.parse::<u16>().unwrap();
                    if !uniques.iter().any(|(x,_)| *x == len) {
                        unique = generate_uniques(len, *rows);
                        uniques.push((len, unique));
                    }
                },
                _ => {continue;}
            }
        }

        let statement = format!("INSERT INTO {} VALUES\n", values[0].1);
        script.write_all(statement.as_bytes())?;
        for row_count in 1..=*rows {
            
            let mut row = String::new();
            row.push('(');
            
            for (f, p) in &values[1..] {
                match (f.as_str(), *p) {
                    ("i", "") => {
                        row.push_str(&format!("{},", row_count));
                    },
                    ("s", param) => {
                        row.push_str(&format!("NEXTVAL('{}'),", param));
                    },
                    ("r", param) => {
                        row.push_str(&format!("{},", &rng.gen_range(0..param.parse::<u32>().unwrap())));
                    },
                    ("n", param) => {
                        let cycle = param.parse::<u32>().unwrap();
                        row.push_str(&format!("{},", ((row_count - 1) % cycle) + 1));
                    },
                    ("o", param) => {
                        let options = param.split(',').collect::<Vec<&str>>();
                        row.push_str(&format!("{},", options[(row_count % options.len() as u32) as usize]));
                    },
                    ("d", param) => {
                        let date_string = &format!("{}", Utc::now() - Duration::days(((row_count - 1) as i64) / param.parse::<i64>().unwrap()))[0..10];
                        row.push_str(&format!("'{}',", date_string));
                    },
                    ("u", param) => {
                        let len = param.parse::<u16>().unwrap();
                        let unique_val;
                        let options = &uniques.iter().find(|(x,_)| *x == len);

                        match options {
                            Some((_, y)) => {
                                unique_val = &y[(row_count - 1) as usize];
                            },
                            None => panic!("Something went wrong. Curious.")
                        }
                        row.push_str(&format!("'{}',", unique_val));
                    },
                    (_, &_) => panic!("Something went wrong. Curious.")
                } 
            }

            row.remove(row.len() - 1);

            if  row_count == *rows {
                row.push_str(");\n\n");
            } else {
                row.push_str("),\n")
            }
            script.write_all(row.as_bytes())?;
        }
    }
    let duration = start.elapsed();
    println!("Generated INSERT statement for {} new database rows in {:?}.", (rows * (args.len() - 3) as u32), duration);
    Ok(())
}

fn extract_param(token: &str) -> &str {
    match token.find("(") {
        Some(par) => token[par + 1..].strip_suffix(")").unwrap(),
        None => panic!("Invalid template.")
    }
}

fn generate_uniques(len: u16, num: u32) -> Vec<String> {
    let mut list = Vec::new();
    for i in 0..num {
        let mut s = String::new();
        let mut c;
        for k in 0..len {
            let div = (i as u64) / 26_u64.pow(k.into());
            let offset = (div % 26) as u8;
            c = (97 + offset) as char;
            s.push(c);
        }
        list.push(s);
    }
    list
}

fn print_guide() {
    let mut guide = String::new();
    guide.push_str("Command syntax: genpop [path+name] [rows] [template(s)]*\n");
    guide.push_str("\t- Template values are separated by |\n");
    guide.push_str("\t- The first value is required to be the name of the table\n");
    guide.push_str("Valid value types are:\n\n");

    guide.push_str("\ti\t\tautoincrementing id, starting from 1\n");
    guide.push_str("\ts(x)\t\tautoincrementing id based on the current value of an existing sequence with name x\n");
    guide.push_str("\tr(x)\t\trandom number from 0 to x (exclusive)\n");
    guide.push_str("\tn(x)\t\tnumber from 1 to x (inclusive). like i but resets after x\n");
    guide.push_str("\to(a,..,z)\tone of the comma-separated options provided. resets to a after reaching z\n");
    guide.push_str("\tu(x)\t\ta unique string with length x\n");
    guide.push_str("\td(x))\t\tdatestring with x as the number of rows with each date before decrementing\n\n");

    guide.push_str("Example:\ngenpop ./migration.sql 3 \"mytable|i|r(4)|d(2)|o('CAT','MOUSE')|u(3)\"");
    guide.push_str("Output:\n");
    guide.push_str("INSERT INTO mytable VALUES\n(1,3,'2022-10-19','CAT','aaa'),\n(2,0,'2022-10-19','MOUSE','baa'),\n(3,2,'2022-10-18','CAT','caa');");
    println!("{}", guide);
}