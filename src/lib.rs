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

#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/astarte-platform/astarte-device-sdk-rust/refs/heads/master/assets/logos/clea-24.svg"
)]
#![doc(
    html_favicon_url = "https://raw.githubusercontent.com/astarte-platform/astarte-device-sdk-rust/refs/heads/master/assets/logos/clea-24.ico"
)]
#![warn(
    clippy::dbg_macro,
    clippy::todo,
    missing_docs,
    rustdoc::missing_crate_level_docs
)]
#![cfg_attr(astarte_device_sdk_docsrs, feature(doc_cfg))]

use std::{
    ffi::{CStr, CString, c_char, c_void},
    mem::ManuallyDrop,
};

use astarte_device_sdk::AstarteData;
use eyre::Context;
use ffi_convert::{AsRust, CDrop, CReprOf, CReprOfError};
use tracing::{error, level_filters::LevelFilter};

/// Configuration types for Astarte device connections
pub mod config;
/// Data types for Astarte device communication
pub mod data;
/// Device handle and runtime management
pub mod device;

pub use config::NativeDeviceConfig;
pub use data::*;
pub use device::NativeDeviceHandle;

/// User-provided data that will be passed back to callbacks
///
/// This is a thin wrapper around a raw pointer that allows users to pass
/// arbitrary data to FFI callbacks. The user is responsible for managing
/// the lifetime of the data pointed to by this pointer.
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct UserData(*mut c_void);
unsafe impl Send for UserData {}

impl UserData {
    /// Create a new UserData from a raw pointer
    ///
    /// # Safety
    ///
    /// The pointer must remain valid for the lifetime of the UserData
    /// instance and until it's passed back to the user in a callback.
    pub fn new(inner: *mut c_void) -> Self {
        Self(inner)
    }

    /// Retrieve the raw pointer stored in this UserData
    ///
    /// # Safety
    ///
    /// The returned pointer is only valid as long as the original data
    /// it points to remains valid and unchanged.
    pub fn inner(self) -> *mut c_void {
        self.0
    }
}

/// A C-compatible string wrapper that owns and manages its memory
///
/// This struct wraps a `*const c_char` pointer and provides conversion
/// to and from Rust `String` values. It properly drops the underlying
/// C string when no longer needed.
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct StaticString(*const c_char);
impl CDrop for StaticString {
    fn do_drop(&mut self) -> Result<(), ffi_convert::CDropError> {
        use ffi_convert::RawPointerConverter;

        unsafe { std::ffi::CString::drop_raw_pointer(self.0)? };

        Ok(())
    }
}

impl CReprOf<String> for StaticString {
    fn c_repr_of(input: String) -> Result<Self, CReprOfError> {
        let leaked = CString::new(input)?.into_raw();

        Ok(Self(leaked as *const c_char))
    }
}

impl Drop for StaticString {
    fn drop(&mut self) {
        let _ = self.do_drop();
    }
}

impl AsRust<String> for StaticString {
    fn as_rust(&self) -> Result<String, ffi_convert::AsRustError> {
        unsafe { CStr::from_ptr(self.0).as_rust() }
    }
}

/// A C-compatible Option type for FFI boundaries
///
/// This enum represents an optional value in a C-compatible way,
/// allowing Rust `Option<T>` to be safely passed across FFI boundaries.
#[repr(C)]
#[derive(Debug, Clone)]
pub enum NativeOption<T> {
    /// Contains a value
    Some(T),
    /// Contains no value
    None,
}

impl<T> CDrop for NativeOption<T> {
    fn do_drop(&mut self) -> Result<(), ffi_convert::CDropError> {
        Ok(())
    }
}

impl<T, U: CReprOf<T>> CReprOf<Option<T>> for NativeOption<U> {
    fn c_repr_of(input: Option<T>) -> Result<Self, CReprOfError> {
        let nat = match input {
            Some(u) => Self::Some(U::c_repr_of(u)?),
            None => Self::None,
        };

        Ok(nat)
    }
}

impl<T: AsRust<U>, U> AsRust<Option<U>> for NativeOption<T> {
    fn as_rust(&self) -> Result<Option<U>, ffi_convert::AsRustError> {
        let opt = match self {
            NativeOption::Some(t) => Some(t.as_rust()?),
            NativeOption::None => None,
        };

        Ok(opt)
    }
}

/// A wrapper that prevents automatic dropping of a value
///
/// This struct wraps a value in `ManuallyDrop` so it can be safely
/// passed across FFI boundaries without being dropped by Rust.
/// This is used to implement the [`ffi_convert::AsRust`] and
/// [`ffi_convert::CReprOf`] traits.
#[repr(transparent)]
#[derive(Debug)]
pub struct NativeManuallyDrop<T>(ManuallyDrop<T>);

impl<T> NativeManuallyDrop<T> {
    /// Create a new NativeManuallyDrop from a value
    pub fn new(inner: T) -> Self {
        Self(ManuallyDrop::new(inner))
    }
}

impl<T: AsRust<U>, U> AsRust<U> for NativeManuallyDrop<T> {
    fn as_rust(&self) -> Result<U, ffi_convert::AsRustError> {
        self.0.as_rust()
    }
}

impl<T> CDrop for NativeManuallyDrop<T> {
    fn do_drop(&mut self) -> Result<(), ffi_convert::CDropError> {
        // do not drop anything
        Ok(())
    }
}

impl<T, U: CDrop + CReprOf<T>> CReprOf<T> for NativeManuallyDrop<U> {
    fn c_repr_of(input: T) -> Result<Self, CReprOfError> {
        let native = Self::new(U::c_repr_of(input)?);

        Ok(native)
    }
}

/// Result type wrapper for FFI returning `eyre::Result<()>`
///
/// This enum wraps a void-style result type for use across FFI boundaries,
/// allowing Rust error types to be safely passed to C code. It provides
/// success and error variants, with errors stored as StaticString for
/// memory safety.
#[repr(C)]
#[derive(Debug)]
pub enum NativeVoidResult {
    /// Success with no value
    Ok,
    /// Error variant containing the error message
    Err(StaticString),
}

impl AsRust<eyre::Result<()>> for NativeVoidResult {
    fn as_rust(&self) -> Result<eyre::Result<()>, ffi_convert::AsRustError> {
        let inner = match self {
            NativeVoidResult::Ok => Ok(()),
            NativeVoidResult::Err(static_string) => {
                let err = static_string.as_rust()?;
                Err(eyre::eyre!(err))
            }
        };

        Ok(inner)
    }
}

impl NativeVoidResult {
    fn unwrap_c_repr_of(input: eyre::Result<()>) -> Self {
        match input {
            Ok(()) => Self::Ok,
            Err(e) => {
                let report_string = format!("{e:?}");
                match StaticString::c_repr_of(report_string) {
                    Ok(s) => Self::Err(s),
                    Err(e) => {
                        error!("failed to convert error to C string: {e}");
                        // Create a fallback error message
                        Self::Err(StaticString::c_repr_of("unknown error".to_string()).unwrap())
                    }
                }
            }
        }
    }
}

/// Generic result type wrapper for FFI returning `eyre::Result<T>`
///
/// This enum wraps a generic result type for use across FFI boundaries,
/// allowing Rust error types to be safely passed to C code. It provides
/// success and error variants, with the success value stored generically
/// and errors stored as StaticString for memory safety.
#[repr(C)]
#[derive(Debug)]
pub enum NativeResult<T> {
    /// Success variant containing a value of type T
    Ok(T),
    /// Error variant containing the error message
    Err(StaticString),
}

impl<T> CDrop for NativeResult<T> {
    fn do_drop(&mut self) -> Result<(), ffi_convert::CDropError> {
        // fields are automatically dropped by rust drop glue
        Ok(())
    }
}

impl<T, U: CDrop + CReprOf<T>> CReprOf<eyre::Result<T>> for NativeResult<U> {
    fn c_repr_of(input: eyre::Result<T>) -> Result<Self, CReprOfError> {
        let native = match input {
            Ok(o) => Self::Ok(U::c_repr_of(o)?),
            Err(e) => {
                let report_string = format!("{e:?}");
                match StaticString::c_repr_of(report_string) {
                    Ok(s) => Self::Err(s),
                    Err(e) => {
                        error!("failed to convert error to C string: {e}");
                        // Create a fallback error message
                        Self::Err(StaticString::c_repr_of("unknown error".to_string()).unwrap())
                    }
                }
            }
        };

        Ok(native)
    }
}

fn init_tracing() {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    tracing_subscriber::registry()
        // .with(console_subscriber::spawn())
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive("astarte_device_sdk=debug".parse().unwrap())
                .from_env_lossy()
                // .add_directive("tokio=trace".parse().unwrap())
                // .add_directive("runtime=trace".parse().unwrap())
                .add_directive(LevelFilter::INFO.into()),
        )
        .try_init()
        .unwrap();
}

/// Initialize a new device handle
///
/// Creates a new `NativeDeviceHandle` that encapsulates an asynchronous Tokio runtime
/// and manages state for an Astarte device client.
///
/// # Returns
///
/// A new `NativeDeviceHandle` ready for connection initialization.
///
/// # Memory Management
///
/// The returned handle allocates runtime state on the heap and must be freed
/// using [`device_handle_free`] when no longer needed.
#[unsafe(no_mangle)]
pub extern "C" fn device_handle_init() -> NativeDeviceHandle {
    // NOTE if we want the result here we need to free in the caller
    let handle_result = NativeDeviceHandle::new().unwrap();
    // let native_result = NativeStringResult::from_report(handle_result).unwrap();
    // native_result
    handle_result
}

/// Callback type for device build callbacks in FFI
///
/// This callback is invoked after device client construction finishes. It receives
/// a `NativeVoidResult` pointer indicating success or failure, and the user data pointer.
pub type DeviceHandleBuildCallback =
    extern "C" fn(result: *const NativeVoidResult, user_data: UserData);

/// Callback type for device loop exit callbacks in FFI
///
/// This callback is invoked when the device event loop exits, either after an error
/// or graceful disconnection. It receives a `NativeVoidResult` pointer and user data.
pub type DeviceHandleLoopCallback =
    extern "C" fn(result: *const NativeVoidResult, user_data: UserData);

/// Connect an Astarte device using the provided configuration
///
/// Asynchronously initializes and connects the Astarte device client using the
/// options specified in `config` (MQTT or gRPC transport).
///
/// # Parameters
/// - `handle`: A valid `NativeDeviceHandle` created via [`device_handle_init`].
/// - `config`: Pointer to a `NativeDeviceConfig` structure specifying connection details.
/// - `build_cbk`: Callback called once device client initialization finishes.
/// - `build_user_data`: Opaque user data pointer passed to `build_cbk`.
/// - `loop_cbk`: Callback called when the background event loop terminates.
/// - `loop_user_data`: Opaque user data pointer passed to `loop_cbk`.
///
/// # Callbacks
/// - `build_cbk` is invoked after building the device connection client.
/// - `loop_cbk` is invoked when the event loop loop terminates.
/// - If `build_cbk` receives an error result, `loop_cbk` will **not** be called.
///
/// # Safety
/// - `handle` must be valid at least until both callbacks return.
/// - `config` and all strings referenced within it must remain valid until this function returns.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn device_handle_connect(
    handle: NativeDeviceHandle,
    config: *const NativeDeviceConfig,
    build_cbk: DeviceHandleBuildCallback,
    build_user_data: UserData,
    loop_cbk: DeviceHandleLoopCallback,
    loop_user_data: UserData,
) {
    // NOTE this function is called connect but after the callback is called we are still not connected
    // this could result in surprising behaviour in case of non stored datastream send which would get dropped

    // FIXME remove this
    // console_subscriber::init();
    color_eyre::install().unwrap();
    init_tracing();
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .map_err(|_| eyre::eyre!("couldn't install default crypto provider"))
        .unwrap();
    // FIXME remove this end

    let connected = move |result: eyre::Result<()>| {
        let result = result.wrap_err("error while building client");

        let c_res = NativeVoidResult::unwrap_c_repr_of(result);

        build_cbk(&c_res, build_user_data);
    };

    let exited = move |result: eyre::Result<()>| {
        let result = result.wrap_err("error in handle_events");

        let c_res = NativeVoidResult::unwrap_c_repr_of(result);

        loop_cbk(&c_res, loop_user_data);
    };

    // SAFETY: the device_handle pointer has to be valid at least until both the callbacks return
    unsafe {
        handle.connect(config, connected, exited);
    }
}

/// Callback type for device receive callbacks in FFI
///
/// Invoked when a device event is received from Astarte. Receives a pointer to a `NativeResult`
/// containing the event data and the user data pointer.
pub type DeviceHandleReceiveCallback = extern "C" fn(
    result: *mut NativeResult<NativeManuallyDrop<NativeDeviceEvent>>,
    user_data: UserData,
);

/// Receive an incoming event from the device
///
/// Registers a callback to receive incoming events (datastream transmission or property changes)
/// sent from the Astarte platform to this device.
///
/// # Parameters
/// - `device_handle`: A valid connected `NativeDeviceHandle`.
/// - `callback`: Callback function pointer invoked when an event is received.
/// - `user_data`: Opaque user data pointer passed to `callback`.
///
/// # Memory Management
/// The `NativeDeviceEvent` received in `callback` contains allocated memory buffers (e.g. C strings).
/// Call [`device_handle_free_device_event`] on the event pointer after processing to prevent memory leaks.
///
/// # Safety
/// - `device_handle` must be valid at least until `callback` completes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn device_handle_receive(
    device_handle: NativeDeviceHandle,
    callback: DeviceHandleReceiveCallback,
    user_data: UserData,
) {
    let received = move |res| {
        let mut c_res = NativeResult::c_repr_of(res).unwrap();

        callback(&mut c_res, user_data);
    };

    // SAFETY: the device_handle pointer has to be valid at least until the callback returns
    unsafe {
        device_handle.receive(received);
    }
}

/// Free resources associated with a `NativeDeviceEvent` pointer
///
/// Deallocates memory owned by the event (interface strings, paths, data buffers, array entries).
///
/// # Parameters
/// - `event`: Pointer to the `NativeDeviceEvent` to free.
///
/// # Safety
/// - `event` must be non-null, valid, and must not be accessed after calling this function.
// NOTE i'd like to pass a value but currently it is not supported since therer are unnamed structs in unions
#[unsafe(no_mangle)]
pub unsafe extern "C" fn device_handle_free_device_event(event: *mut NativeDeviceEvent) {
    let Some(event) = (unsafe { event.as_mut() }) else {
        error!("the pointer has to be non null and valid");
        return;
    };

    if let Err(e) = event.do_drop() {
        error!("{e:#}");
    }
}

/// Callback type for device send callbacks in FFI
///
/// Invoked after a send or property modification operation finishes.
pub type DeviceHandleSendCallback =
    extern "C" fn(result: *const NativeVoidResult, user_data: UserData);

/// Send an individual datastream value from the device
///
/// Asynchronously sends an individual value (scalar or array with optional timestamp)
/// to an Astarte interface and path.
///
/// # Parameters
/// - `device_handle`: A valid connected `NativeDeviceHandle`.
/// - `data`: Pointer to a `NativeIndividualSend` struct defining interface, path, data, and timestamp.
/// - `callback`: Callback function invoked upon send completion.
/// - `user_data`: Opaque user data pointer passed to `callback`.
///
/// # Safety
/// - `device_handle` must be valid at least until `callback` completes.
/// - `data` and its referenced memory must remain valid until this function returns.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn device_handle_send_individual(
    device_handle: NativeDeviceHandle,
    data: *const NativeIndividualSend,
    callback: DeviceHandleSendCallback,
    user_data: UserData,
) {
    let sent = move |res: eyre::Result<()>| {
        let c_res = NativeVoidResult::unwrap_c_repr_of(res);

        callback(&c_res, user_data);
    };

    // SAFETY: the device_handle pointer has to be valid at least until the callback returns
    unsafe {
        device_handle.send_individual(data, sent);
    }
}

/// Send an object datastream value from the device
///
/// Asynchronously sends an object aggregate (a set of key-value fields with optional timestamp)
/// to an Astarte object interface and path.
///
/// # Parameters
/// - `device_handle`: A valid connected `NativeDeviceHandle`.
/// - `data`: Pointer to a `NativeObjectSend` struct defining interface, path, object entries, and timestamp.
/// - `callback`: Callback function invoked upon send completion.
/// - `user_data`: Opaque user data pointer passed to `callback`.
///
/// # Safety
/// - `device_handle` must be valid at least until `callback` completes.
/// - `data` and its referenced memory must remain valid until this function returns.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn device_handle_send_object(
    device_handle: NativeDeviceHandle,
    data: *const NativeObjectSend,
    callback: DeviceHandleSendCallback,
    user_data: UserData,
) {
    let sent = move |res: eyre::Result<()>| {
        let c_res = NativeVoidResult::unwrap_c_repr_of(res);

        callback(&c_res, user_data);
    };

    // SAFETY: the device_handle pointer has to be valid at least until the callback returns
    unsafe {
        device_handle.send_object(data, sent);
    }
}

/// Set a property value on the device
///
/// Asynchronously sets a property value on an Astarte property interface and path.
///
/// # Parameters
/// - `device_handle`: A valid connected `NativeDeviceHandle`.
/// - `property`: Pointer to a `NativeSetProperty` struct defining interface, path, and value.
/// - `callback`: Callback function invoked upon set completion.
/// - `user_data`: Opaque user data pointer passed to `callback`.
///
/// # Safety
/// - `device_handle` must be valid at least until `callback` completes.
/// - `property` and its referenced memory must remain valid until this function returns.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn device_handle_set_property(
    device_handle: NativeDeviceHandle,
    property: *const NativeSetProperty,
    callback: DeviceHandleSendCallback,
    user_data: UserData,
) {
    let set = move |res: eyre::Result<()>| {
        let c_res = NativeVoidResult::unwrap_c_repr_of(res);

        callback(&c_res, user_data);
    };

    // SAFETY: the device_handle pointer has to be valid at least until the callback returns
    unsafe {
        device_handle.set_property(property, set);
    }
}

/// Unset a property on the device
///
/// Asynchronously unsets (deletes) a property on an Astarte property interface and path.
///
/// # Parameters
/// - `device_handle`: A valid connected `NativeDeviceHandle`.
/// - `property`: Pointer to a `NativePropertyIdentifier` specifying interface and path.
/// - `callback`: Callback function invoked upon unset completion.
/// - `user_data`: Opaque user data pointer passed to `callback`.
///
/// # Safety
/// - `device_handle` must be valid at least until `callback` completes.
/// - `property` and its referenced memory must remain valid until this function returns.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn device_handle_unset_property(
    device_handle: NativeDeviceHandle,
    property: *const NativePropertyIdentifier,
    callback: DeviceHandleSendCallback,
    user_data: UserData,
) {
    let set = move |res: eyre::Result<()>| {
        let c_res = NativeVoidResult::unwrap_c_repr_of(res);

        callback(&c_res, user_data);
    };

    // SAFETY: the device_handle pointer has to be valid at least until the callback returns
    unsafe {
        device_handle.unset_property(property, set);
    }
}

/// Callback type for device property get callbacks in FFI
///
/// Invoked when a property query operation completes. Receives a pointer to `NativeResult`
/// containing `NativeOption<NativeDeviceData>` and the user data pointer.
pub type DeviceHandleGetCallback =
    extern "C" fn(result: *mut NativeResult<NativeOption<NativeDeviceData>>, user_data: UserData);

/// Get a property value from the device
///
/// Asynchronously queries the cached value of a property stored locally on the device.
///
/// # Parameters
/// - `device_handle`: A valid connected `NativeDeviceHandle`.
/// - `property`: Pointer to a `NativePropertyIdentifier` specifying interface and path.
/// - `callback`: Callback function invoked with the query result.
/// - `user_data`: Opaque user data pointer passed to `callback`.
///
/// # Memory Management
/// The `NativeOption<NativeDeviceData>` result passed to `callback` must be freed after reading
/// by calling [`device_handle_free_get_property`].
///
/// # Safety
/// - `device_handle` must be valid at least until `callback` completes.
/// - `property` and its referenced memory must remain valid until this function returns.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn device_handle_get_property(
    device_handle: NativeDeviceHandle,
    property: *const NativePropertyIdentifier,
    callback: DeviceHandleGetCallback,
    user_data: UserData,
) {
    let data = move |res: eyre::Result<Option<AstarteData>>| {
        let mut c_res = NativeResult::c_repr_of(res).unwrap();

        callback(&mut c_res, user_data);
    };

    // SAFETY: the device_handle pointer has to be valid at least until the callback returns
    unsafe {
        device_handle.get_property(property, data);
    }
}

/// Free resources allocated for a property get result
///
/// Deallocates memory owned by the property value result obtained during [`device_handle_get_property`].
///
/// # Parameters
/// - `property`: Pointer to the `NativeOption<NativeDeviceData>` to free.
///
/// # Safety
/// - `property` must be non-null, valid, and must not be accessed after calling this function.
// NOTE i'd like to pass a value but currently it is not supported since therer are unnamed structs in unions
#[unsafe(no_mangle)]
pub unsafe extern "C" fn device_handle_free_get_property(
    property: *mut NativeOption<NativeDeviceData>,
) {
    let Some(property) = (unsafe { property.as_mut() }) else {
        error!("the pointer has to be non null and valid");
        return;
    };

    if let Err(e) = property.do_drop() {
        error!("{e:#}");
    }
}

/// Callback type for device disconnect callbacks in FFI
///
/// Invoked when device disconnection completes.
pub type DeviceHandleDisconnectCallback =
    extern "C" fn(result: *const NativeVoidResult, user_data: UserData);

/// Disconnect the device from Astarte
///
/// Asynchronously disconnects the device client from Astarte and shuts down its background event loop task.
///
/// # Parameters
/// - `handle`: A valid connected `NativeDeviceHandle`.
/// - `disconnect_cbk`: Callback function invoked upon disconnection completion.
/// - `user_data`: Opaque user data pointer passed to `disconnect_cbk`.
///
/// # Memory Management
/// Disconnecting the device does not automatically destroy the handle object.
/// [`device_handle_free`] must still be called to release the handle and its Tokio runtime.
///
/// # Safety
/// - `handle` must be valid at least until `disconnect_cbk` completes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn device_handle_disconnect(
    handle: NativeDeviceHandle,
    disconnect_cbk: DeviceHandleDisconnectCallback,
    user_data: UserData,
) {
    let disconnected = move |res: eyre::Result<()>| {
        let c_res = NativeVoidResult::unwrap_c_repr_of(res);

        disconnect_cbk(&c_res, user_data);
    };

    // SAFETY: the device_handle pointer has to be valid at least until the callback returns
    unsafe { handle.disconnect(disconnected) };
}

/// Free a device handle
///
/// Releases all resources held by the `NativeDeviceHandle` and shuts down the underlying Tokio runtime.
///
/// # Parameters
/// - `handle`: The `NativeDeviceHandle` to free.
///
/// # Safety
/// The caller must ensure that no other thread is concurrently accessing `handle` and that no
/// subsequent FFI function calls will use `handle`.
#[unsafe(no_mangle)]
pub extern "C" fn device_handle_free(handle: NativeDeviceHandle) {
    // SAFETY the caller has to ensure that this is the only thread accessing the handle
    // and that no other function will be called using this handle
    if let Err(e) = unsafe { handle.free() } {
        error!("{e:#}");
    }
}
