use std::ffi::c_char;

use ffi_convert::AsRust;

pub struct MqttConnectionConfig {
    pub device_id: String,
    pub cred_secr: String,
    pub realm: String,
    pub pairing_url: String,
}

pub struct GrpcConnectionConfig {
    pub message_hub_addr: String,
}

pub enum ConnectionConfig {
    Mqtt(MqttConnectionConfig),
    Grpc(GrpcConnectionConfig),
}

pub struct GenericDeviceConfig {
    pub interfaces_dir: String,
    pub channel_size: usize,
    pub writable_dir: String,
}

pub struct DeviceConfig {
    pub connection: ConnectionConfig,
    pub generic: GenericDeviceConfig,
    // pub max_volatile_retention: usize,
    // pub connection_timeout_millis: u64,
    // pub send_timeout_millis: u64,
    // pub slow_receive_threshold: u64,
}

#[repr(C)]
#[derive(AsRust)]
#[target_type(MqttConnectionConfig)]
pub struct NativeMqttConnectionConfig {
    pub device_id: *const c_char,
    pub cred_secr: *const c_char,
    pub realm: *const c_char,
    pub pairing_url: *const c_char,
}

#[repr(C)]
#[derive(AsRust)]
#[target_type(GrpcConnectionConfig)]
pub struct NativeGrpcConnectionConfig {
    pub message_hub_addr: *const c_char,
}

#[repr(C)]
pub enum NativeConnectionConfig {
    Mqtt(NativeMqttConnectionConfig),
    Grpc(NativeGrpcConnectionConfig),
}

impl AsRust<ConnectionConfig> for NativeConnectionConfig {
    fn as_rust(&self) -> Result<ConnectionConfig, ffi_convert::AsRustError> {
        let nat = match self {
            NativeConnectionConfig::Mqtt(native_mqtt_connection_config) => {
                ConnectionConfig::Mqtt(native_mqtt_connection_config.as_rust()?)
            }
            NativeConnectionConfig::Grpc(native_grpc_connection_config) => {
                ConnectionConfig::Grpc(native_grpc_connection_config.as_rust()?)
            }
        };

        Ok(nat)
    }
}

#[repr(C)]
#[derive(AsRust)]
#[target_type(GenericDeviceConfig)]
pub struct NativeGenericDeviceConfig {
    pub interfaces_dir: *const c_char,
    pub channel_size: usize,
    pub writable_dir: *const c_char,
}

#[repr(C)]
#[derive(AsRust)]
#[target_type(DeviceConfig)]
pub struct NativeDeviceConfig {
    pub connection: NativeConnectionConfig,
    pub generic: NativeGenericDeviceConfig,
    // pub max_volatile_retention: usize,
    // pub connection_timeout_millis: u64,
    // pub send_timeout_millis: u64,
    // pub slow_receive_threshold: u64,
}
