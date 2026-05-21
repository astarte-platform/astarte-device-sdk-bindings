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
    Integer          { value: i32 },
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
            AstarteData::Integer(v)          => AstarteVal::Integer     { value: v },
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
            AstarteVal::Integer     { value } => AstarteData::Integer(value),
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

/// One field of an Object-aggregation event (key + converted value).
#[data]
pub struct ObjectEntry {
    pub key: String,
    pub value: AstarteVal,
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
    /// Called for Individual datastream events.
    fn on_data_received(&self, interface: String, path: String, data: AstarteVal);
    /// Called for Object (aggregate) datastream events. Unsupported fields are silently skipped.
    fn on_object_received(&self, interface: String, path: String, entries: Vec<ObjectEntry>);
    /// Called when a property is set.
    fn on_property_received(&self, interface: String, path: String, data: AstarteVal);
    /// Called when a property is unset.
    fn on_property_unset(&self, interface: String, path: String);
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
                        astarte_device_sdk::Value::Object { data, .. } => {
                            let entries: Vec<ObjectEntry> = data
                                .into_key_values()
                                .filter_map(|(key, val)| {
                                    AstarteVal::try_from(val)
                                        .ok()
                                        .map(|value| ObjectEntry { key, value })
                                })
                                .collect();
                            listener.on_object_received(
                                event.interface.to_string(),
                                event.path.to_string(),
                                entries,
                            );
                        }
                        astarte_device_sdk::Value::Property(Some(data)) => {
                            if let Ok(val) = AstarteVal::try_from(data) {
                                listener.on_property_received(
                                    event.interface.to_string(),
                                    event.path.to_string(),
                                    val,
                                );
                            }
                        }
                        astarte_device_sdk::Value::Property(None) => {
                            listener.on_property_unset(
                                event.interface.to_string(),
                                event.path.to_string(),
                            );
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

#[cfg(test)]
mod tests {
    use super::*;
    use astarte_device_sdk::chrono::TimeZone;

    fn ts_millis(ms: i64) -> astarte_device_sdk::chrono::DateTime<astarte_device_sdk::chrono::Utc> {
        Utc.timestamp_millis_opt(ms).unwrap()
    }

    // ── AstarteData → AstarteVal ─────────────────────────────────────────────

    #[test]
    fn data_to_val_double() {
        let val = AstarteVal::try_from(AstarteData::Double(1.5_f64.try_into().unwrap())).unwrap();
        assert!(matches!(val, AstarteVal::Double { value } if value == 1.5));
    }

    #[test]
    fn data_to_val_integer() {
        let val = AstarteVal::try_from(AstarteData::Integer(42)).unwrap();
        assert!(matches!(val, AstarteVal::Integer { value } if value == 42));
    }

    #[test]
    fn data_to_val_boolean() {
        let val = AstarteVal::try_from(AstarteData::Boolean(true)).unwrap();
        assert!(matches!(val, AstarteVal::Boolean { value } if value));
    }

    #[test]
    fn data_to_val_long_integer() {
        let val = AstarteVal::try_from(AstarteData::LongInteger(i64::MAX)).unwrap();
        assert!(matches!(val, AstarteVal::LongInteger { value } if value == i64::MAX));
    }

    #[test]
    fn data_to_val_string() {
        let val = AstarteVal::try_from(AstarteData::String("hello".into())).unwrap();
        assert!(matches!(val, AstarteVal::IString { value } if value == "hello"));
    }

    #[test]
    fn data_to_val_binary_blob() {
        let val = AstarteVal::try_from(AstarteData::BinaryBlob(vec![1, 2, 3])).unwrap();
        assert!(matches!(val, AstarteVal::BinaryBlob { value } if value == [1, 2, 3]));
    }

    #[test]
    fn data_to_val_datetime() {
        let ms = 1_700_000_000_000_i64;
        let val = AstarteVal::try_from(AstarteData::DateTime(ts_millis(ms))).unwrap();
        assert!(matches!(val, AstarteVal::DateTime { value } if value == ms));
    }

    #[test]
    fn data_to_val_double_array() {
        let doubles: Vec<SdkDouble> = vec![1.0_f64.try_into().unwrap(), 2.0_f64.try_into().unwrap()];
        let val = AstarteVal::try_from(AstarteData::DoubleArray(doubles)).unwrap();
        assert!(matches!(val, AstarteVal::DoubleArray { ref value } if *value == [1.0, 2.0]));
    }

    #[test]
    fn data_to_val_integer_array() {
        let val = AstarteVal::try_from(AstarteData::IntegerArray(vec![1, 2, 3])).unwrap();
        assert!(matches!(val, AstarteVal::IntegerArray { ref value } if *value == [1, 2, 3]));
    }

    #[test]
    fn data_to_val_boolean_array() {
        let val = AstarteVal::try_from(AstarteData::BooleanArray(vec![true, false])).unwrap();
        assert!(matches!(val, AstarteVal::BooleanArray { ref value } if *value == [true, false]));
    }

    #[test]
    fn data_to_val_long_integer_array() {
        let val = AstarteVal::try_from(AstarteData::LongIntegerArray(vec![10, 20])).unwrap();
        assert!(matches!(val, AstarteVal::LongIntegerArray { ref value } if *value == [10, 20]));
    }

    #[test]
    fn data_to_val_string_array() {
        let val = AstarteVal::try_from(AstarteData::StringArray(vec!["a".into(), "b".into()])).unwrap();
        assert!(matches!(val, AstarteVal::StringArray { ref value } if *value == ["a", "b"]));
    }

    #[test]
    fn data_to_val_datetime_array() {
        let ms = vec![1_000_000_i64, 2_000_000_i64];
        let dates = ms.iter().map(|&m| ts_millis(m)).collect();
        let val = AstarteVal::try_from(AstarteData::DateTimeArray(dates)).unwrap();
        assert!(matches!(val, AstarteVal::DateTimeArray { ref value } if *value == ms));
    }

    #[test]
    fn data_to_val_binary_blob_array_is_err() {
        let result = AstarteVal::try_from(AstarteData::BinaryBlobArray(vec![]));
        assert!(result.is_err());
    }

    // ── AstarteVal → AstarteData ─────────────────────────────────────────────

    #[test]
    fn val_to_data_double() {
        let data = AstarteData::try_from(AstarteVal::Double { value: 3.14 }).unwrap();
        assert!(matches!(data, AstarteData::Double(_)));
    }

    #[test]
    fn val_to_data_double_nan_is_err() {
        let result = AstarteData::try_from(AstarteVal::Double { value: f64::NAN });
        assert!(result.is_err());
    }

    #[test]
    fn val_to_data_double_inf_is_err() {
        let result = AstarteData::try_from(AstarteVal::Double { value: f64::INFINITY });
        assert!(result.is_err());
    }

    #[test]
    fn val_to_data_integer() {
        let data = AstarteData::try_from(AstarteVal::Integer { value: -7 }).unwrap();
        assert!(matches!(data, AstarteData::Integer(-7)));
    }

    #[test]
    fn val_to_data_boolean() {
        let data = AstarteData::try_from(AstarteVal::Boolean { value: false }).unwrap();
        assert!(matches!(data, AstarteData::Boolean(false)));
    }

    #[test]
    fn val_to_data_long_integer() {
        let data = AstarteData::try_from(AstarteVal::LongInteger { value: i64::MIN }).unwrap();
        assert!(matches!(data, AstarteData::LongInteger(v) if v == i64::MIN));
    }

    #[test]
    fn val_to_data_string() {
        let data = AstarteData::try_from(AstarteVal::IString { value: "world".into() }).unwrap();
        assert!(matches!(data, AstarteData::String(ref s) if s == "world"));
    }

    #[test]
    fn val_to_data_binary_blob() {
        let data = AstarteData::try_from(AstarteVal::BinaryBlob { value: vec![0xFF] }).unwrap();
        assert!(matches!(data, AstarteData::BinaryBlob(ref v) if *v == [0xFF]));
    }

    #[test]
    fn val_to_data_datetime_roundtrip() {
        let ms = 1_700_000_000_000_i64;
        let data = AstarteData::try_from(AstarteVal::DateTime { value: ms }).unwrap();
        if let AstarteData::DateTime(dt) = data {
            assert_eq!(dt.timestamp_millis(), ms);
        } else {
            panic!("expected DateTime variant");
        }
    }

    #[test]
    fn val_to_data_datetime_invalid_ts_is_err() {
        // i64::MAX milliseconds is outside the range chrono can represent
        let result = AstarteData::try_from(AstarteVal::DateTime { value: i64::MAX });
        assert!(result.is_err());
    }

    #[test]
    fn val_to_data_double_array_with_nan_is_err() {
        let result = AstarteData::try_from(AstarteVal::DoubleArray { value: vec![1.0, f64::NAN] });
        assert!(result.is_err());
    }

    #[test]
    fn val_to_data_integer_array() {
        let data = AstarteData::try_from(AstarteVal::IntegerArray { value: vec![1, 2] }).unwrap();
        assert!(matches!(data, AstarteData::IntegerArray(ref v) if *v == [1, 2]));
    }

    #[test]
    fn val_to_data_boolean_array() {
        let data = AstarteData::try_from(AstarteVal::BooleanArray { value: vec![false, true] }).unwrap();
        assert!(matches!(data, AstarteData::BooleanArray(ref v) if *v == [false, true]));
    }

    #[test]
    fn val_to_data_long_integer_array() {
        let data = AstarteData::try_from(AstarteVal::LongIntegerArray { value: vec![100, 200] }).unwrap();
        assert!(matches!(data, AstarteData::LongIntegerArray(ref v) if *v == [100, 200]));
    }

    #[test]
    fn val_to_data_string_array() {
        let data = AstarteData::try_from(AstarteVal::StringArray { value: vec!["x".into()] }).unwrap();
        assert!(matches!(data, AstarteData::StringArray(ref v) if *v == ["x"]));
    }

    #[test]
    fn val_to_data_datetime_array_roundtrip() {
        let ms = vec![0_i64, 1_000_i64];
        let data = AstarteData::try_from(AstarteVal::DateTimeArray { value: ms.clone() }).unwrap();
        if let AstarteData::DateTimeArray(dates) = data {
            let got: Vec<i64> = dates.iter().map(|d| d.timestamp_millis()).collect();
            assert_eq!(got, ms);
        } else {
            panic!("expected DateTimeArray variant");
        }
    }

    #[test]
    fn val_to_data_datetime_array_invalid_is_err() {
        let result = AstarteData::try_from(AstarteVal::DateTimeArray { value: vec![i64::MAX] });
        assert!(result.is_err());
    }

    // ── ObjectEntry / AstarteObject → Vec<ObjectEntry> ──────────────────────

    fn obj_to_entries(obj: astarte_device_sdk::aggregate::AstarteObject) -> Vec<ObjectEntry> {
        obj.into_key_values()
            .filter_map(|(key, val)| {
                AstarteVal::try_from(val).ok().map(|value| ObjectEntry { key, value })
            })
            .collect()
    }

    #[test]
    fn object_entry_holds_key_and_value() {
        let entry = ObjectEntry {
            key: "temperature".into(),
            value: AstarteVal::Double { value: 23.5 },
        };
        assert_eq!(entry.key, "temperature");
        assert!(matches!(entry.value, AstarteVal::Double { value } if value == 23.5));
    }

    #[test]
    fn object_to_entries_empty() {
        let entries = obj_to_entries(astarte_device_sdk::aggregate::AstarteObject::new());
        assert!(entries.is_empty());
    }

    #[test]
    fn object_to_entries_converts_all_supported_fields() {
        let mut obj = astarte_device_sdk::aggregate::AstarteObject::new();
        obj.insert("count".into(), AstarteData::Integer(7));
        obj.insert("label".into(), AstarteData::String("hi".into()));
        obj.insert("flag".into(), AstarteData::Boolean(true));

        let entries = obj_to_entries(obj);

        assert_eq!(entries.len(), 3);
        fn find<'a>(k: &str, entries: &'a [ObjectEntry]) -> &'a ObjectEntry {
            entries.iter().find(|e| e.key == k).unwrap()
        }
        assert!(matches!(find("count", &entries).value, AstarteVal::Integer { value } if value == 7));
        assert!(matches!(&find("label", &entries).value, AstarteVal::IString { value } if value == "hi"));
        assert!(matches!(find("flag", &entries).value, AstarteVal::Boolean { value } if value));
    }

    #[test]
    fn object_to_entries_skips_binary_blob_array() {
        let mut obj = astarte_device_sdk::aggregate::AstarteObject::new();
        obj.insert("ok".into(), AstarteData::Integer(1));
        obj.insert("bad".into(), AstarteData::BinaryBlobArray(vec![vec![0x01]]));

        let entries = obj_to_entries(obj);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].key, "ok");
    }

    #[test]
    fn object_to_entries_preserves_insertion_order() {
        let mut obj = astarte_device_sdk::aggregate::AstarteObject::new();
        obj.insert("first".into(), AstarteData::Integer(1));
        obj.insert("second".into(), AstarteData::Integer(2));
        obj.insert("third".into(), AstarteData::Integer(3));

        let entries = obj_to_entries(obj);

        let keys: Vec<&str> = entries.iter().map(|e| e.key.as_str()).collect();
        assert_eq!(keys, ["first", "second", "third"]);
    }

    #[test]
    fn object_to_entries_all_astarte_val_variants() {
        let ms = 1_700_000_000_000_i64;
        let mut obj = astarte_device_sdk::aggregate::AstarteObject::new();
        obj.insert("d".into(),   AstarteData::Double(1.0_f64.try_into().unwrap()));
        obj.insert("i".into(),   AstarteData::Integer(2));
        obj.insert("b".into(),   AstarteData::Boolean(false));
        obj.insert("li".into(),  AstarteData::LongInteger(3));
        obj.insert("s".into(),   AstarteData::String("x".into()));
        obj.insert("bb".into(),  AstarteData::BinaryBlob(vec![0xAB]));
        obj.insert("dt".into(),  AstarteData::DateTime(ts_millis(ms)));

        let entries = obj_to_entries(obj);

        assert_eq!(entries.len(), 7);
        fn find<'a>(k: &str, v: &'a [ObjectEntry]) -> &'a ObjectEntry {
            v.iter().find(|e| e.key == k).unwrap()
        }
        assert!(matches!(find("d",  &entries).value, AstarteVal::Double       { .. }));
        assert!(matches!(find("i",  &entries).value, AstarteVal::Integer     { value } if value == 2));
        assert!(matches!(find("b",  &entries).value, AstarteVal::Boolean      { value } if !value));
        assert!(matches!(find("li", &entries).value, AstarteVal::LongInteger  { value } if value == 3));
        assert!(matches!(&find("s",  &entries).value, AstarteVal::IString     { value } if value == "x"));
        assert!(matches!(&find("bb", &entries).value, AstarteVal::BinaryBlob  { value } if *value == [0xAB]));
        assert!(matches!(find("dt", &entries).value, AstarteVal::DateTime     { value } if value == ms));
    }
}
