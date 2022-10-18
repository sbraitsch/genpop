use std::env;
use std::fs::File;
use std::io::Write;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let rows = &args[1].parse::<u32>().unwrap();
    let mut templates = Vec::new();
    if args.len() >= 2 {
        for i in 2..args.len() {
            templates.push(&args[i]);
        }
    } else {
        panic!("No template provided.")
    }


    let mut script = File::create("./migration.sql")?;
    for &template in &templates {
        let statement_end = template.find('{').unwrap();
        let statement = &template[0..statement_end];
        script.write_all(statement.as_bytes())?;
        script.write(b"\n")?;

        let values = &template[statement_end + 1..];
        let token = &values.split(',');

        for row_count in 1..=*rows {
            let mut row = String::new();
            for (i, t) in token.clone().enumerate() {
                match i {
                    0 => {
                        row.push_str(&format!("(NEXTVAL('{}'),", t.strip_suffix("}").unwrap()));
                        continue;
                    },
                    _ => {
                        let template_index = match t.find("{") {
                            Some(index) => index as usize,
                            None => {
                                row.push_str(t);
                                row.push_str(",");
                                continue;
                            }
                        };
                        let constraint = t.chars().nth(template_index + 1).unwrap();
                        match constraint {
                            '}' => {
                                row.push_str(&format!("Text#{}", row_count));
                            },
                            'm' => {
                                row.push_str(&((row_count + 2) % 3).to_string());
                                row.push_str(",");
                            }
                            _ => {
                                let constraint_bound = constraint.to_digit(10).unwrap();
                                row.push_str(&((row_count + 4) % constraint_bound).to_string());
                                row.push_str(",");
                            }
                        }
                    }
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