use std::{env, ops::Deref};
use std::fs::File;
use std::io::Write;
use chrono::{Utc};

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let rows = &args[1].parse::<u32>().unwrap();
    let mut templates = Vec::new();
    if args.len() >= 2 {
        for i in 2..args.len() {
            templates.push(&args[i]);
        }
    } else {
        println!("genpop [number of rows] [{{tablename}}{{id: i|(s[n])}}{{value: (n[b])|t|(o[opt1|opt2|opt3]|d)}}]");
        println!("Example:\ngenpop 1000 {{crabs}}{{i}}{{n[4]}}{{e[NORTH, SOUTH]}}{{d}}");
        println!("genpop -h for template explanations");

        panic!("No template provided.")
    }

    let mut script = File::create("./migration.sql")?;
    for &template in &templates {
        let token = &template.split('{');
        let mut values: Vec<(&str, Option<String>)> = Vec::new();
        for (i, t) in token.clone().enumerate() {
            match i {
                0 => {
                    continue;
                },
                1 => {
                    values.push( (t.strip_suffix("}").unwrap(), None));
                },
                _ => {
                    match t.chars().nth(0) {
                        Some(x) => {
                            match x {
                                'i' | 'd' => {values.push(("", Some(x.to_string())));},
                                's' | 'n' | 'o' => {values.push((extract_param(t), Some(x.to_string())));},
                                _ => panic!("Invalid template.")
                            }
                        },
                        None => panic!("Invalid template.")
                    }
                }
            }
        }
        let statement = format!("INSERT INTO {} values\n", values[0].0);
        script.write_all(statement.as_bytes())?;
        for row_count in 1..=*rows {
            
            let mut row = String::new();
            row.push('(');
            
            for i in 1..values.len() {
                match &values[i] {
                    ("", Some(s)) => {
                        match s.deref() {
                            "i" => {row.push_str(&format!("{},", row_count));},
                            "d" => {row.push_str(&format!("{},", Utc::now()));},
                            &_ => panic!("Something went wrong. Curious.")

                        }
                    },
                    (param, Some(s)) => {
                        match s.deref() {
                            "s" => {row.push_str(&format!("NEXTVAL('{}',", param));},
                            "n" => {row.push_str(&format!("{},", ((row_count + 2) % param.parse::<u32>().unwrap())));},
                            "o" => {
                                let options = param.split('|').collect::<Vec<&str>>();
                                row.push_str(&format!("{},", options[((row_count + 2) % options.len() as u32) as usize]));},
                            &_ => panic!("Something went wrong. Curious.")

                        }
                    },
                    (&_, _) => panic!("Something went wrong. Curious.")
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

    Ok(())
}

fn extract_param(token: &str) -> &str {
    match token.find("[") {
        Some(par) => token[par + 1..].strip_suffix("]}").unwrap(),
        None => panic!("Invalid template.")
    }
}