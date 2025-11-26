use clap::Parser;
use rand::Rng;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::time::{interval, Duration};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Number of unique random transponders to simulate
    #[arg(short, long, default_value_t = 10)]
    transponders: u32,

    /// Interval between passings in seconds
    #[arg(short, long, default_value_t = 1.0)]
    interval: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let addr = "0.0.0.0:3601";
    let listener = TcpListener::bind(addr).await?;

    println!("Mock Decoder listening on {}", addr);
    println!("Simulating {} transponders every {} seconds", args.transponders, args.interval);

    // Generate random transponders
    let mut rng = rand::thread_rng();
    let mut transponder_list = Vec::new();
    for _ in 0..args.transponders {
        transponder_list.push(format!("{}", rng.gen_range(10000..99999)));
    }
    let transponders = Arc::new(transponder_list);

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("New connection from {}", addr);
        
        let transponders = transponders.clone();
        let interval_secs = args.interval;

        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();
            let mut push_enabled = false;
            
            // Create interval but don't tick it yet if we don't need to?
            // Actually, we can just tick it and ignore if not enabled.
            let mut timer = interval(Duration::from_secs_f64(interval_secs));
            // First tick completes immediately
            timer.tick().await;

            let mut passing_number = 1;

            loop {
                tokio::select! {
                    res = reader.read_line(&mut line) => {
                        match res {
                            Ok(0) => break, // EOF
                            Ok(_) => {
                                let msg = line.trim();
                                if !msg.is_empty() {
                                    println!("[{}] Received: {}", addr, msg);
                                    
                                    if msg == "GETMODE" {
                                        let _ = writer.write_all(b"GETMODE;OPERATION\r\n").await;
                                    } else if msg.starts_with("SETPROTOCOL") {
                                        let _ = writer.write_all(b"SETPROTOCOL;2.0\r\n").await;
                                    } else if msg.starts_with("SETPUSHPASSINGS") {
                                        let parts: Vec<&str> = msg.split(';').collect();
                                        if parts.len() >= 2 && parts[1] == "1" {
                                            let _ = writer.write_all(b"SETPUSHPASSINGS;1\r\n").await;
                                            push_enabled = true;
                                        } else {
                                            let _ = writer.write_all(b"SETPUSHPASSINGS;0\r\n").await;
                                            push_enabled = false;
                                        }
                                    } else if msg == "PING" {
                                        // Ignore
                                    }
                                }
                                line.clear();
                            }
                            Err(e) => {
                                eprintln!("Error reading from {}: {}", addr, e);
                                break;
                            }
                        }
                    }
                    _ = timer.tick() => {
                        if push_enabled {
                            let transponder = {
                                let mut rng = rand::thread_rng();
                                transponders.get(rng.gen_range(0..transponders.len())).cloned()
                            };

                            if let Some(transponder) = transponder {
                                let now = chrono::Local::now();
                                let date_str = now.format("%Y-%m-%d");
                                let time_str = now.format("%H:%M:%S.%3f");
                                
                                let passing_msg = format!(
                                    "#P;{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{}\r\n",
                                    passing_number, // PassingNo
                                    transponder,    // Transponder
                                    date_str,       // Date
                                    time_str,       // Time
                                    "123456",       // EventID
                                    "10",           // Hits
                                    "-50",          // MaxRSSI
                                    "0000",         // InternalData
                                    "1",            // IsActive
                                    "1",            // Channel
                                    "1",            // LoopID
                                    "1",            // LoopIDWakeup
                                    "3.0",          // Battery
                                    "25",           // Temperature
                                    "0000",         // InternalActiveData
                                    "30",           // BoxTemp
                                    "0"             // BoxReaderID
                                );
                                println!("[{}] Sending: {}", addr, passing_msg.trim());
                                if let Err(e) = writer.write_all(passing_msg.as_bytes()).await {
                                    eprintln!("Error writing to {}: {}", addr, e);
                                    break;
                                }
                                passing_number += 1;
                            }
                        }
                    }
                }
            }
            println!("Connection from {} closed", addr);
        });
    }
}
