use etcd_client::{Client, Error};
use rand::{distributions::Alphanumeric, Rng};
use std::time::Instant;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // etcd 클라이언트 연결
    let mut client = Client::connect(["http://localhost:2379"], None).await?;
    
    println!("etcdtester 시작 - SCENARIO/TEST1 키로 100회 PUT/GET 테스트");
    println!("PUT시간(ms)\tGET시간(ms)");
    
    for i in 1..=100 {
        // 30자리 랜덤 문자열 생성
        let random_value: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();
        
        // PUT 작업 시간 측정
        let put_start = Instant::now();
        client.put("SCENARIO/TEST1", random_value.clone(), None).await?;
        let put_elapsed = put_start.elapsed().as_micros();
        
        // GET 작업 시간 측정
        let get_start = Instant::now();
        let _response = client.get("SCENARIO/TEST1", None).await?;
        let get_elapsed = get_start.elapsed().as_micros();
        
        // 결과 출력
        println!("{}\t{}", put_elapsed, get_elapsed);
        
        // 마지막 반복이 아니면 1초 대기
        if i < 100 {
            sleep(Duration::from_secs(1)).await;
        }
    }
    
    println!("etcdtester 완료");
    Ok(())
}
