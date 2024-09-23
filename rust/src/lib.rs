

use tokio;

use std::{thread, time::Duration};


#[no_mangle]
pub extern "C" fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async { async_fn().await });
}

// these all execute in sequence (not as seperate threads) because tokio doesn't yet support "rt-multi-thread" 
async fn async_fn() {
    let mut handles = vec![];

    for i in 0..5 {
        let handle = tokio::spawn(async move {
            println!("Task {} is running!", i);

            thread::sleep(Duration::from_millis(1000));
            println!("Task {} is running!", i);

        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

/*
#[no_mangle]
pub extern "C" fn greet() {
    println!("Hello, World!");
    std::thread::sleep(std::time::Duration::from_secs(1));
    println!("Goodbye");
}
*/