extern crate dotenv;

use dotenv::dotenv;
use std::env;
use home::home_dir;
use indicatif::ProgressBar;
use ssh2::Session;
use std::fs::File;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use crate::utils::utils::read_lines;


fn send_backend_scripts(db_name:&String, log:bool) {
    dotenv().ok();

    let database_pwd = env::var("DATABASE_PWD").expect("DATABASE_PWD not set");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let server_url = env::var("SERVER_URL").expect("SERVER_URL not set");
    let server_usr = env::var("SERVER_USR").expect("SERVER_USR not set");
    let server_pwd = env::var("SERVER_PWD").expect("SERVER_PWD not set");


    // Connect to the local SSH server
    let tcp = TcpStream::connect(server_url).unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();

    sess.userauth_password(&server_usr, &server_pwd)
        .unwrap();
    let authenticated = sess.authenticated();

    println!("authenticated {:?}", authenticated);

    let home_path = home_dir().expect("no home!");
    let path = format!("{}", home_path.to_str().unwrap());

    let dbadmin_path = format!("{}/nb/backend/DBADMIN.SQL", path); 

    println!("DBADMIN File {}", dbadmin_path);

    // File hosts must exist in current path before this produces output
    if let Ok(lines) = read_lines(dbadmin_path.clone()) {
        let count = read_lines(dbadmin_path).unwrap().count() as u64;
        println!("{} files", count);
        let bar = ProgressBar::new(count);
        // Consumes the iterator, returns an (Optional) String
        for line in lines {
            if let Ok(file) = line {

                let file_path = format!("{}/nb/backend/{}", path, file.replace("\\", "/"));

                if log {
                    println!("Send file {} to server", file_path);
                }   

                let mut source = File::open(file_path).expect("Erro ao carregar arquivo ");

                let mut buffer = Vec::new();
                io::copy(&mut source, &mut buffer).unwrap();

                let s = String::from_utf8(buffer).expect("Found invalid UTF-8");

                let buffer = s.as_bytes();

                let sftp = sess.sftp().unwrap();

                let path_remote = format!(r"c:\nb\backend\{}", file);

                let file_remote = Path::new(&path_remote);

                sftp.create(&file_remote)
                    .unwrap()
                    .write_all(&buffer)
                    .unwrap();

                if log {
                    println!("File {} sent to server", file);
                }

                let mut channel = sess.channel_session().unwrap();
                let cmd = format!("echo exit | sqlplus {}/{}@{} @{}", db_name, database_pwd, database_url, path_remote.replace("/","\\"));
                if log {
                    println!("run script {} on server", cmd);
                }
                channel.exec(cmd.as_str()).unwrap();
                let mut buffer = Vec::new();

                // read the whole file
                channel
                    .read_to_end(&mut buffer)
                    .expect("falha ao pegar output");

                let s = unsafe { std::str::from_utf8_unchecked(&buffer) };
                if log {
                    println!("{}", s);
                }
 
                let _ = channel.wait_close();
                if log {
                    println!("{}", channel.exit_status().unwrap());
                }


            }
            bar.inc(1);
        }
        bar.finish();
    }

}

pub fn run_packages(db_name:&String, log:bool) {
    send_backend_scripts(db_name, log);
}
