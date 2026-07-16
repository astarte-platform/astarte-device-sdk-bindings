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

use std::{f64, option::Option};

use astarte_device_sdk::{
    AstarteData, DeviceEvent, aggregate::AstarteObject, chrono::Utc, types::Double,
};
use astarte_device_sdk_bindings::{
    NativeManuallyDrop, NativeOption, NativeResult, NativeVoidResult, UserData,
    config::{
        ConnectionConfig, DeviceConfig, GenericDeviceConfig, MqttConnectionConfig,
        NativeDeviceConfig,
    },
    data::{
        IndividualSend, NativeDeviceData, NativeDeviceEvent, NativeIndividualSend,
        NativeObjectSend, NativePropertyIdentifier, NativeSetProperty, ObjectSend,
        PropertyIdentifier, SetProperty,
    },
    device_handle_connect, device_handle_disconnect, device_handle_free,
    device_handle_free_device_event, device_handle_free_get_property, device_handle_get_property,
    device_handle_init, device_handle_receive, device_handle_send_individual,
    device_handle_send_object, device_handle_set_property, device_handle_unset_property,
};
use ffi_convert::{AsRust, CReprOf};
use serde::Deserialize;
use tokio::{
    sync::oneshot,
    task,
    time::{Duration, sleep},
};
use tracing::{error, info};
use url::Url;

#[derive(Deserialize)]
struct Config {
    realm: String,
    device_id: String,
    credentials_secret: String,
    pairing_url: Url,
    wait_receive: bool,
}

extern "C" fn void_result_cbk(result: *const NativeVoidResult, user_data: UserData) {
    let res = unsafe { result.as_ref().unwrap() };
    let res = res.as_rust().unwrap();

    let sender: Box<oneshot::Sender<eyre::Result<()>>> =
        unsafe { Box::from_raw(user_data.inner().cast()) };

    let _ = sender.send(res);
}

extern "C" fn get_property_cbk(
    result: *mut NativeResult<NativeOption<NativeDeviceData>>,
    user_data: UserData,
) {
    let res = unsafe { result.as_mut().unwrap() };

    let sender: Box<oneshot::Sender<eyre::Result<Option<AstarteData>>>> =
        unsafe { Box::from_raw(user_data.inner().cast()) };

    let converted: eyre::Result<Option<AstarteData>> = match res {
        NativeResult::Ok(opt) => {
            let rust_opt = opt.as_rust().unwrap();

            // NOTE after the as_rust copy, free the received native property
            unsafe {
                device_handle_free_get_property(opt as *mut NativeOption<NativeDeviceData>);
            }

            Ok(rust_opt)
        }
        NativeResult::Err(err_str) => {
            let err_msg: String = err_str.as_rust().unwrap();
            Err(eyre::eyre!(err_msg))
        }
    };

    let _ = sender.send(converted);
}

extern "C" fn receive_cbk(
    result: *mut NativeResult<NativeManuallyDrop<NativeDeviceEvent>>,
    user_data: UserData,
) {
    let sender: Box<oneshot::Sender<eyre::Result<DeviceEvent>>> =
        unsafe { Box::from_raw(user_data.inner().cast()) };

    let res = unsafe { result.as_mut().unwrap() };
    let converted = match res {
        NativeResult::Ok(native_event) => {
            info!(?native_event, "received device event");

            let event = native_event.as_rust().unwrap();

            // NOTE after the as_rust copy, free the passed event
            let native_event: *mut NativeDeviceEvent =
                (native_event as *mut NativeManuallyDrop<NativeDeviceEvent>).cast();
            unsafe { device_handle_free_device_event(native_event) }

            Ok(event)
        }
        NativeResult::Err(err) => {
            let msg: String = err.as_rust().unwrap();
            error!(%msg, "error receiving device event");
            Err(eyre::eyre!(msg))
        }
    };

    let _ = sender.send(converted);
}

fn cbk_channel<T>() -> (Box<oneshot::Sender<T>>, oneshot::Receiver<T>) {
    let (tx, rx) = oneshot::channel();

    (Box::new(tx), rx)
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let handle = device_handle_init();

    let json_config = tokio::fs::read_to_string("./examples/simple/config.json").await?;
    let json_config: Config = serde_json::from_str(&json_config)?;

    // device config
    let connection = ConnectionConfig::Mqtt(MqttConnectionConfig {
        device_id: json_config.device_id.to_string(),
        cred_secr: json_config.credentials_secret.to_string(),
        realm: json_config.realm.to_string(),
        pairing_url: json_config.pairing_url.to_string(),
    });
    let generic = GenericDeviceConfig {
        interfaces_dir: "./examples/simple/interfaces".to_string(),
        channel_size: 0,
        writable_dir: "/tmp/bindings-test".to_string(),
    };
    let device_config = DeviceConfig {
        connection,
        generic,
    };

    let native_device_config = NativeDeviceConfig::c_repr_of(device_config)?;

    let (build_tx, build_rx) = cbk_channel::<eyre::Result<()>>();
    let (loop_tx, loop_rx) = cbk_channel::<eyre::Result<()>>();

    info!("connecting device handle...");
    unsafe {
        device_handle_connect(
            handle,
            &native_device_config,
            void_result_cbk,
            UserData::new(Box::into_raw(build_tx).cast()),
            void_result_cbk,
            UserData::new(Box::into_raw(loop_tx).cast()),
        );
    };

    let build_res = build_rx.await?;
    info!(?build_res, "build result");
    build_res?;

    task::spawn(async move {
        match loop_rx.await {
            Ok(Ok(())) => info!("event loop exited"),
            Ok(Err(e)) => error!(%e, "event loop exited with error"),
            Err(e) => error!(%e, "event loop channel closed"),
        };
    });

    sleep(Duration::from_secs(2)).await;

    // 1. Send Individual Datastream
    info!("sending individual datastream...");
    let individual_data = IndividualSend {
        interface: "org.astarte-platform.rust.e2etest.DeviceDatastream".to_string(),
        path: "/doublearray_endpoint".to_string(),
        data: AstarteData::DoubleArray(vec![
            Double::try_from(1.0)?,
            Double::try_from(1.1)?,
            Double::try_from(1.2)?,
            Double::try_from(1.3)?,
            Double::try_from(10999999999.49)?,
        ]),
        timestamp: Some(Utc::now()),
    };
    let native_individual = NativeIndividualSend::c_repr_of(individual_data)?;
    let (tx, rx) = cbk_channel::<eyre::Result<()>>();
    unsafe {
        device_handle_send_individual(
            handle,
            &native_individual,
            void_result_cbk,
            UserData::new(Box::into_raw(tx).cast()),
        );
    }
    let res = rx.await?;
    info!(?res, "send_individual result");
    res?;

    // 2. Send Object Aggregate
    info!("sending object aggregate...");
    let object_data = ObjectSend {
        interface: "org.astarte-platform.rust.e2etest.DeviceAggregate".to_string(),
        path: "/test".to_string(),
        data: AstarteObject::from_iter([
            (
                "double_endpoint".to_string(),
                AstarteData::Double(Double::try_from(f64::consts::PI)?),
            ),
            ("integer_endpoint".to_string(), AstarteData::Integer(1)),
            ("boolean_endpoint".to_string(), AstarteData::Boolean(true)),
            (
                "longinteger_endpoint".to_string(),
                AstarteData::LongInteger(1 << 32),
            ),
            (
                "string_endpoint".to_string(),
                AstarteData::String("hey".to_string()),
            ),
            (
                "binaryblob_endpoint".to_string(),
                AstarteData::BinaryBlob(b"tests".to_vec()),
            ),
            (
                "datetime_endpoint".to_string(),
                AstarteData::DateTime(Utc::now()),
            ),
            (
                "doublearray_endpoint".to_string(),
                AstarteData::DoubleArray(vec![
                    Double::try_from(1.1)?,
                    Double::try_from(1.2)?,
                    Double::try_from(1.3)?,
                ]),
            ),
            (
                "integerarray_endpoint".to_string(),
                AstarteData::IntegerArray(vec![1, 2, 3]),
            ),
            (
                "booleanarray_endpoint".to_string(),
                AstarteData::BooleanArray(vec![true, false, true]),
            ),
            (
                "longintegerarray_endpoint".to_string(),
                AstarteData::LongIntegerArray(vec![1 << 33, 1 << 34]),
            ),
            (
                "stringarray_endpoint".to_string(),
                AstarteData::StringArray(vec!["a".to_string(), "b".to_string(), "c".to_string()]),
            ),
            (
                "binaryblobarray_endpoint".to_string(),
                AstarteData::BinaryBlobArray(vec![
                    b"blob1".to_vec(),
                    b"blob2".to_vec(),
                    b"this is a test binary blob".to_vec(),
                ]),
            ),
            (
                "datetimearray_endpoint".to_string(),
                AstarteData::DateTimeArray(vec![Utc::now(), Utc::now()]),
            ),
        ]),
        timestamp: Some(Utc::now()),
    };
    let native_object = NativeObjectSend::c_repr_of(object_data)?;
    let (tx, rx) = cbk_channel::<eyre::Result<()>>();
    unsafe {
        device_handle_send_object(
            handle,
            &native_object,
            void_result_cbk,
            UserData::new(Box::into_raw(tx).cast()),
        );
    }
    let res = rx.await?;
    info!(?res, "send_object result");
    res?;

    // 3. Set Property
    info!("setting property...");
    let set_prop_data = SetProperty {
        interface: "org.astarte-platform.rust.e2etest.ForUpdateDeviceProperty".to_string(),
        path: "/sensor_1/endpoint".to_string(),
        data: AstarteData::Double(Double::try_from(3231.1231)?),
    };
    let native_set_prop = NativeSetProperty::c_repr_of(set_prop_data)?;
    let (tx, rx) = cbk_channel::<eyre::Result<()>>();
    unsafe {
        device_handle_set_property(
            handle,
            &native_set_prop,
            void_result_cbk,
            UserData::new(Box::into_raw(tx).cast()),
        );
    }
    let res = rx.await?;
    info!(?res, "set_property result");
    res?;

    // 4. Unset Property
    info!("unsetting property...");
    let unset_prop_data = PropertyIdentifier {
        interface: "org.astarte-platform.rust.e2etest.DeviceProperty".to_string(),
        path: "/sensor_1/doublearray_endpoint".to_string(),
    };
    let native_unset_prop = NativePropertyIdentifier::c_repr_of(unset_prop_data)?;
    let (tx, rx) = cbk_channel::<eyre::Result<()>>();
    unsafe {
        device_handle_unset_property(
            handle,
            &native_unset_prop,
            void_result_cbk,
            UserData::new(Box::into_raw(tx).cast()),
        );
    }
    let res = rx.await?;
    info!(?res, "unset_property result");
    res?;

    // 5. Get Property
    info!("getting property...");
    let get_prop_data = PropertyIdentifier {
        interface: "org.astarte-platform.rust.e2etest.ForUpdateDeviceProperty".to_string(),
        path: "/sensor_1/endpoint".to_string(),
    };
    let native_get_prop = NativePropertyIdentifier::c_repr_of(get_prop_data)?;
    let (tx, rx) = cbk_channel::<eyre::Result<Option<AstarteData>>>();
    unsafe {
        device_handle_get_property(
            handle,
            &native_get_prop,
            get_property_cbk,
            UserData::new(Box::into_raw(tx).cast()),
        );
    }
    let res = rx.await?;
    info!(?res, "get_property result");
    let _prop_val = res?;

    // 6. Receive data callback
    if json_config.wait_receive {
        info!("registering receive callback...");
        let (tx, rx) = cbk_channel::<eyre::Result<DeviceEvent>>();
        unsafe {
            device_handle_receive(handle, receive_cbk, UserData::new(Box::into_raw(tx).cast()));
        }

        let event = rx.await?;
        info!(?event, "received device event");
        event?;
    }
    // 7. Disconnect device
    info!("disconnecting device...");
    let (tx, rx) = cbk_channel::<eyre::Result<()>>();
    unsafe {
        device_handle_disconnect(
            handle,
            void_result_cbk,
            UserData::new(Box::into_raw(tx).cast()),
        );
    }
    let res = rx.await?;
    info!(?res, "disconnect result");
    res?;

    // 8. Free device handle
    info!("freeing device handle...");
    device_handle_free(handle);
    info!("device handle freed successfully!");

    Ok(())
}
