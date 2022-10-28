use clap::Parser;
use home::home_dir;
use indicatif::ProgressBar;
use ssh2::Session;
use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
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

    Command::new("npm")
        .arg("run")
        .arg("build")
        .spawn()
        .expect("Erro ao executar build no projeto");
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

    println!("Arquivo: {}", path.display());

    let source = File::open(path).expect("Erro ao carregar arquivo build");
    let mut len = 0;
    if let Ok(metadata) = source.metadata() {
        len = metadata.len();
    }
    println!("Len {}", len);
    let pb = ProgressBar::new(len);

    let mut buffer = Vec::new();
    io::copy(&mut pb.wrap_read(source), &mut buffer);

    let s = match std::str::from_utf8(&buffer) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    println!("buffer len {}", buffer.len());
    println!("buffer len {}", s.len() as u64);

    // Write the file
    let mut remote_file = sess
        .scp_send(
            Path::new("../../inetpub/wwwroot/app/dist/build2.js"),
            0o644,
            len as u64,
            None,
        )
        .unwrap();

    remote_file.write(&buffer);
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
