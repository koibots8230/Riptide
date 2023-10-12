use std::env;
use std::net::{Ipv4Addr, SocketAddrV4};
use network_tables::v4::{SubscriptionOptions, Type};
use std::string;
use std::sync::{ Arc, Mutex };

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arg: Vec<string::String> = env::args().collect();
    let ip: Ipv4Addr;

    if arg[1] == "--sim".to_owned() {
        ip = Ipv4Addr::new(127, 0, 0, 1)
    } else {
        ip = Ipv4Addr::new(10, 82, 30, 2)
    }

    let client = network_tables::v4::Client::try_new_w_config(
        SocketAddrV4::new(ip, 5810),
        network_tables::v4::client_config::Config::default()
    ).await?;

    let trajectory_publisher = client
        .publish_topic("/Riptide/Trajectory", Type::String, None)
        .await?;

    let mut waypoint_subscriber = client
        .subscribe_w_options(
            &["/Riptide/Waypoints"],
            Some(SubscriptionOptions::default()),
        )
        .await?;

    let mut trajectory_data: Arc<Mutex<[f64; 6]>> = Arc::new(Mutex::new([0f64; 6]));
    let mut trajectory_changed: bool = false;

    let task_client = client.clone();
    tokio::spawn(async move {
        loop {
            if trajectory_changed {
                let data: [f64; 6] = trajectory_data.lock().unwrap().clone();

                task_client
                    .publish_value(
                        &trajectory_publisher,
                        &network_tables::Value::from(
                            data[2]
                        ))
                    .await
                    .unwrap();

                trajectory_changed = false;
            }
            tokio::time::sleep(std::time::Duration::from_millis(2)).await;
        }
    });


    while let Some(message) = waypoint_subscriber.next().await {
        // Trajectory stuff goes here

        trajectory_changed = true
    }

    Ok(())
}
