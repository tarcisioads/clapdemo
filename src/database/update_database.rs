extern crate dotenv;

use dotenv::dotenv;
use std::env;
use std::path::PathBuf;
use home::home_dir;
use crate::utils::utils;
use std::process::Command;
use indicatif::ProgressBar;
use ssh2::Session;
use std::fs::File;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use crate::utils::utils::read_lines;


fn execute_script_sqlplus(db_name:&String, file_path: &str){
    dotenv().ok();

    let database_pwd = env::var("DATABASE_PWD").expect("DATABASE_PWD not set");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");


    let output = Command::new("sqlplus")
        .arg("-S")
        .arg(format!("{}/{}@{}",db_name, database_pwd, database_url)) 
        .arg(format!("@{}", file_path))
        .output()
        .expect("Failed to execute SQL*Plus");

    if output.status.success() {
        println!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        println!("Failed to execute ORDS script:\n{}", String::from_utf8_lossy(&output.stderr));
    }
}

fn send_backend_scripts() {
    dotenv().ok();

    let database_pwd = env::var("DATABASE_PWD").expect("DATABASE_PWD not set");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");


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

                let buffer = s.as_bytes();

                let sftp = sess.sftp().unwrap();

                let mut path_remote = "../../nb/backend/".to_owned();
                path_remote.push_str(&file);

                let file_remote = Path::new(&path_remote);

                sftp.create(&file_remote)
                    .unwrap()
                    .write_all(&buffer)
                    .unwrap();

                println!("File {} sent to server", file);

                let mut channel = sess.channel_session().unwrap();
                let cmd = format!("sqlplus NBDATA000178/{}@{} @{}", database_pwd, database_url, path_remote);
                channel.exec(cmd.as_str()).unwrap();
                let mut buffer = Vec::new();

                // read the whole file
                channel.read_to_end(&mut buffer).expect("falha ao pegar output");

                let s = unsafe { std::str::from_utf8_unchecked(&buffer) };
                println!("result: {}", s);
                let _ = channel.wait_close();
                println!("{}", channel.exit_status().unwrap());

            }
        }
    }

}

pub fn run_packages(db_name:&String) {
    send_backend_scripts();

    let mut path = PathBuf::new();
    let home = home_dir().expect("no home!");
    path.push(home.clone());
    path.push(&"nb/backend/DBADMIN.SQL");

    let mut path_home = PathBuf::new();
    path_home.push(home);

    println!("DBADMIN File {}", path.display());

    // File hosts must exist in current path before this produces output
    if let Ok(lines) = utils::read_lines(path.clone()) {
        // Consumes the iterator, returns an (Optional) String
        for line in lines {
            if let Ok(file) = line {
                let mut owned_string: String = "/nb/backend/".to_owned();

                if cfg!(target_os = "windows") {
                    owned_string.push_str(&file);
                } else {
                    owned_string.push_str(&file.replace("\\", "/"));
                }

                let mut file_path = String::new();
                file_path.push_str(path_home.clone().to_str().unwrap());
                file_path.push_str(&owned_string);
                println!("run script {}", file_path);

                execute_script_sqlplus(db_name, &file_path);

            }
        }
    }

}
