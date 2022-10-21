use clap::Parser;
use home::home_dir;
use indicatif::ProgressBar;
use ssh2::Session;
use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::net::TcpStream;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    #[clap(short, long, value_parser)]
    nobuild: bool,

    #[clap(short, long, value_parser)]
    erp: bool,

    #[clap(short, long, value_parser)]
    loja: bool,
}

fn build(folder: &str) {
    let mut path = PathBuf::new();

    if cfg!(target_os = "windows") {
        path.push("c:/");
    } else {
        let home = home_dir().expect("no home!");
        path.push(home);
    }
    path.push("nb/app/");
    path.push(folder);

    if path.is_dir() {
        println!("{}", path.display());
    }

    assert!(env::set_current_dir(&path).is_ok());

    println!("build nb/app/erp");

    let output = Command::new("npm")
        .arg("run")
        .arg("build")
        .output()
        .expect("failed to execute process");

    let hello = output.stdout;

    io::stdout().write_all(&hello).unwrap();
}

fn send(folder: &str) {
    // Connect to the local SSH server
    let tcp = TcpStream::connect("ec2-52-202-145-226.compute-1.amazonaws.com:22").unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();

    sess.userauth_password("atualizacao", "@1643Tar1643")
        .unwrap();
    let authenticated = sess.authenticated();

    println!("authenticated {:?}", authenticated);

    let mut path = PathBuf::new();
    if cfg!(target_os = "windows") {
        path.push("c:/");
    } else {
        let home = home_dir().expect("no home!");
        path.push(home);
    }
    let mut owned_string: String = "nb/app/".to_owned();
    owned_string.push_str(folder);
    owned_string.push_str(&"/dist/build.js");
    path.push(owned_string);

    println!("{}", path.display());

    let source = File::open(path);
    let metadata = source.metadata().unwrap();
    let pb = ProgressBar::new(metadata.len());
    let contents = pb.wrap_read(source);

    println!("read build file {:?}", contents);

    // Write the file
    let mut remote_file = sess
        .scp_send(
            Path::new("../../inetpub/wwwroot/app/dist/build.js"),
            0o644,
            10,
            None,
        )
        .unwrap();
    remote_file.write(contents);
    // Close the channel and wait for the whole content to be tranferred
    remote_file.send_eof().unwrap();
    remote_file.wait_eof().unwrap();
    remote_file.close().unwrap();
    remote_file.wait_close().unwrap();
}

fn main() {
    let args = Args::parse();

    println!("{:?}", args);

    if !args.nobuild {
        if args.erp {
            build("erp");
        }
        if args.loja {
            build("loja");
        }
    }

    if args.erp {
        send("erp");
    }
    if args.loja {
        send("loja");
    }

    println!("finish");
}
