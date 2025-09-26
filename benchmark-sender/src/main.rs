use clap::Parser;
use std::fs;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;
use tonic::transport::{Channel, Certificate, ClientTlsConfig, Endpoint};

// Generated proto code
pub mod benchmark {
    tonic::include_proto!("benchmark");
}

use benchmark::benchmark_service_client::BenchmarkServiceClient;
use benchmark::BenchmarkRequest;

#[derive(Parser)]
#[command(name = "benchmark-sender")]
#[command(about = "gRPC mTLS performance benchmark sender")]
struct Args {
    #[arg(long, help = "Use TLS/SSL")]
    tls: bool,
    
    #[arg(long, default_value = "100", help = "Number of requests to send")]
    requests: usize,
    
    #[arg(long, default_value = "10", help = "Delay between requests in milliseconds")]
    delay: u64,
    
    #[arg(long, default_value = "", help = "Payload size (empty, small, medium, large)")]
    payload: String,
}

fn generate_payload(size: &str) -> String {
    match size {
        "small" => "x".repeat(100),
        "medium" => "x".repeat(1000),
        "large" => "x".repeat(10000),
        _ => String::new(),
    }
}

fn load_ca_cert() -> Result<Certificate, Box<dyn std::error::Error>> {
    let ca_cert = fs::read("certs/ca.crt")?;
    Ok(Certificate::from_pem(ca_cert))
}

async fn create_client(use_tls: bool) -> Result<BenchmarkServiceClient<Channel>, Box<dyn std::error::Error>> {
    if use_tls {
        let ca_cert = load_ca_cert()?;
        let tls = ClientTlsConfig::new()
            .ca_certificate(ca_cert)
            .domain_name("localhost");
        
        let endpoint = Endpoint::from_static("https://localhost:50051")
            .tls_config(tls)?;
        
        let channel = endpoint.connect().await?;
        Ok(BenchmarkServiceClient::new(channel))
    } else {
        let channel = Endpoint::from_static("http://localhost:50051")
            .connect()
            .await?;
        Ok(BenchmarkServiceClient::new(channel))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("Starting benchmark sender...");
    println!("TLS enabled: {}", args.tls);
    println!("Requests: {}", args.requests);
    println!("Delay: {}ms", args.delay);
    println!("Payload: {}", if args.payload.is_empty() { "none" } else { &args.payload });
    
    let mut client = create_client(args.tls).await?;
    let payload = generate_payload(&args.payload);
    
    let mut results = Vec::new();
    let mut errors = 0;
    
    println!("\nStarting benchmark...");
    println!("microseconds");
    
    for i in 0..args.requests {
        let timestamp_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_nanos() as u64;
        
        let request = BenchmarkRequest {
            timestamp_nanos,
            payload: payload.clone(),
        };
        
        match client.send_timestamp(request).await {
            Ok(response) => {
                let elapsed_us = response.into_inner().elapsed_microseconds;
                results.push(elapsed_us);
                println!("{}", elapsed_us);
            }
            Err(e) => {
                errors += 1;
                eprintln!("Request {} failed: {}", i + 1, e);
            }
        }
        
        if i < args.requests - 1 && args.delay > 0 {
            sleep(Duration::from_millis(args.delay)).await;
        }
    }
    
    // Calculate statistics
    if !results.is_empty() {
        results.sort();
        let len = results.len();
        let min = results[0];
        let max = results[len - 1];
        let avg = results.iter().sum::<u64>() / len as u64;
        let median = results[len / 2];
        let p95 = results[(len as f64 * 0.95) as usize];
        let p99 = results[(len as f64 * 0.99) as usize];
        
        println!("\n=== Benchmark Results ===");
        println!("Total requests: {}", args.requests);
        println!("Successful: {}", results.len());
        println!("Errors: {}", errors);
        println!("Min: {}μs", min);
        println!("Max: {}μs", max);
        println!("Average: {}μs", avg);
        println!("Median: {}μs", median);
        println!("95th percentile: {}μs", p95);
        println!("99th percentile: {}μs", p99);
    } else {
        println!("No successful requests!");
    }
    
    Ok(())
}