use std::{error::Error, net::TcpStream, io::{BufReader, BufRead}};
use serde::Deserialize;
use ssh2::{Session, Stream};
use chrono::prelude::*;
use std::io::Lines;
use lazy_static::*;
use toml::Value;

lazy_static! {
    static ref TOML_VALUE: Value = {
        let toml_str = std::fs::read_to_string("config.toml").expect("Unable to read config.toml");
        toml::from_str(&toml_str).expect("Unable to parse config.toml")
    };
    static ref HOST: &'static str = TOML_VALUE.get("host").unwrap().as_str().unwrap();
    static ref PORT: &'static str = TOML_VALUE.get("port").unwrap().as_str().unwrap();
    static ref USER: &'static str = TOML_VALUE.get("user").unwrap().as_str().unwrap();
    static ref PASS: &'static str = TOML_VALUE.get("pass").unwrap().as_str().unwrap();
    static ref DOKR: &'static str = TOML_VALUE.get("dokr").unwrap().as_str().unwrap();
}

#[derive(Debug, Deserialize, Clone)]
struct Record {
    person: String, 
    start: String, 
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let vec_rec = read_csv("queue.csv")?;
    scheduler(vec_rec).await?;
    Ok(())
}

fn read_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(file_path)?;
    let mut rec_vec: Vec<Record> = vec![];
    for result in rdr.deserialize() {
        let record: Record = result?;
        rec_vec.push(record);
    }
    Ok(rec_vec)
}

fn ssh_exec_cmd(command: &str) -> Result<Lines<BufReader<Stream>>, Box<dyn Error>> {
    let host = HOST.to_string() + ":" + &PORT;
    let tcp = TcpStream::connect(host)?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;
    sess.userauth_password(&USER, &PASS)?;
    let mut channel = sess.channel_session()?;
    channel.exec(command)?;
    channel.handle_extended_data(ssh2::ExtendedData::Merge)?;
    let out = BufReader::new(channel.stream(0));
    Ok(out.lines())
}

fn get_all_users() -> Result<Vec<String>, Box<dyn Error>> {
    let command = "getent passwd {1000..6000} | cut -d: -f1";
    let out = ssh_exec_cmd(&command)?;
    let mut users: Vec<String> = vec![];
    out.for_each(|line| {
        users.push(line.unwrap())
    });
    Ok(users)
}

fn disable_user(username: &str) -> Result<(), Box<dyn Error>> {
    let command = "chage -E0 ".to_string() + username;
    ssh_exec_cmd(&command)?;
    Ok(())
}

fn enable_user(username: &str) -> Result<(), Box<dyn Error>> {
    let command = "chage -E-1 ".to_string() + username;
    ssh_exec_cmd(&command)?;
    Ok(())
}

fn disable_all_users() -> Result<(), Box<dyn Error>> {
    let users = get_all_users()?;
    for user in users {
        disable_user(&user)?;
    }
    Ok(())
}

fn enable_all_users() -> Result<(), Box<dyn Error>> {
    let users = get_all_users()?;
    for user in users {
        enable_user(&user)?;
    }
    Ok(())
}

fn restart_docker() -> Result<(), Box<dyn Error>> {
    let command = "docker restart ".to_string() + &DOKR;
    ssh_exec_cmd(&command)?;
    Ok(())
}

async fn scheduler(vec_rec: Vec<Record>) -> Result<(), Box<dyn Error>> {
    let now = Local::now().with_nanosecond(0).unwrap();
    let mut handles = vec![];

    let person_end = vec_rec[vec_rec.len() - 1].person.as_str();
    if person_end != "__END" {
        panic!("The last item must be __END.");
    }

    let time_end = vec_rec[vec_rec.len() - 1].start.as_str();
    let end_duration = Local.datetime_from_str(time_end, "%Y-%m-%d %H:%M:%S")? - now;
    let end_duration = std::time::Duration::from_millis(end_duration.num_milliseconds() as u64);
    let end_sheduler = async move {
        tokio::time::sleep(end_duration).await;
        enable_all_users().unwrap();
        println!("__END: Enable all users at {}.", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
    };
    let end_handle = tokio::spawn(end_sheduler);

    let vec_rec = vec_rec[0..(vec_rec.len() - 1)].to_vec();
    for record in vec_rec {
        let person = record.person;
        let start_duration = Local.datetime_from_str(&record.start, "%Y-%m-%d %H:%M:%S")? - now;
        let start_duration = std::time::Duration::from_millis(start_duration.num_milliseconds() as u64);
        
        if &person != "__ALL" {
            let start_scheduler = async move {
                tokio::time::sleep(start_duration).await;
                restart_docker().unwrap();
                println!("{}: Docker restart at {}.", &person, chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
                disable_all_users().unwrap();
                println!("{}: Disable all users at {}.", &person, chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
                enable_user(&person).unwrap();
                println!("{}: Enable user {} at {}.", &person, &person, chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
            };
            let start_handle = tokio::spawn(start_scheduler);
            handles.push(start_handle);    
        } else {
            let start_scheduler = async move {
                tokio::time::sleep(start_duration).await;
                restart_docker().unwrap();
                println!("__ALL: Docker restart at {}.", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
                enable_all_users().unwrap();
                println!("__ALL: Enable all users at {}.", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
            };
            let start_handle = tokio::spawn(start_scheduler);
            handles.push(start_handle);
        }
    }
    
    handles.push(end_handle);
    
    for handle in handles {
        handle.await.unwrap();
    }

    Ok(())
}

