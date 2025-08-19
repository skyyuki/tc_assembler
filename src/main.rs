use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Source assembry file.
    file: PathBuf,

    /// Output file name.
    #[arg(short, long)]
    out: Option<PathBuf>,
}

#[derive(Debug, Clone)]
enum Immdiate {
    Int(u8),
    Label(String),
}

#[allow(clippy::upper_case_acronyms)]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum Operater {
    OR,
    NAND,
    NOR,
    AND,
    ADD,
    SUB,
}

#[allow(clippy::upper_case_acronyms)]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum Condition {
    OFF,
    EQ,
    LS,
    LSEQ,
    ON,
    NEQ,
    GREQ,
    GR,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum Reg {
    REG0,
    REG1,
    REG2,
    REG3,
    REG4,
    REG5,
    IO,
}

#[allow(clippy::upper_case_acronyms)]
#[repr(u8)]
#[derive(Debug, Clone)]
enum Stmt {
    LET(Immdiate),
    Calc(Operater),
    COPY(Reg, Reg),
    Cond(Condition),
    Label(String),
}

impl Display for Immdiate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Immdiate::Int(num) => write!(f, "{num}"),
            Immdiate::Label(label) => write!(f, "@{label}"),
        }
    }
}

impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stmt::LET(immdiate) => write!(f, "LET {immdiate}"),
            Stmt::Calc(operater) => write!(f, "{operater:?}"),
            Stmt::COPY(dist, src) => write!(f, "COPY {dist:?} {src:?}"),
            Stmt::Cond(condition) => write!(f, "{condition:?}"),
            Stmt::Label(lable) => write!(f, "@{lable}:"),
        }
    }
}

fn parse<R: BufRead>(input: R) -> Result<Vec<Stmt>> {
    let mut stmts = Vec::new();
    // TODO: handle ; as newline
    for line in input.lines() {
        let line = line?;
        let mut tokens: std::str::SplitWhitespace<'_> = line.split_whitespace();
        let Some(token) = tokens.next() else {
            continue;
        };
        if token.starts_with("#") {
            continue;
        }
        if let Some(label) = token
            .strip_prefix('@')
            .and_then(|token| token.strip_suffix(':'))
        {
            stmts.push(Stmt::Label(String::from(label)));
            continue;
        }
        let stmt = match token.to_uppercase().as_str() {
            "LET" => {
                let operand = tokens.next().context("missing immdiate number")?;
                if let Some(label) = operand.strip_prefix('@') {
                    Stmt::LET(Immdiate::Label(label.to_string()))
                } else {
                    let operand: i32 = operand.parse()?;
                    if operand >= 1 << 5 {
                        anyhow::bail!(
                            "Immdiate number must less than 2^6, by restriction of the architecture. :{:}",
                            operand
                        );
                    } else if operand < 0 {
                        anyhow::bail!("Immdiate number must be non negative. :{:?}", operand);
                    }
                    Stmt::LET(Immdiate::Int(operand as u8))
                }
            }
            token if is_calc(token) => parse_calc(token)?,
            "COPY" => parse_copy(
                tokens.next().context("missing target registors")?,
                tokens.next().context("missing target registors")?,
            )?,
            token if is_cond(token) => parse_cond(token)?,
            _ => anyhow::bail!("Unexpected opcode name"),
        };
        stmts.push(stmt);
    }
    Ok(stmts)
}

fn parse_reg(reg: &str) -> Result<Reg> {
    // outputting IO registor reglerdress it is source or dest.
    Ok(match reg.to_uppercase().as_str() {
        "REG0" => Reg::REG0,
        "REG1" => Reg::REG1,
        "REG2" => Reg::REG2,
        "REG3" => Reg::REG3,
        "REG4" => Reg::REG4,
        "REG5" => Reg::REG5,
        "IO" => Reg::IO,
        "OUT" => Reg::IO,
        "IN" => Reg::IO,
        _ => anyhow::bail!("Unexpected Registor name"),
    })
}

fn is_calc(token: &str) -> bool {
    matches!(token, "OR" | "NAND" | "NOR" | "AND" | "ADD" | "SUB")
}
fn is_cond(token: &str) -> bool {
    matches!(
        token,
        "OFF" | "EQ" | "LS" | "LSEQ" | "ON" | "NEQ" | "GREQ" | "GR"
    )
}
fn parse_calc(operater: &str) -> Result<Stmt> {
    Ok(match operater {
        "OR" => Stmt::Calc(Operater::OR),
        "NAND" => Stmt::Calc(Operater::NAND),
        "NOR" => Stmt::Calc(Operater::NOR),
        "AND" => Stmt::Calc(Operater::AND),
        "ADD" => Stmt::Calc(Operater::ADD),
        "SUB" => Stmt::Calc(Operater::SUB),
        _ => unreachable!("pre check is needed"),
    })
}
fn parse_copy(dest: &str, src: &str) -> Result<Stmt> {
    Ok(Stmt::COPY(parse_reg(dest)?, parse_reg(src)?))
}
fn parse_cond(condition: &str) -> Result<Stmt> {
    Ok(match condition {
        "OFF" => Stmt::Cond(Condition::OFF),
        "EQ" => Stmt::Cond(Condition::EQ),
        "LS" => Stmt::Cond(Condition::LS),
        "LSEQ" => Stmt::Cond(Condition::LSEQ),
        "ON" => Stmt::Cond(Condition::ON),
        "NEQ" => Stmt::Cond(Condition::NEQ),
        "GREQ" => Stmt::Cond(Condition::GREQ),
        "GR" => Stmt::Cond(Condition::GR),
        _ => anyhow::bail!("Unexpected condition type"),
    })
}

fn solve_labels(stmts: &[Stmt]) -> Result<HashMap<&str, u8>> {
    let mut labels = HashMap::new();

    let mut i = 0;
    for stmt in stmts {
        match stmt {
            Stmt::Label(label) => {
                labels.insert(label.as_str(), i);
                if i > 1 << 5 {
                    anyhow::bail!(
                        "label \"@{}\" can't fit in 6 bit, for restriction of the architecture.",
                        label
                    );
                }
            }
            _ => i += 1,
        }
    }

    Ok(labels)
}

fn generate_code(stmts: &[Stmt]) -> Result<Vec<u8>> {
    let labels = solve_labels(stmts)?;

    let mut code: Vec<u8> = Vec::new();
    for stmt in stmts {
        let opcode = match stmt {
            Stmt::LET(immdiate) => match immdiate {
                Immdiate::Int(num) => *num,
                Immdiate::Label(label) => {
                    let pos = labels.get(label.as_str()).context("")?;
                    *pos
                }
            },
            Stmt::Calc(operater) => *operater as u8 + (1 << 6),
            Stmt::COPY(dest, src) => (*dest as u8) + ((*src as u8) << 3) + (2 << 6),
            Stmt::Cond(condition) => (*condition as u8) + (3 << 6),
            Stmt::Label(_) => continue,
        };
        code.push(opcode);
    }

    Ok(code)
}

fn write_output<W: Write>(mut out: W, stmts: &[Stmt], code: &[u8]) -> Result<()> {
    let mut code_it = code.iter();
    for stmt in stmts {
        if let Stmt::Label(lable) = stmt {
            write!(out, "    # @{lable}:")?;
        } else {
            let op = code_it
                .next()
                .expect("length of code and stmt is not consistent");
            write!(out, "{op:<3} # {stmt}")?;
        }
        out.write_all(b"\n")?;
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    let infile = (args.file.as_path() != Path::new("-")).then_some(args.file.as_path());

    let source: Box<dyn BufRead> = if let Some(file) = infile {
        Box::new(BufReader::new(File::open(file)?))
    } else {
        Box::new(io::stdin().lock())
    };

    let stmts = parse(source)?;

    let code = generate_code(&stmts)?;

    let mut outfile = args
        .out
        .unwrap_or_else(|| infile.unwrap_or(Path::new("stdin.os")).to_path_buf());
    outfile.set_extension("out");

    let out = BufWriter::new(File::create(outfile)?);
    write_output(out, &stmts, &code)?;

    Ok(())
}
