use astarte_device_sdk::chrono::Utc;
use astarte_device_sdk::types::Double;
use astarte_device_sdk::{AstarteData, connection, error};

use astarte_device_sdk::{
    builder::DeviceBuilder,
    client::DeviceClient,
    prelude::*,
    store::memory::MemoryStore,
    transport::mqtt::{Mqtt, MqttConfig},
};

use boltffi::{data, export};
use tokio::runtime::Runtime;
// use uniffi::deps::anyhow::Ok as UF_OK;

// uniffi::setup_scaffolding!();

// ==========================================
// 1. Data Types (Simplified for FFI)
// ==========================================

/// Represents the data values Astarte accepts.
/// We map this to the SDK's `AstarteType`.
#[data]
pub enum AstarteVal {
    // Double { value: f64 },
    Integer { value: i32 },
    // Boolean { value: bool },
    // LongInteger { value: i64 },
    // String { value: String },
}

/// Simplified configuration record.
#[data]
pub struct AstarteConfig {
    pub realm: String,
    pub device_id: String,
    pub credentials_secret: String,
    pub pairing_url: String,
    pub ignore_ssl: bool,
}

#[boltffi::error]
pub enum SdkError {
    // #[error("Configuration error: {msg}")]
    Config { msg: String },
    // #[error("Connection failed: {msg}")]
    Connection { msg: String },
    // #[error("Send failed: {msg}")]
    Send { msg: String },
}

// ==========================================
// 2. Callback Interface
// ==========================================

/// Foreign languages implement this trait to receive events.
#[export]
pub trait EventListener: Send + Sync {
    fn on_connected(&self);
    fn on_disconnected(&self);
    fn on_data_received(&self, interface: String, path: String, data: AstarteVal);
}

// ==========================================
// 3. The Main Wrapper Object
// ==========================================

#[boltffi::data]
pub struct AstarteDevice {
    // inner: DeviceClient<Mqtt<MemoryStore>>,
    // rt: Runtime,
}

// #[export]
// impl AstarteDevice {
//     /// Constructor: Builds the SDK instance
//     #[uniffi::constructor]
//     pub fn new(
//         config: AstarteConfig,
//         interfaces_dir: String,
//     ) -> Result<std::sync::Arc<Self>, SdkError> {
//         use env_logger::Env;

//         env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();

//         let rt = Runtime::new().map_err(|e| SdkError::Config { msg: e.to_string() })?;

//         // Execute the build process inside the runtime
//         let (device, connection) = rt
//             .block_on(async {
//                 let mut opts = MqttConfig::with_credential_secret(
//                     &config.realm,
//                     &config.device_id,
//                     &config.credentials_secret,
//                     &config.pairing_url,
//                 );

//                 if config.ignore_ssl {
//                     opts.ignore_ssl_errors();
//                 }

//                 let device = DeviceBuilder::new()
//                     .store(MemoryStore::new())
//                     .interface_directory(interfaces_dir)?
//                     .connection(opts)
//                     .build()
//                     .await
//                     .map_err(|e| SdkError::Connection { msg: e.to_string() })?;

//                 UF_OK(device)
//             })
//             .unwrap();

//         rt.spawn(async move {
//             let _ = connection.handle_events().await.unwrap();
//             println!("disconnected");
//         });

//         UF_OK(std::sync::Arc::new(Self { inner: device, rt }))
//             .map_err(|e| SdkError::Connection { msg: e.to_string() })
//     }

//     // Sends data to Astarte.
//     // This bridges the blocking FFI call to the async Rust SDK.
//     pub fn send(
//         &self,
//         interface_name: String,
//         interface_path: String,
//         data: AstarteVal,
//     ) -> Result<(), SdkError> {
//         // Convert our simplified Enum to the SDK's AstarteType
//         let sdk_data = match data {
//             // AstarteVal::Double { value } => AstarteData::Double(astarte_device_sdk::types::Double(value)),
//             AstarteVal::Integer { value } => AstarteData::Integer(value),
//             // AstarteVal::Boolean { value } => AstarteData::BinaryBlob::Boolean(value),
//             // AstarteVal::LongInteger { value } => AstarteData::BinaryBlob::LongInteger(value),
//             // AstarteVal::String { value } => AstarteData::BinaryBlob::String(value),
//         };

//         let mut device = self.inner.clone();

//         Ok(self.rt.block_on(async move {
//             device
//                 .send_individual_with_timestamp(&interface_name, &interface_path, sdk_data,  Utc::now())
//                 .await
//                 .map_err(|e| SdkError::Send { msg: e.to_string() })
//                 .unwrap();
//         }))
//     }

//     // /// Starts the event loop in a background task.
//     // /// This calls the `listener` methods when events occur.
//     pub fn start_listening(&self, listener: Box<dyn EventListener>) {
//         let device = self.inner.clone();

//         let listener = std::sync::Arc::new(listener); // Arc it to share across threads

//         self.rt.block_on(async move {
//             loop {
//                 match device.recv().await {
//                     Ok(event) => {
//                         match event.data {
//                             astarte_device_sdk::Value::Individual { data, timestamp } => {
//                                 println!("receive Individual");
//                             }
//                             astarte_device_sdk::Value::Object { data, timestamp } => {
//                                 println!("receive Object");
//                             }
//                             astarte_device_sdk::Value::Property(_) => {
//                                 println!("receive prop");
//                             }
//                         };
//                     }
//                     Err(e) => {
//                         println!("receive error: {:?}", e);
//                         listener.on_disconnected();
//                     }
//                 }
//             }
//         });
//     }
// }
