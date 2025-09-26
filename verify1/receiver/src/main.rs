use chrono::Utc;
use etcd_client::*;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tonic::{transport::Server, Request, Response, Status};

// ----------- 시나리오 구조 정의 -----------
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Scenario {
    pub name: String,
    pub conditions: Vec<String>,
    pub actions: Vec<String>,
}

// ----------- gRPC 정의 -----------
pub mod scenario_proto {
    tonic::include_proto!("scenario"); // .proto 파일에서 생성된 모듈
}

use scenario_proto::scenario_service_server::{ScenarioService, ScenarioServiceServer};
use scenario_proto::{ScenarioNameRequest, ScenarioYamlRequest, ScenarioResponse};

// ----------- gRPC 서버 구현 -----------
#[derive(Default)]
pub struct MyScenarioService {}

#[tonic::async_trait]
impl ScenarioService for MyScenarioService {
    async fn send_name(
        &self,
        request: Request<ScenarioNameRequest>,
    ) -> Result<Response<ScenarioResponse>, Status> {
        let req_inner = request.into_inner();
        let name = req_inner.name;
        let sender_timestamp_nanos = req_inner.timestamp_nanos;

        // etcd에서 조회 (데모용으로 간단히 시뮬레이션)
         let mut client = Client::connect(["http://localhost:2379"], None)
            .await
            .map_err(|e| Status::internal(format!("etcd error: {}", e)))?;
        let _resp = client.get(name.clone(), None).await;
                
        // 전체 시간 계산 (sender 전송 시작부터 receiver 처리 완료까지)
        let current_timestamp_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        let total_elapsed_nanos = current_timestamp_nanos - sender_timestamp_nanos;
        let total_elapsed_ms = total_elapsed_nanos / 1_000; // nanos to millis

        println!("Alternative 1 - 전체 처리 시간 (sender->receiver): {} microseconds", total_elapsed_ms);

        Ok(Response::new(ScenarioResponse {
            status: format!("Retrieved scenario '{}'", name),
            elapsed_ms: total_elapsed_ms,
        }))
    }

    async fn send_yaml(
        &self,
        request: Request<ScenarioYamlRequest>,
    ) -> Result<Response<ScenarioResponse>, Status> {
        let req_inner = request.into_inner();
        let yaml = req_inner.yaml;
        let sender_timestamp_nanos = req_inner.timestamp_nanos;

        // YAML 파싱
        let parsed: Result<Scenario, _> = serde_yaml::from_str(&yaml);
        
        // 전체 시간 계산 (sender 전송 시작부터 receiver 처리 완료까지)
        let current_timestamp_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        let total_elapsed_nanos = current_timestamp_nanos - sender_timestamp_nanos;
        let total_elapsed_ms = total_elapsed_nanos / 1_000; // nanos to millis

        println!("Alternative 2 - 전체 처리 시간 (sender->receiver): {} microseconds", total_elapsed_ms);

        Ok(Response::new(ScenarioResponse {
            status: format!("Parsed scenario '{}'", parsed.unwrap().name),
            elapsed_ms: total_elapsed_ms,
        }))
    }
}

// ----------- 서버 실행 -----------
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let service = MyScenarioService::default();

    println!("ScenarioService gRPC 서버 시작: {}", Utc::now());
    Server::builder()
        .add_service(ScenarioServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
