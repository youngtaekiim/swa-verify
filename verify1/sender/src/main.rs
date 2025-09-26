// ----------- gRPC 클라이언트 모듈 -----------
use scenario_proto::scenario_service_client::ScenarioServiceClient;
use scenario_proto::{ScenarioNameRequest, ScenarioYamlRequest};
use std::time::{SystemTime, UNIX_EPOCH};

pub mod scenario_proto {
    tonic::include_proto!("scenario"); // scenario.proto에서 생성된 모듈
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 서버 주소
    let mut client = ScenarioServiceClient::connect("http://[::1]:50051").await?;

    for _i in 0..100 {
    // ----------- Alternative 1: 이름만 전달 -----------
    let timestamp_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    
    let name_request = ScenarioNameRequest {
        name: "scen-gear-p-left-window-close".to_string(),
        timestamp_nanos,
    };

    let response = client.send_name(name_request).await?.into_inner();
    //println!("[Alternative 1] Status: {}", response.status);
    //println!("[Alternative 1] Total Elapsed: {} microseconds", response.elapsed_ms);
    let elapsed_ms_1 = response.elapsed_ms;

    // ----------- Alternative 2: YAML 전체 전달 -----------
    let yaml_string = r#"
name: scen-gear-p-left-window-close
conditions:
  - gear == P
actions:
  - launch act-leftwindow-close
"#;

    let timestamp_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    let yaml_request = ScenarioYamlRequest {
        yaml: yaml_string.to_string(),
        timestamp_nanos,
    };

    let response = client.send_yaml(yaml_request).await?.into_inner();
    //println!("[Alternative 2] Status: {}", response.status);
    //println!("[Alternative 2] Total Elapsed: {} microseconds", response.elapsed_ms);
    let elapsed_ms_2 = response.elapsed_ms;
    println!("{},{}", elapsed_ms_1, elapsed_ms_2);
    std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    Ok(())
}
