use chrono::prelude::*;

pub fn chrono_example() -> Result<(), Box<dyn std::error::Error>> {
    let dt = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    println!("Hello {}!", dt);

    println!("Hello {}!", chrono::Utc::now().timestamp());


    let last_second = dt - chrono::Duration::seconds(1);
    println!("Goodbye {}!", last_second);
    
    Ok(())
}
