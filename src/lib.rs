use std::convert::TryFrom;

use astarte_device_sdk::chrono::{TimeZone, Utc};
use astarte_device_sdk::types::Double as SdkDouble;
use astarte_device_sdk::AstarteData;

use astarte_device_sdk::{
    builder::DeviceBuilder,
    client::DeviceClient,
    prelude::*,
    store::memory::MemoryStore,
    transport::mqtt::{Mqtt, MqttConfig},
};

use boltffi::{data, export};
use tokio::runtime::Runtime;

#[data]
pub enum AstarteVal {
    Double           { value: f64 },
    IInteger         { value: i32 },
    Boolean          { value: bool },
    LongInteger      { value: i64 },
    IString          { value: String },
    BinaryBlob       { value: Vec<u8> },
    DateTime         { value: i64 },         // milliseconds since Unix epoch
    DoubleArray      { value: Vec<f64> },
    IntegerArray     { value: Vec<i32> },
    BooleanArray     { value: Vec<bool> },
    LongIntegerArray { value: Vec<i64> },
    StringArray      { value: Vec<String> },
    DateTimeArray    { value: Vec<i64> },    // milliseconds since Unix epoch
}

impl TryFrom<AstarteData> for AstarteVal {
    type Error = String;

    fn try_from(data: AstarteData) -> Result<Self, Self::Error> {
        Ok(match data {
            AstarteData::Double(v)           => AstarteVal::Double      { value: v.into() },
            AstarteData::Integer(v)          => AstarteVal::IInteger     { value: v },
            AstarteData::Boolean(v)          => AstarteVal::Boolean      { value: v },
            AstarteData::LongInteger(v)      => AstarteVal::LongInteger  { value: v },
            AstarteData::String(v)           => AstarteVal::IString      { value: v },
            AstarteData::BinaryBlob(v)       => AstarteVal::BinaryBlob   { value: v },
            AstarteData::DateTime(v)         => AstarteVal::DateTime     { value: v.timestamp_millis() },
            AstarteData::DoubleArray(v)      => AstarteVal::DoubleArray  { value: v.into_iter().map(f64::from).collect() },
            AstarteData::IntegerArray(v)     => AstarteVal::IntegerArray     { value: v },
            AstarteData::BooleanArray(v)     => AstarteVal::BooleanArray     { value: v },
            AstarteData::LongIntegerArray(v) => AstarteVal::LongIntegerArray { value: v },
            AstarteData::StringArray(v)      => AstarteVal::StringArray      { value: v },
            AstarteData::BinaryBlobArray(_)  => return Err("BinaryBlobArray is not supported".into()),
            AstarteData::DateTimeArray(v)    => AstarteVal::DateTimeArray {
                value: v.into_iter().map(|d| d.timestamp_millis()).collect(),
            },
        })
    }
}

impl TryFrom<AstarteVal> for AstarteData {
    type Error = SdkError;

    fn try_from(val: AstarteVal) -> Result<Self, Self::Error> {
        Ok(match val {
            AstarteVal::Double { value } => AstarteData::Double(
                SdkDouble::try_from(value).map_err(|e| SdkError::Send { msg: e.to_string() })?,
            ),
            AstarteVal::IInteger     { value } => AstarteData::Integer(value),
            AstarteVal::Boolean      { value } => AstarteData::Boolean(value),
            AstarteVal::LongInteger  { value } => AstarteData::LongInteger(value),
            AstarteVal::IString      { value } => AstarteData::String(value),
            AstarteVal::BinaryBlob   { value } => AstarteData::BinaryBlob(value),
            AstarteVal::DateTime { value } => AstarteData::DateTime(
                Utc.timestamp_millis_opt(value)
                    .single()
                    .ok_or_else(|| SdkError::Send { msg: format!("invalid timestamp: {value}") })?,
            ),
            AstarteVal::DoubleArray { value } => AstarteData::DoubleArray(
                value
                    .into_iter()
                    .map(|v| SdkDouble::try_from(v).map_err(|e| SdkError::Send { msg: e.to_string() }))
                    .collect::<Result<_, _>>()?,
            ),
            AstarteVal::IntegerArray     { value } => AstarteData::IntegerArray(value),
            AstarteVal::BooleanArray     { value } => AstarteData::BooleanArray(value),
            AstarteVal::LongIntegerArray { value } => AstarteData::LongIntegerArray(value),
            AstarteVal::StringArray      { value } => AstarteData::StringArray(value),
            AstarteVal::DateTimeArray { value } => AstarteData::DateTimeArray(
                value
                    .into_iter()
                    .map(|v| {
                        Utc.timestamp_millis_opt(v)
                            .single()
                            .ok_or_else(|| SdkError::Send { msg: format!("invalid timestamp: {v}") })
                    })
                    .collect::<Result<_, _>>()?,
            ),
        })
    }
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

#[derive(Debug)]
#[boltffi::error]
pub enum SdkError {
    Config     { msg: String },
    Connection { msg: String },
    Send       { msg: String },
}

/// Foreign languages implement this trait to receive events.
#[export]
pub trait EventListener: Send + Sync {
    fn on_connected(&self);
    fn on_disconnected(&self);
    fn on_data_received(&self, interface: String, path: String, data: AstarteVal);
}

pub struct AstarteDevice {
    inner: DeviceClient<Mqtt<MemoryStore>>,
    rt: Runtime,
}

#[export]
impl AstarteDevice {
    pub fn new(config: AstarteConfig, interfaces_dir: String) -> Result<Self, SdkError> {
        use env_logger::Env;

        env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();

        let rt = Runtime::new().map_err(|e| SdkError::Config { msg: e.to_string() })?;

        let (device, connection) = rt
            .block_on(async {
                let mut opts = MqttConfig::with_credential_secret(
                    &config.realm,
                    &config.device_id,
                    &config.credentials_secret,
                    &config.pairing_url,
                );

                if config.ignore_ssl {
                    opts.ignore_ssl_errors();
                }

                DeviceBuilder::new()
                    .store(MemoryStore::new())
                    .interface_directory(interfaces_dir)
                    .map_err(|e| SdkError::Connection { msg: e.to_string() })?
                    .connection(opts)
                    .build()
                    .await
                    .map_err(|e| SdkError::Connection { msg: e.to_string() })
            })
            ?;

        rt.spawn(async move {
            let _ = connection.handle_events().await;
            println!("disconnected");
        });

        Ok(Self { inner: device, rt })
    }

    pub fn send(
        &self,
        interface_name: String,
        interface_path: String,
        data: AstarteVal,
    ) -> Result<(), SdkError> {
        let sdk_data: AstarteData = data.try_into()?;
        let mut device = self.inner.clone();

        self.rt.block_on(async move {
            device
                .send_individual_with_timestamp(
                    &interface_name,
                    &interface_path,
                    sdk_data,
                    Utc::now(),
                )
                .await
                .map_err(|e| SdkError::Send { msg: e.to_string() })
        })
    }

    pub fn start_listening(&self, listener: Box<dyn EventListener>) {
        let device = self.inner.clone();
        let listener = std::sync::Arc::new(listener);

        self.rt.block_on(async move {
            loop {
                match device.recv().await {
                    Ok(event) => match event.data {
                        astarte_device_sdk::Value::Individual { data, .. } => {
                            if let Ok(val) = data.try_into() {
                                listener.on_data_received(
                                    event.interface.to_string(),
                                    event.path.to_string(),
                                    val,
                                );
                            }
                        }
                        astarte_device_sdk::Value::Object { .. } => {
                            println!("received Object event (not yet supported)");
                        }
                        astarte_device_sdk::Value::Property(_) => {
                            println!("received Property event");
                        }
                    },
                    Err(e) => {
                        println!("receive error: {:?}", e);
                        listener.on_disconnected();
                    }
                }
            }
        });
    }

    pub fn disconenct(&self) -> Result<(), SdkError> {
        let mut device = self.inner.clone();

        Ok(self.rt.block_on(async move {
            device
                .disconnect()
                .await
                .map_err(|e| SdkError::Send { msg: e.to_string() })
                .unwrap();
        }))
    }
}
