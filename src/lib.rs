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
use std::ffi::CStr;
use std::os::raw::c_char;

pub struct AstarteDevice {
    inner: DeviceClient<Mqtt<MemoryStore>>,
    rt: Runtime,
}

// Helper to safely convert C Strings to Rust Strings
unsafe fn c_str_to_string(ptr: *const c_char) -> String {
    CStr::from_ptr(ptr).to_string_lossy().into_owned()
}

#[no_mangle]
pub unsafe extern "C" fn astarte_device_new(
    realm: *const c_char,
    device_id: *const c_char,
    credentials_secret: *const c_char,
    pairing_url: *const c_char,
    ignore_ssl: bool,
    interfaces_dir: *const c_char,
) -> *mut AstarteDevice {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    let rt = Runtime::new().unwrap();

    let r_realm = c_str_to_string(realm);
    let r_device_id = c_str_to_string(device_id);
    let r_secret = c_str_to_string(credentials_secret);
    let r_url = c_str_to_string(pairing_url);
    let r_dir = c_str_to_string(interfaces_dir);

    let (device, mut connection) = rt.block_on(async {
        let mut opts = MqttConfig::with_credential_secret(&r_realm, &r_device_id, &r_secret, &r_url);

        if ignore_ssl {
            opts.ignore_ssl_errors();
        }

        DeviceBuilder::new()
            .store(MemoryStore::new())
            .interface_directory(&r_dir)
            .unwrap()
            .connection(opts)
            .build()
            .await
            .unwrap()
    });

    rt.spawn(async move {
        let _ = connection.handle_events().await;
        println!("disconnected");
    });

    // Allocate on the heap and return a raw pointer
    Box::into_raw(Box::new(AstarteDevice { inner: device, rt }))
}

#[no_mangle]
pub unsafe extern "C" fn astarte_device_send(
    ptr: *mut AstarteDevice,
    interface_name: *const c_char,
    interface_path: *const c_char,
    value: i32,
) {
    if ptr.is_null() { return; }

    // Borrow the pointer, do not consume it
    let device_wrapper = &*ptr;
    let mut device = device_wrapper.inner.clone();

    let r_name = c_str_to_string(interface_name);
    let r_path = c_str_to_string(interface_path);

    device_wrapper.rt.block_on(async move {
        let _ = device
            .send_individual_with_timestamp(&r_name, &r_path, AstarteData::Integer(value), Utc::now())
            .await
            .unwrap();
    });
}

#[no_mangle]
pub unsafe extern "C" fn astarte_device_start_listening(ptr: *mut AstarteDevice) {
    if ptr.is_null() { return; }
    let device_wrapper = &*ptr;
    let device = device_wrapper.inner.clone();

    device_wrapper.rt.block_on(async move {
        loop {
            match device.recv().await {
                Ok(event) => {
                    match event.data {
                        astarte_device_sdk::Value::Individual { .. } => println!("receive Individual"),
                        astarte_device_sdk::Value::Object { .. } => println!("receive Object"),
                        astarte_device_sdk::Value::Property(_) => println!("receive prop"),
                    };
                }
                Err(e) => println!("receive error: {:?}", e),
            }
        }
    });
}

// Crucial: You must provide a way for Java to free the memory!
#[no_mangle]
pub unsafe extern "C" fn astarte_device_free(ptr: *mut AstarteDevice) {
    if !ptr.is_null() {
        // Reconstruct the Box and let it drop
        drop(Box::from_raw(ptr));
    }
}
