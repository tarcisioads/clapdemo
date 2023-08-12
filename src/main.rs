mod database;
mod utils;

use clap::Parser;
use home::home_dir;
use indicatif::ProgressBar;
use ssh2::Session;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufRead;
use std::io::{self, Write};
use std::net::TcpStream;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use oracle::{Connection};

#[cfg(windows)]
pub const NPM: &'static str = "npm.cmd";

#[cfg(not(windows))]
pub const NPM: &'static str = "npm";

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

    #[clap(short, long)]
    packages: Option<String>,
}


fn update_procedures_database() {

    let conn = Connection::connect("nbapp","1", "notabrasil-2.cgvmwsljznim.us-east-1.rds.amazonaws.com").expect("falha ao conectar na base de dados"); 

    conn.execute("delete from metalog;", &[]).expect("falha ao excluir metalog");

    conn.commit().expect("falha no commit excluir metalog");

    conn.execute("BEGIN DBADMIN.UPDATE_PROCEDURES; END;/", &[]).expect("falha ao atualizar bases de dados");

    conn.commit().expect("falha no commit atualizar bases de dados");

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
    println!("{}", env::current_dir().unwrap().to_string_lossy());

    Command::new(NPM)
        .arg("run")
        .arg("build")
        .spawn()
        .unwrap()
        .wait()
        .expect("Erro ao executar build no projeto");

    println!("finish build");
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(std::io::BufReader::new(file).lines())
}

fn send_backend_scripts() {
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
        path.push("c:");
    } else {
        let home = home_dir().expect("no home!");
        path.push(home);
    }

    let mut path_dbadmin = path.to_owned();
    path_dbadmin.push(&"nb/backend/DBADMIN.SQL");

    println!("DBADMIN File {}", path_dbadmin.display());

    // File hosts must exist in current path before this produces output
    if let Ok(lines) = read_lines(path_dbadmin) {
        // Consumes the iterator, returns an (Optional) String
        for line in lines {
            if let Ok(file) = line {
                println!("Send file {} to server", file);
                let mut owned_string: String = "/nb/backend/".to_owned();

                if cfg!(target_os = "windows") {
                    owned_string.push_str(&file);
                } else {
                    owned_string.push_str(&file.replace("\\", "/"));
                }

                let mut file_path = String::new();
                file_path.push_str(path.to_str().unwrap());
                file_path.push_str(&owned_string);

                let source = File::open(file_path).expect("Erro ao carregar arquivo ");
                let mut len = 0;
                if let Ok(metadata) = source.metadata() {
                    len = metadata.len();
                }
                let pb = ProgressBar::new(len);

                let mut buffer = Vec::new();
                io::copy(&mut pb.wrap_read(source), &mut buffer).unwrap();

                let s = String::from_utf8(buffer).expect("Found invalid UTF-8");

                let s2 = s.replace("set define off;", "");

                let buffer = s2.as_bytes();

                let sftp = sess.sftp().unwrap();

                let mut path_remote = "../../nb/backend/".to_owned();
                path_remote.push_str(&file);

                let file_remote = Path::new(&path_remote);

                sftp.create(&file_remote)
                    .unwrap()
                    .write_all(&buffer)
                    .unwrap();

                println!("File {} sent to server", file);
            }
        }
    }

    let file_path: String = "/nb/backend/_________SCRIPT VIEWS.sql".to_owned();

    println!("Send file {} to server", file_path);

    let source = File::open(file_path).expect("Erro ao carregar arquivo ");
    let mut len = 0;
    if let Ok(metadata) = source.metadata() {
        len = metadata.len();
    }
    let pb = ProgressBar::new(len);

    let mut buffer = Vec::new();
    io::copy(&mut pb.wrap_read(source), &mut buffer).unwrap();

    let s = String::from_utf8(buffer).expect("Found invalid UTF-8");

    let s2 = s.replace("set define off;", "");

    let buffer = s2.as_bytes();

    let sftp = sess.sftp().unwrap();

    let path_remote = "../../nb/nbadmin/scripts/views.sql".to_owned();

    let file_remote = Path::new(&path_remote);

    sftp.create(&file_remote)
        .unwrap()
        .write_all(&buffer)
        .unwrap();

    println!("File {} sent to server", path_remote);


    let file_path: String = "/nb/backend/_________SCRIPT SEQUENCES.sql".to_owned();

    println!("Send file {} to server", file_path);

    let source = File::open(file_path).expect("Erro ao carregar arquivo ");
    let mut len = 0;
    if let Ok(metadata) = source.metadata() {
        len = metadata.len();
    }
    let pb = ProgressBar::new(len);

    let mut buffer = Vec::new();
    io::copy(&mut pb.wrap_read(source), &mut buffer).unwrap();

    let s = String::from_utf8(buffer).expect("Found invalid UTF-8");

    let s2 = s.replace("set define off;", "");

    let buffer = s2.as_bytes();

    let sftp = sess.sftp().unwrap();

    let path_remote = "../../nb/nbadmin/scripts/sequences.sql".to_owned();

    let file_remote = Path::new(&path_remote);

    sftp.create(&file_remote)
        .unwrap()
        .write_all(&buffer)
        .unwrap();

    println!("File {} sent to server", path_remote);

}

fn update_webdata() {
    // Connect to the local SSH server
    let tcp = TcpStream::connect("ec2-52-202-145-226.compute-1.amazonaws.com:22").unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();

    sess.userauth_password("atualizacao", "@1643Tar1643")
        .unwrap();
    let authenticated = sess.authenticated();

    println!("authenticated {:?}", authenticated);

    let mut channel = sess.channel_session().unwrap();
    channel.exec("c:\\nb\\nbadmin\\bin\\nbadmin.exe -v -server NBTESTE -schema WEBDATA -path c:\\nb\\backend -update_schema").unwrap();
    let mut buffer = Vec::new();

    // read the whole file
    channel
        .read_to_end(&mut buffer)
        .expect("falha ao pegar output");

    let s = unsafe { std::str::from_utf8_unchecked(&buffer) };
    println!("result: {}", s);
    let _ = channel.wait_close();
    println!("{}", channel.exit_status().unwrap());
}

fn update_nbdata(nbdata:String) {

    println!("Atualizacao: NBDATA000{}", nbdata);

    // Connect to the local SSH server
    let tcp = TcpStream::connect("ec2-52-202-145-226.compute-1.amazonaws.com:22").unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();

    sess.userauth_password("atualizacao", "@1643Tar1643")
        .unwrap();
    let authenticated = sess.authenticated();

    println!("authenticated {:?}", authenticated);

    let mut channel = sess.channel_session().unwrap();
    let command = format!("c:\\nb\\nbadmin\\bin\\nbadmin.exe -v -server NBTESTE -schema NBDATA000{} -path c:\\nb\\backend -update_schema", nbdata);
    channel.exec(command.as_str()).unwrap();
    let mut buffer = Vec::new();

    // read the whole file
    channel
        .read_to_end(&mut buffer)
        .expect("falha ao pegar output");

    let s = unsafe { std::str::from_utf8_unchecked(&buffer) };
    println!("result: {}", s);
    let _ = channel.wait_close();
    println!("{}", channel.exit_status().unwrap());
}


fn update_database() {
    println!("start update database");

    send_backend_scripts();

    update_webdata();

    for n in 179..210 {
        update_nbdata(n.to_string());
    }

    update_procedures_database();

    println!("finish update database");
}

fn send(folder: &str) {
    println!("start send frontend");
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
    let pb = ProgressBar::new(len);

    let mut buffer = Vec::new();
    io::copy(&mut pb.wrap_read(source), &mut buffer).unwrap();

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

    println!("finish send frontend");
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

    if args.database {
        update_database();
    }

    if let Some(database) = &args.packages {
        database::update_database::run_packages(database);
               
        println!("finish run packages");
    }

    println!("finish");
}
