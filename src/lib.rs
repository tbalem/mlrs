use std::error::Error;
use std::fmt;
use std::fs;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let data = from_csv_file(&config.csv_file, &config.columns)?;
    println!("Header: {:?}", data.header);
    println!("{:?} ; {:?}", data.content[0][0], data.content[0][1]
    );
    Ok(())
}

pub struct Config {
    csv_file: String,
    columns: Vec<String>,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("Wrong number of argument given, 2 expected");
        }
        let csv_file = args[1].clone();
        let columns: Vec<String> = args[2].split(",").map(str::to_owned).collect();
        Ok(Config {
            csv_file: csv_file,
            columns: columns,
        })
    }
}

#[derive(Debug)]
struct CSVError {
    details: String,
}

impl fmt::Display for CSVError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for CSVError {
    fn description(&self) -> &str {
        &self.details
    }
}

#[derive(Debug)]
enum GenericCellValue {
    U32(u32),
    F32(f32),
    Str(String),
}

struct CSV {
    header: Vec<String>,
    content: Vec<Vec<GenericCellValue>>,
}

impl CSV {
    fn new() -> CSV {
        CSV {
            header: Vec::new(),
            content: Vec::new(),
        }
    }

    fn read_from_str(&mut self, csv_content: &str) -> Result<(), Box<dyn Error>> {
        match csv_content.len() {
            // File cannot be empty
            0 => Err(Box::new(CSVError {
                details: String::from("String is empty"),
            })),
            nb_lines => {
                // Split between header (first line) and body (other lines)
                let mut splitted_file_content = csv_content.splitn(2, "\n");
                self.header = Vec::from_iter(
                    splitted_file_content
                        .next()
                        .expect("CSV file should contain at least one line")
                        .split(",")
                        .map(str::to_owned),
                );

                self.content = Vec::with_capacity(nb_lines - 1);
                // Split each column of each row
                for (i, csv_line) in splitted_file_content
                    .next()
                    .unwrap_or("")
                    .split("\n")
                    .enumerate()
                {
                    fn parse_uint_float_or_str(s: &str) -> GenericCellValue {
                        match s.parse::<u32>() {
                            Ok(n) => GenericCellValue::U32(n),
                            Err(_) => match s.parse::<f32>() {
                                Ok(f) => GenericCellValue::F32(f),
                                Err(_) => GenericCellValue::Str(s.to_owned()),
                            },
                        }
                    }
                    if !csv_line.is_empty() {
                        let csv_row =
                            Vec::from_iter(csv_line.split(",").map(|s| parse_uint_float_or_str(s)));
                        // Check if row has the correct number of columns
                        if csv_row.len() != self.header.len() {
                            return Err(Box::new(CSVError {
                                details: format!(
                                    "Wrong column count for line {}, {} expected and got {}",
                                    i + 1,
                                    self.header.len(),
                                    csv_row.len()
                                ),
                            }));
                        }
                        self.content.push(csv_row);
                    }
                }

                Ok(())
            }
        }
    }

    // Filter columns and keep old order
    fn filter_csv(&mut self, columns_to_keep: &[String]) // -> Result<(), Box<dyn Error>>
    {
        // Get columns to remove
        let mut columns_to_remove = Vec::new();
        for column in self.header.iter() {
            if !columns_to_keep.contains(&column) {
                columns_to_remove.push(column.clone());
            }
        }

        // Remove all column to remove
        for column_to_remove in columns_to_remove.iter() {
            let index_to_remove = self
                .header
                .iter()
                .position(|column| column_to_remove == column)
                .unwrap();
            self.header.remove(index_to_remove);
            for row in self.content.iter_mut() {
                let mut index = 0usize;
                row.retain(|_| {
                    index += 1;
                    index_to_remove != index -1
                });
            }
        }
    }
}

fn from_csv_file(file_path: &str, columns: &[String]) -> Result<CSV, Box<dyn Error>>
// T: FromStr,
    // T: Copy,
    // <T as FromStr>::Err: fmt::Debug,
{
    // Read file
    let mut csv = CSV::new();
    csv.read_from_str(&fs::read_to_string(file_path)?)?;
    csv.filter_csv(columns);
    Ok(csv)
}
