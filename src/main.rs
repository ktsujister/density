use structopt::StructOpt;
use thiserror::Error;

use std::io::{self, Read, BufRead, BufReader, Write};
use std::num::ParseFloatError;

#[derive(Debug, Error)]
enum AppError {
    #[error("parse error")]
    ParseError(#[from] ParseFloatError),

    #[error("io error occured")]
    IoError(#[from] std::io::Error),
}

type Result<T> = ::std::result::Result<T, AppError>;

const DEFAULT_FIELD_POS: &str = "0";

fn get_input_reader() -> BufReader<Box<dyn Read>> {
    BufReader::new(Box::new(io::stdin()))
}

fn parse_line(line: &str, field_pos: usize) -> Result<f64> {
    let fields: Vec<_> = line.split("\t").collect();
    Ok(fields[field_pos].parse::<f64>()?)
}

#[derive(Debug, Clone, StructOpt)]
#[structopt(long_version(option_env!("LONG_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"))))]
#[structopt(rename_all = "kebab-case")]
/// Simple command line tool for getting density and cumulative density from input.
struct Density {
    /// Sets verbose flag.
    #[structopt(short, long)]
    verbose: bool,

    /// Specify which field to use. (0 origin).
    #[structopt(short, long="field", value_name="FIELD", default_value=DEFAULT_FIELD_POS)]
    field_pos: usize,

    /// Show results in percentage.
    #[structopt(short, long)]
    percentage: bool,
}

impl Density {
    fn run(self) -> Result<()> {
        let mut lines: Vec<(f64, String)> = vec![];
        let mut total = 0.0;
        for line_val in get_input_reader().lines() {
            let line_str = line_val?;
            let val = parse_line(&line_str, self.field_pos)?;
            lines.push((val, line_str));
            total += val;
        }
        if self.verbose {
            eprintln!("total: {}, total-lines: {}", total, lines.len());
        }
        let stdout = io::stdout();
        let mut w = stdout.lock();
        let mut cum_total = 0.0;
        for (val, line) in lines {
            cum_total += val;
            let density;
            let cum_density;
            if self.percentage {
                density = (val * 100.0) / total;
                cum_density = (cum_total * 100.0) / total;
            } else {
                density = val / total;
                cum_density = cum_total / total;
            }
            writeln!(&mut w, "{}\t{:.5}\t{:.5}", line, density, cum_density)?;
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let density = Density::from_args();
    if density.verbose {
	eprintln!("(field_pos) = {:?}", (density.field_pos));
    }

    match density.run() {
	Ok(_) => {},
	Err(AppError::IoError(e)) if e.kind() == io::ErrorKind::BrokenPipe => {},
	Err(e) => {
	    eprintln!("get_density failed! err: {:?}", e);
	    return Err(e);
	}
    }
    Ok(())
}
