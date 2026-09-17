// Copyright 2026 SECO Mind Srl
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

use std::ffi::c_char;

use ffi_convert::{AsRust, CDrop, CReprOf};

/// Configuration for MQTT-based connection to Astarte
pub struct MqttConnectionConfig {
    /// Astarte device ID
    pub device_id: String,
    /// Credential secret for pairing/authentication
    pub cred_secr: String,
    /// Astarte realm name
    pub realm: String,
    /// Astarte pairing API URL
    pub pairing_url: String,
}

/// Configuration for gRPC-based connection to Astarte Message Hub
pub struct GrpcConnectionConfig {
    /// Address of the Astarte Message Hub gRPC service
    pub message_hub_addr: String,
}

/// Connection configuration variant (MQTT or gRPC)
pub enum ConnectionConfig {
    /// MQTT connection configuration
    Mqtt(MqttConnectionConfig),
    /// gRPC connection configuration
    Grpc(GrpcConnectionConfig),
}

/// Generic device configuration parameters
pub struct GenericDeviceConfig {
    /// Path to directory containing Astarte interface JSON files
    pub interfaces_dir: String,
    /// Channel size for internal message queues
    pub channel_size: usize,
    /// Path to directory for storing persistent data
    pub writable_dir: String,
}

/// Complete device configuration combining connection and generic settings
pub struct DeviceConfig {
    /// Connection-specific configuration
    pub connection: ConnectionConfig,
    /// Generic device configuration
    pub generic: GenericDeviceConfig,
    // pub max_volatile_retention: usize,
    // pub connection_timeout_millis: u64,
    // pub send_timeout_millis: u64,
    // pub slow_receive_threshold: u64,
}

/// C-compatible MQTT connection configuration struct
#[repr(C)]
#[derive(CDrop, CReprOf, AsRust)]
#[target_type(MqttConnectionConfig)]
pub struct NativeMqttConnectionConfig {
    /// C string pointer to device ID
    pub device_id: *const c_char,
    /// C string pointer to credential secret
    pub cred_secr: *const c_char,
    /// C string pointer to realm name
    pub realm: *const c_char,
    /// C string pointer to pairing URL
    pub pairing_url: *const c_char,
}

/// C-compatible gRPC connection configuration struct
#[repr(C)]
#[derive(CDrop, CReprOf, AsRust)]
#[target_type(GrpcConnectionConfig)]
pub struct NativeGrpcConnectionConfig {
    /// C string pointer to Message Hub address
    pub message_hub_addr: *const c_char,
}

/// C-compatible connection configuration enum
#[repr(C)]
pub enum NativeConnectionConfig {
    /// MQTT connection variant
    Mqtt(NativeMqttConnectionConfig),
    /// gRPC connection variant
    Grpc(NativeGrpcConnectionConfig),
}

impl CDrop for NativeConnectionConfig {
    fn do_drop(&mut self) -> Result<(), ffi_convert::CDropError> {
        // nothing to do
        Ok(())
    }
}

impl CReprOf<ConnectionConfig> for NativeConnectionConfig {
    fn c_repr_of(input: ConnectionConfig) -> Result<Self, ffi_convert::CReprOfError> {
        let this = match input {
            ConnectionConfig::Mqtt(mqtt_connection_config) => Self::Mqtt(
                NativeMqttConnectionConfig::c_repr_of(mqtt_connection_config)?,
            ),
            ConnectionConfig::Grpc(grpc_connection_config) => Self::Grpc(
                NativeGrpcConnectionConfig::c_repr_of(grpc_connection_config)?,
            ),
        };

        Ok(this)
    }
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

/// C-compatible generic device configuration struct
#[repr(C)]
#[derive(CDrop, CReprOf, AsRust)]
#[target_type(GenericDeviceConfig)]
pub struct NativeGenericDeviceConfig {
    /// C string pointer to interfaces directory path
    pub interfaces_dir: *const c_char,
    /// Channel size for internal message queues
    pub channel_size: usize,
    /// C string pointer to writable directory path
    pub writable_dir: *const c_char,
}

/// C-compatible complete device configuration struct
#[repr(C)]
#[derive(CDrop, CReprOf, AsRust)]
#[target_type(DeviceConfig)]
pub struct NativeDeviceConfig {
    /// Native connection configuration
    pub connection: NativeConnectionConfig,
    /// Native generic device configuration
    pub generic: NativeGenericDeviceConfig,
    // pub max_volatile_retention: usize,
    // pub connection_timeout_millis: u64,
    // pub send_timeout_millis: u64,
    // pub slow_receive_threshold: u64,
}
