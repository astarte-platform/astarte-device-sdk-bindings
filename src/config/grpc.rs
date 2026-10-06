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

//! Configuration for gRPC-based connection to Astarte Message Hub

use std::ffi::c_char;

use ffi_convert::{AsRust, CDrop, CReprOf};

/// Configuration for gRPC-based connection to Astarte Message Hub
pub struct GrpcConnectionConfig {
    /// Address of the Astarte Message Hub gRPC service
    pub message_hub_addr: String,
}

/// C-compatible gRPC connection configuration struct
#[repr(C)]
#[derive(CDrop, CReprOf, AsRust)]
#[target_type(GrpcConnectionConfig)]
pub struct NativeGrpcConnectionConfig {
    /// C string pointer to Message Hub address
    pub message_hub_addr: *const c_char,
}
