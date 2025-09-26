use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use tonic::transport::{Server, Identity, ServerTlsConfig};
use tonic::{Request, Response, Status};

// Generated proto code
pub mod benchmark {
    tonic::include_proto!("benchmark");
}

use benchmark::benchmark_service_server::{BenchmarkService, BenchmarkServiceServer};
use benchmark::{BenchmarkRequest, BenchmarkResponse};

#[derive(Default)]
pub struct BenchmarkServer {}

#[tonic::async_trait]
impl BenchmarkService for BenchmarkServer {
    async fn send_timestamp(
        &self,
        request: Request<BenchmarkRequest>,
    ) -> Result<Response<BenchmarkResponse>, Status> {
        let req = request.into_inner();
        let sender_timestamp_nanos = req.timestamp_nanos;
        
        // Calculate elapsed time in microseconds
        let current_timestamp_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| Status::internal(format!("Time error: {}", e)))?
            .as_nanos() as u64;
        
        let elapsed_nanos = current_timestamp_nanos - sender_timestamp_nanos;
        let elapsed_microseconds = elapsed_nanos / 1_000;
        
        let response = BenchmarkResponse {
            elapsed_microseconds,
            success: true,
        };
        
        Ok(Response::new(response))
    }
}

fn load_tls_identity() -> Result<Identity, Box<dyn std::error::Error>> {
    let cert = fs::read("certs/server.crt")?;
    let key = fs::read("certs/server.key")?;
    let identity = Identity::from_pem(cert, key);
    Ok(identity)
}

async fn run_with_tls() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse()?;
    let benchmark_service = BenchmarkServer::default();
    
    println!("Benchmark Receiver starting with TLS on {}", addr);
    let identity = load_tls_identity()?;
    let tls = ServerTlsConfig::new().identity(identity);
    
    Server::builder()
        .tls_config(tls)?
        .add_service(BenchmarkServiceServer::new(benchmark_service))
        .serve(addr)
        .await?;
    
    Ok(())
}

async fn run_without_tls() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse()?;
    let benchmark_service = BenchmarkServer::default();
    
    println!("Benchmark Receiver starting without TLS on {}", addr);
    
    Server::builder()
        .add_service(BenchmarkServiceServer::new(benchmark_service))
        .serve(addr)
        .await?;
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let use_tls = args.len() > 1 && args[1] == "--tls";
    
    if use_tls {
        run_with_tls().await
    } else {
        run_without_tls().await
    }
}