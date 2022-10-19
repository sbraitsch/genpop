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
    let flag_i = "i";
    let flag_s = "s";
    let flag_n = "n";
    let flag_o = "o";
    let flag_d = "d";
    let flag_u = "u";

    let mut uniques:Vec<(u16, Vec<String>)> = Vec::new();
    let mut script = File::create(path)?;

    for &template in &templates {
        let token = &template.split('|');
        let mut values: Vec<(&str, &str)> = Vec::new();
        for (i, t) in token.clone().enumerate() {
            match i {
                0 => {
                    values.push( ("", t));
                },
                _ => {
                    match t.chars().nth(0) {
                        Some(x) => {
                            match x {
                                'i' => {
                                    values.push((flag_i, ""));
                                },
                                's' => {
                                    values.push((flag_s, extract_param(t)));
                                },
                                'n' => {
                                    values.push((flag_n, extract_param(t)));
                                },
                                'o' => {
                                    values.push((flag_o, extract_param(t)));
                                },
                                'd' => {
                                    values.push((flag_d, extract_param(t)));
                                },
                                'u' => {
                                    values.push((flag_u, extract_param(t)));
                                }
                                _ => panic!("Invalid template.")
                            }
                        },
                        None => panic!("Invalid template.")
                    }
                }
            }
        }
        let mut unique: Vec<String>;
        for v in &values {
            match v {
                ("u", param) => {
                    let spec = param.split(',').collect::<Vec<&str>>();
                    let len = spec[0].parse::<u16>().unwrap();
                    if !uniques.iter().any(|(x,_)| *x == len) {
                        let num = spec[1].parse::<u32>().unwrap();
                        unique = generate_uniques(len, num);
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
            
            for i in 1..values.len() {
                match &values[i] {
                    ("i", "") => {
                        row.push_str(&format!("{},", row_count));
                    },
                    ("s", param) => {
                        row.push_str(&format!("NEXTVAL('{}'),", param));
                    },
                    ("n", param) => {
                        row.push_str(&format!("{},", &rng.gen_range(0..param.parse::<u32>().unwrap())));
                    },
                    ("o", param) => {
                        let options = param.split(',').collect::<Vec<&str>>();
                        row.push_str(&format!("{},", options[(row_count % options.len() as u32) as usize]));
                    },
                    ("d", param) => {
                        let date_string = &format!("{}", Utc::now() - Duration::days((row_count as i64) / param.parse::<i64>().unwrap()))[0..10];
                        row.push_str(&format!("'{}',", date_string));
                    },
                    ("u", param) => {
                        let spec = param.split(',').collect::<Vec<&str>>();
                        let len = spec[0].parse::<u16>().unwrap();
                        let num = spec[1].parse::<u32>().unwrap();
                        let unique_val;
                        let options = &uniques.iter().find(|(x,_)| *x == len);

                        match options {
                            Some((_, y)) => {
                                unique_val = &y[((row_count - 1) % num) as usize];
                            },
                            None => panic!("Something went wrong. Curious.")
                        }
                        row.push_str(&format!("'{}',", unique_val));
                    },
                    (&_, &_) => panic!("Something went wrong. Curious.")
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
    println!("Generated inserts for {} database entries in {:?}", (rows * (args.len() - 3) as u32), duration);
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
    let exp_gen = "genpop takes 3+ arguments: a path with filename+extension, the number of rows to generate and a variable amount of templates.\nTemplate values are separated by |.\nThe first value has to be the name of the table.\nValues can be one of:";
    let exp_i = "\ti\t\t\tautoincrementing id, starting from 1";
    let exp_s = "\ts[sequence_name]\tautoincrementing id starting, from the sequences current value";
    let exp_n = "\tn[upper_bound]\t\trandom number from 0 to exclusive upper_bound";
    let exp_o = "\to[o1,o2,o3]\t\tone of the comma-separated options provided, rotates by given order";
    let exp_d = "\td[rows_per_day]\t\tdatestring and occurrence count before decrementing";

    println!("{}\n{}\n{}\n{}\n{}\n{}", exp_gen, exp_i, exp_s, exp_n, exp_o, exp_d);
    println!("Example:\ngenpop ./migration.sql 3 \"mytable|i|n[4]|d[2]|o['CAT','MOUSE']\"");
    println!("Output:");
    println!("INSERT INTO mytable VALUES\n(1,3,'2022-10-19','CAT'),\n(2,0,'2022-10-19','MOUSE'),\n(3,2,'2022-10-18','CAT');")
}