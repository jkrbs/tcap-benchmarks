use csv::Writer;
use tokio::sync::Mutex;
use std::{fs::OpenOptions, sync::Arc};

pub async fn write_csv(path: String, rate: Arc<Mutex<Vec<(u128, u128)>>>) {
    let file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path)
        .unwrap();
    let mut wtr = Writer::from_writer(file);
    wtr.write_record(["time(s)", "packets"]).unwrap();
    rate.lock().await.iter().for_each(|v| {
        wtr.write_record([
            v.0.to_string().as_str(),
            v.1.to_string().as_str()
        ])
        .unwrap();
    });
    wtr.flush().unwrap();
}
