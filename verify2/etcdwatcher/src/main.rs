use etcd_client::{Client, Error, WatchOptions};
use rand::{distributions::Alphanumeric, Rng};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("etcdwatcher 시작 - SCENARIO/TEST2 키 모니터링");
    
    // etcd 클라이언트들 생성
    let mut watch_client = Client::connect(["http://localhost:2379"], None).await?;
    let mut put_client = Client::connect(["http://localhost:2379"], None).await?;
    
    // 초기 값 설정을 위한 별도 클라이언트
    let mut init_client = Client::connect(["http://localhost:2379"], None).await?;
    init_client.put("SCENARIO/TEST2", "initial_value", None).await?;
    sleep(Duration::from_millis(500)).await;
    
    // 값 변경 시간을 공유하기 위한 Arc<Mutex>
    let change_time = Arc::new(Mutex::new(0u64));
    let change_time_clone = Arc::clone(&change_time);
    
    // 채널을 통해 메인 스레드에서 PUT 작업 트리거
    let (tx, mut rx) = mpsc::channel::<String>(100);
    
    // Watch 스레드
    let watch_handle = {
        let change_time = Arc::clone(&change_time);
        tokio::spawn(async move {
            let (_watcher, mut watch_stream) = watch_client
                .watch("SCENARIO", Some(WatchOptions::new().with_prefix()))
                .await
                .expect("Failed to create watcher");
            
            println!("Watcher 시작됨");
            
            while let Some(_resp) = watch_stream.message().await.expect("Watch stream error") {
                // watch callback이 호출된 시점
                let callback_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos() as u64;
                
                // 값이 변경된 시점을 읽어옴
                let stored_change_time = {
                    let lock = change_time.lock().unwrap();
                    *lock
                };
                
                if stored_change_time > 0 {
                    // 소요 시간 계산 (나노초를 밀리초로 변환)
                    let elapsed_nanos = callback_time - stored_change_time;
                    let elapsed_ms = elapsed_nanos as f64 / 1_000_000.0;
                    
                    //println!("Watch callback 소요시간: {:.3}ms", elapsed_ms);
                    println!("{:.3}", elapsed_ms);
                    
                    // 시간 초기화
                    let mut lock = change_time.lock().unwrap();
                    *lock = 0;
                }
            }
        })
    };
    
    // PUT 작업을 수행하는 스레드
    let put_handle = tokio::spawn(async move {
        while let Some(new_value) = rx.recv().await {
            // 값 변경 직전 시간 기록
            let change_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64;
            
            {
                let mut lock = change_time_clone.lock().unwrap();
                *lock = change_timestamp;
            }
            
            // PUT 실행
            if let Err(e) = put_client.put("SCENARIO/TEST2", new_value.clone(), None).await {
                println!("PUT 오류: {}", e);
            } else {
                //println!("PUT 완료: {}", new_value);
                println!(" ");
            }
        }
    });
    
    // 메인 스레드에서 값 변경을 주기적으로 트리거
    tokio::spawn(async move {
        for _i in 1..=100 {
            sleep(Duration::from_secs(2)).await;
            
            // 30자리 랜덤 문자열 생성
            let random_value: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(30)
                .map(char::from)
                .collect();
            
            //println!("값 변경 #{}: {}", i, random_value);
            if let Err(_) = tx.send(random_value).await {
                break;
            }
        }
        
        // 종료를 위해 채널 닫기
        drop(tx);
    });
    
    // 스레드들이 완료될 때까지 대기
    tokio::select! {
        _ = watch_handle => println!("Watch 스레드 종료"),
        _ = put_handle => println!("PUT 스레드 종료"),
        _ = sleep(Duration::from_secs(300)) => {
            println!("타임아웃 - etcdwatcher 종료");
        }
    }
    
    Ok(())
}