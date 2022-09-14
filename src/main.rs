use clap::Parser;
use std::io::{self, Write};
use std::process::Command;

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    #[clap(short, long, value_parser)]
    erp: bool,

    #[clap(short, long, value_parser)]
    loja: bool,
}

fn main() {
    let args = Args::parse();

    println!("{:?}", args);

    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "echo hello"])
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("sh")
            .arg("-c")
            .arg("echo hello")
            .output()
            .expect("failed to execute process")
    };

    let hello = output.stdout;

    io::stdout().write_all(&hello).unwrap();
}
