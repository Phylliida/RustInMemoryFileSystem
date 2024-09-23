

use tokio;

#[no_mangle]
pub async extern "C" fn _start() {
    let mut handles = vec![];

    for i in 0..5 {
        let handle = tokio::spawn(async move {
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