use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;

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

#[derive(Debug)]
enum Immdiate {
    Int(u8),
    Label(String),
}

#[derive(Debug)]
enum Operater {
    OR,
    NAND,
    NOR,
    AND,
    ADD,
    SUB,
}

#[derive(Debug)]
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

#[derive(Debug)]
enum Reg {
    REG0,
    REG1,
    REG2,
    REG3,
    REG4,
    REG5,
    IO,
}

#[derive(Debug)]
enum Stmt {
    LET(Immdiate),
    Calc(Operater, Reg),
    COPY(Reg, Reg),
    Cond(Condition),
    Label(String),
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
                        Err(anyhow::anyhow!(
                            "Immdiate number must less than 2^6. :{:?}",
                            operand
                        ))?;
                    } else if operand < 0 {
                        Err(anyhow::anyhow!(
                            "Immdiate number must be non negative. :{:?}",
                            operand
                        ))?;
                    }
                    Stmt::LET(Immdiate::Int(operand as u8))
                }
            }
            token if is_calc(token) => {
                parse_calc(token, tokens.next().context("missing a target registor")?)?
            }
            "COPY" => parse_copy(
                tokens.next().context("missing target registors")?,
                tokens.next().context("missing target registors")?,
            )?,
            token if is_cond(token) => parse_cond(token)?,
            _ => todo!(),
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
        _ => Err(anyhow::anyhow!("Unexpected Registor name"))?,
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
fn parse_calc(operater: &str, target: &str) -> Result<Stmt> {
    let reg = parse_reg(target)?;
    Ok(match operater {
        "OR" => Stmt::Calc(Operater::OR, reg),
        "NAND" => Stmt::Calc(Operater::NAND, reg),
        "NOR" => Stmt::Calc(Operater::NOR, reg),
        "AND" => Stmt::Calc(Operater::AND, reg),
        "ADD" => Stmt::Calc(Operater::ADD, reg),
        "SUB" => Stmt::Calc(Operater::SUB, reg),
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
        _ => Err(anyhow::anyhow!("Unexpected condition type"))?,
    })
}

fn main() -> Result<()> {
    let args = Args::parse();

    let source: Box<dyn BufRead> = if args.file.as_path() == Path::new("-") {
        Box::new(io::stdin().lock())
    } else {
        Box::new(BufReader::new(File::open(args.file.as_path())?))
    };

    let stmts = parse(source)?;
    dbg!(stmts);

    Ok(())
}
