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

    #[clap(short, long, value_parser)]
    database: bool,
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

    println!("start build");

    Command::new("npm")
        .arg("run")
        .arg("build")
        .spawn()
        .unwrap()
        .wait()
        .expect("Erro ao executar build no projeto");

    println!("finish build");
}

fn update_webdata() {
    let mut path = PathBuf::new();

    if cfg!(target_os = "windows") {
        path.push("c:/");
    } else {
        let home = home_dir().expect("no home!");
        path.push(home);
    }
    path.push("nb/app/");

    if path.is_dir() {
        println!("{}", path.display());
    }

    assert!(env::set_current_dir(&path).is_ok());

    println!("start update database");

    Command::new("npm")
        .arg("run")
        .arg("build")
        .spawn()
        .unwrap()
        .wait()
        .expect("Erro ao executar build no projeto");

    println!("finish update database");
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
    io::copy(&mut pb.wrap_read(source), &mut buffer).unwrap();

    println!("buffer len {}", buffer.len());

    let sftp = sess.sftp().unwrap();

    let file_remote = match folder {
        "loja" => Path::new("../../inetpub/wwwroot/loja/dist/build.js"),
        "erp" => Path::new("../../inetpub/wwwroot/app/dist/build.js"),
        _ => Path::new("../../inetpub/wwwroot/app/dist/build.js"),
    };

    sftp.create(&file_remote)
        .unwrap()
        .write_all(&buffer)
        .unwrap();
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
