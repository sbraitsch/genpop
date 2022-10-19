use std::process;
use std::env;
use std::fs::File;
use std::io::Write;
use std::time::Instant;
use chrono::{Utc, Duration};

fn main() -> std::io::Result<()> {
    let start = Instant::now();
    let args: Vec<String> = env::args().collect();

    let mut templates = Vec::new();
    if args.len() >= 2 {
        for i in 2..args.len() {
            templates.push(&args[i]);
        }
    } else {
        print_guide();

        process::exit(0);
    }

    let rows = &args[1].parse::<u32>().unwrap();
    let flag_i = "i";
    let flag_s = "s";
    let flag_n = "n";
    let flag_o = "o";
    let flag_d = "d";

    let mut script = File::create("./migration.sql")?;
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
                                }
                                _ => panic!("Invalid template.")
                            }
                        },
                        None => panic!("Invalid template.")
                    }
                }
            }
        }
        let statement = format!("INSERT INTO {} values\n", values[0].1);
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
                        row.push_str(&format!("{},", ((row_count + 2) % param.parse::<u32>().unwrap())));
                    },
                    ("o", param) => {
                        let options = param.split(',').collect::<Vec<&str>>();
                        row.push_str(&format!("{},", options[(row_count % options.len() as u32) as usize]));
                    },
                    ("d", param) => {
                        let date_string = &format!("{}", Utc::now() - Duration::days((row_count as i64) / param.parse::<i64>().unwrap()))[0..10];
                        row.push_str(&format!("'{}',", date_string));
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
    println!("Generated inserts for {} database entries in {:?}", (rows * (args.len() - 2) as u32), duration);
    Ok(())
}

fn extract_param(token: &str) -> &str {
    match token.find("[") {
        Some(par) => token[par + 1..].strip_suffix("]").unwrap(),
        None => panic!("Invalid template.")
    }
}

fn print_guide() {
    let exp_gen = "genpop takes 2 or more arguments. The numbe rof rows to generate and a variable amount of templates.\nTemplates values are separated by |\n First value is always the name of the table.\nFollowing values can be one of:";
    let exp_i = "i -> autoincrementing id, increases for every row";
    let exp_s = "s[sequence_name] -> id taken from a preexisting sequence";
    let exp_n = "n[upper_bound] -> number from 0 to upper_bound";
    let exp_o = "o[o1,o2,o3] -> one of the comma-separated options provided";
    let exp_d = "d[rows_per_day] -> datestring and how many rows with each day";

    println!("{}\n{}\n{}\n{}\n{}\n{}", exp_gen, exp_i, exp_s, exp_n, exp_o, exp_d);
    println!("Example:\ngenpop 1000 \"mytable|i|n[4]|d[3]|o[cat,mouse]\"");
}