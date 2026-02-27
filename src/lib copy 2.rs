use astarte_device_sdk::chrono::Utc;
use astarte_device_sdk::types::Double;
use astarte_device_sdk::{connection, error, AstarteData};

use astarte_device_sdk::{
    builder::DeviceBuilder,
    client::DeviceClient,
    prelude::*,
    store::memory::MemoryStore,
    transport::mqtt::{Mqtt, MqttConfig},
};

use tokio::runtime::Runtime;

/// Simplified configuration record.
#[repr(C)]
pub struct AstarteConfig {
    pub realm: String,
    pub device_id: String,
    pub credentials_secret: String,
    pub pairing_url: String,
    pub ignore_ssl: bool,
}

#[repr(C)]
pub struct AstarteDevice {
    inner: DeviceClient<Mqtt<MemoryStore>>,
    rt: Runtime,
}

impl AstarteDevice {
    #[no_mangle]
    pub extern "C" fn new(config: AstarteConfig, interfaces_dir: String) -> Self {
        use env_logger::Env;

        env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();

        let rt = Runtime::new().unwrap();

        // Execute the build process inside the runtime
        let (device, connection) = rt.block_on(async {
            let mut opts = MqttConfig::with_credential_secret(
                &config.realm,
                &config.device_id,
                &config.credentials_secret,
                &config.pairing_url,
            );

            if config.ignore_ssl {
                opts.ignore_ssl_errors();
            }

            let device = DeviceBuilder::new()
                .store(MemoryStore::new())
                .interface_directory(interfaces_dir)
                .unwrap()
                .connection(opts)
                .build()
                .await
                .unwrap();

            device
        });

        rt.spawn(async move {
            let _ = connection.handle_events().await.unwrap();
            println!("disconnected");
        });

        Self { inner: device, rt }
    }

    // Sends data to Astarte.
    // This bridges the blocking FFI call to the async Rust SDK.
    #[no_mangle]
    pub extern "C" fn send(
        self,
        interface_name: String,
        interface_path: String,
        // data: AstarteVal,
        value: i32,
    ) {
        let mut device = self.inner.clone();

        self.rt.block_on(async move {
            let _ = device
                .send_individual_with_timestamp(
                    &interface_name,
                    &interface_path,
                    AstarteData::Integer(value),
                    Utc::now(),
                )
                .await
                .unwrap();
            // .map_err(|e| SdkError::Send { msg: e.to_string() }).unwrap()
        });
    }

    //     // /// Starts the event loop in a background task.
    //     // /// This calls the `listener` methods when events occur.
    #[no_mangle]
    pub extern "C" fn start_listening(self) {
        let device = self.inner.clone();

        self.rt.block_on(async move {
            loop {
                match device.recv().await {
                    Ok(event) => {
                        match event.data {
                            astarte_device_sdk::Value::Individual { data, timestamp } => {
                                println!("receive Individual");
                            }
                            astarte_device_sdk::Value::Object { data, timestamp } => {
                                println!("receive Object");
                            }
                            astarte_device_sdk::Value::Property(_) => {
                                println!("receive prop");
                            }
                        };
                    }
                    Err(e) => {
                        println!("receive error: {:?}", e);
                    }
                }
            }
        });
    }
}
