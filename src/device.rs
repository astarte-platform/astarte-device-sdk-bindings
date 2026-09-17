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

use std::{num::NonZeroUsize, str::FromStr, sync::Arc};

use astarte_device_sdk::{
    AstarteData, Client, DeviceEvent, EventLoop,
    aggregate::AstarteObject,
    builder::{DEFAULT_CHANNEL_SIZE, DeviceBuilder},
    chrono::{DateTime, Utc},
    client::ClientConnection,
    pairing::api::PairingApi,
    properties::PropAccess,
    store::memory::MemoryStore,
    transport::{
        grpc::{Grpc, GrpcConfig, tonic::transport::Endpoint},
        mqtt::{Credential, Mqtt, MqttArgs, MqttConfig},
    },
};
use eyre::{Context, OptionExt};
use ffi_convert::{AsRust, CDrop, CReprOf, RawBorrow};
use tokio::{
    runtime::{Handle, Runtime},
    sync::RwLock,
    task::JoinHandle,
};
use tracing::error;
use url::Url;
use uuid::Uuid;

use crate::{
    config::{
        ConnectionConfig, DeviceConfig, GenericDeviceConfig, GrpcConnectionConfig,
        MqttConnectionConfig, NativeDeviceConfig,
    },
    data::{
        IndividualSend, NativeIndividualSend, NativeObjectSend, NativePropertyIdentifier,
        NativeSetProperty, ObjectSend, PropertyIdentifier, SetProperty,
    },
};

macro_rules! ok_or_call {
    ($expr:expr, $callback:ident) => {
        match $expr {
            Ok(h) => h,
            Err(e) => {
                $callback(Err(e));
                return;
            }
        }
    };
}

/// C-compatible handle wrapping the underlying async device runtime handle
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct NativeDeviceHandle(*mut DeviceRuntimeHandle);

impl NativeDeviceHandle {
    /// Create a new native device handle
    pub fn new() -> eyre::Result<Self> {
        let handle = Box::new(DeviceRuntimeHandle::new()?);

        Self::c_repr_of(handle).wrap_err("can't construct native handle")
    }

    unsafe fn raw_borrow<'a>(self) -> eyre::Result<&'a DeviceRuntimeHandle> {
        // SAFETY the caller has to ensure that self pointer remains valid for the required lifetime
        unsafe { DeviceRuntimeHandle::raw_borrow(self.0) }.wrap_err("can't borrow native handle")
    }

    pub(crate) unsafe fn connect<CF, EF>(
        self,
        config: *const NativeDeviceConfig,
        connected: CF,
        exited: EF,
    ) where
        CF: FnOnce(eyre::Result<()>) + Send + 'static,
        EF: FnOnce(eyre::Result<()>) + Send + 'static,
    {
        // SAFETY the caller has to ensure that self pointer remains until the callback is called
        let handle = ok_or_call!(unsafe { self.raw_borrow() }, connected);

        // SAFETY: the pointer has to be valid for the complete duration of the function
        let result = unsafe { config.as_ref() }
            .ok_or_eyre("config has to be a valid pointer")
            .and_then(|c| c.as_rust().wrap_err("can't convert config"));

        let config = ok_or_call!(result, connected);

        handle.connect(config, connected, exited);
    }

    pub(crate) unsafe fn receive<F>(self, received: F)
    where
        F: FnOnce(eyre::Result<DeviceEvent>) + Send + 'static,
    {
        // SAFETY the caller has to ensure that self pointer remains until the callback is called
        let handle = ok_or_call!(unsafe { self.raw_borrow() }, received);

        handle.receive(received);
    }

    pub(crate) unsafe fn send_individual<F>(self, individual: *const NativeIndividualSend, sent: F)
    where
        F: FnOnce(eyre::Result<()>) + Send + 'static,
    {
        // SAFETY the caller has to ensure that self pointer remains until the callback is called
        let handle = ok_or_call!(unsafe { self.raw_borrow() }, sent);

        // SAFETY: the pointer has to be valid for the complete duration of the function
        let individual = unsafe { individual.as_ref() }
            .ok_or_eyre("send data is required to be non null and valid")
            .and_then(|i| i.as_rust().wrap_err("can't convert individual"));

        let individual = ok_or_call!(individual, sent);

        handle.send_individual(individual, sent);
    }

    pub(crate) unsafe fn send_object<F>(self, object: *const NativeObjectSend, sent: F)
    where
        F: FnOnce(eyre::Result<()>) + Send + 'static,
    {
        // SAFETY the caller has to ensure that self pointer remains until the callback is called
        let handle = ok_or_call!(unsafe { self.raw_borrow() }, sent);

        // SAFETY: the pointer has to be valid for the complete duration of the function
        let object = unsafe { object.as_ref() }
            .ok_or_eyre("send data is required to be non null and valid")
            .and_then(|o| o.as_rust().wrap_err("can't convert object"));

        let object = ok_or_call!(object, sent);

        handle.send_object(object, sent);
    }

    pub(crate) unsafe fn set_property<F>(self, property: *const NativeSetProperty, sent: F)
    where
        F: FnOnce(eyre::Result<()>) + Send + 'static,
    {
        // SAFETY the caller has to ensure that self pointer remains until the callback is called
        let handle = ok_or_call!(unsafe { self.raw_borrow() }, sent);

        // SAFETY: the pointer has to be valid for the complete duration of the function
        let property = unsafe { property.as_ref() }
            .ok_or_eyre("send data is required to be non null and valid")
            .and_then(|i| i.as_rust().wrap_err("can't convert individual"));

        let property = ok_or_call!(property, sent);

        handle.set_property(property, sent);
    }

    pub(crate) unsafe fn unset_property<F>(self, property: *const NativePropertyIdentifier, sent: F)
    where
        F: FnOnce(eyre::Result<()>) + Send + 'static,
    {
        // SAFETY the caller has to ensure that self pointer remains until the callback is called
        let handle = ok_or_call!(unsafe { self.raw_borrow() }, sent);

        // SAFETY: the pointer has to be valid for the complete duration of the function
        let property = unsafe { property.as_ref() }
            .ok_or_eyre("send data is required to be non null and valid")
            .and_then(|i| i.as_rust().wrap_err("can't convert individual"));

        let property = ok_or_call!(property, sent);

        handle.unset_property(property, sent);
    }

    pub(crate) unsafe fn get_property<F>(self, property: *const NativePropertyIdentifier, data: F)
    where
        F: FnOnce(eyre::Result<Option<AstarteData>>) + Send + 'static,
    {
        // SAFETY the caller has to ensure that self pointer remains until the callback is called
        let handle = ok_or_call!(unsafe { self.raw_borrow() }, data);

        // SAFETY: the pointer has to be valid for the complete duration of the function
        let property = unsafe { property.as_ref() }
            .ok_or_eyre("send data is required to be non null and valid")
            .and_then(|i| i.as_rust().wrap_err("can't convert individual"));

        let property = ok_or_call!(property, data);

        handle.get_property(property, data);
    }

    pub(crate) unsafe fn disconnect<F>(self, disconnected: F)
    where
        F: FnOnce(eyre::Result<()>) + Send + 'static,
    {
        // NOTE this could be a raw_borrow_mut but that would mean that we require the caller to call
        // disconnect and make sure the pointer is not accessed anywhere else and I'm not sure this is something
        // we can encode easily in the python documentation so currently we are using an Arc<RwLock<..>>
        // using mut here would remove the need for the rwlock entirely

        // SAFETY: the caller has to ensure the pointer remains valid for the required lifetime
        let handle = ok_or_call!(unsafe { self.raw_borrow() }, disconnected);

        handle.disconnect(disconnected);
    }

    pub(crate) unsafe fn free(self) -> eyre::Result<()> {
        let handle = self.as_rust().wrap_err("error while dropping handle")?;

        handle.rt.shutdown_background();

        Ok(())
    }
}

impl CDrop for NativeDeviceHandle {
    fn do_drop(&mut self) -> Result<(), ffi_convert::CDropError> {
        let ptr: *mut DeviceRuntimeHandle = self.0;
        let _box = unsafe { Box::from_raw(ptr) };

        Ok(())
    }
}

// to allow converting a boxed device handle to a native device handle
impl CReprOf<Box<DeviceRuntimeHandle>> for NativeDeviceHandle {
    fn c_repr_of(input: Box<DeviceRuntimeHandle>) -> Result<Self, ffi_convert::CReprOfError> {
        let raw = Box::into_raw(input);

        Ok(Self(raw))
    }
}

impl AsRust<Box<DeviceRuntimeHandle>> for NativeDeviceHandle {
    fn as_rust(&self) -> Result<Box<DeviceRuntimeHandle>, ffi_convert::AsRustError> {
        // SAFETY when as_rust is called the caller must ensure noone is accessing the pointer
        let owned = unsafe { Box::from_raw(self.0) };

        Ok(owned)
    }
}

struct MqttMemoryStoreHandle {
    client: astarte_device_sdk::client::DeviceClient<Mqtt<MemoryStore, PairingApi>>,
}

impl MqttMemoryStoreHandle {
    pub(crate) async fn connect<F>(
        generic: GenericDeviceConfig,
        conn: MqttConnectionConfig,
        exited: F,
    ) -> eyre::Result<(Self, JoinHandle<()>)>
    where
        F: FnOnce(eyre::Result<()>) + Send + 'static,
    {
        let (client, connection) = Self::mk_device(generic, conn).await?;

        // NOTE after spawning the handle_events task we should not error
        let eloop = tokio::spawn(async move {
            let result = connection
                .handle_events()
                .await
                .wrap_err("handle events error");

            tokio::task::block_in_place(move || {
                exited(result);
            });
        });

        let handle = Self { client };

        Ok((handle, eloop))
    }

    async fn mk_device(
        generic: GenericDeviceConfig,
        con: MqttConnectionConfig,
    ) -> eyre::Result<(
        astarte_device_sdk::client::DeviceClient<Mqtt<MemoryStore, PairingApi>>,
        astarte_device_sdk::connection::DeviceConnection<Mqtt<MemoryStore, PairingApi>>,
    )> {
        let args = MqttArgs {
            realm: con.realm,
            device_id: con.device_id,
            credential: Credential::secret(con.cred_secr),
            pairing_url: Url::from_str(con.pairing_url.as_ref())?,
        };

        let mqtt_config = MqttConfig::new(args).ignore_ssl_errors();

        let channel_size = NonZeroUsize::new(generic.channel_size).unwrap_or(DEFAULT_CHANNEL_SIZE);

        let (client, connection) = DeviceBuilder::new()
            .writable_dir(generic.writable_dir)
            .channel_size(channel_size)
            .store(MemoryStore::new())
            .interface_directory(generic.interfaces_dir)?
            .connection(mqtt_config)
            .build()
            .await?;

        Ok((client, connection))
    }
}

impl DeviceMethods for MqttMemoryStoreHandle {
    fn receive(&self, rt: Handle, received: BoxedReceiveCallback) {
        let client = self.client.clone();

        rt.spawn(async move {
            let result = client.recv().await.ok_or_eyre("can't receive event");

            received(result);
        });
    }

    fn send_individual(
        &self,
        rt: Handle,
        interface: String,
        path: String,
        data: AstarteData,
        sent: BoxedCallback,
    ) {
        let client = self.client.clone();

        rt.spawn(async move {
            let mut client = client;
            let result = client
                .send_individual(&interface, &path, data)
                .await
                .wrap_err("can't send individual");

            sent(result);
        });
    }

    fn send_individual_ts(
        &self,
        rt: Handle,
        interface: String,
        path: String,
        data: AstarteData,
        timestamp: DateTime<Utc>,
        sent: BoxedCallback,
    ) {
        let client = self.client.clone();

        rt.spawn(async move {
            let mut client = client;
            let result = client
                .send_individual_with_timestamp(&interface, &path, data, timestamp)
                .await
                .wrap_err("can't send individual");

            sent(result);
        });
    }

    fn send_object(
        &self,
        rt: Handle,
        interface: String,
        path: String,
        data: AstarteObject,
        sent: BoxedCallback,
    ) {
        let client = self.client.clone();

        rt.spawn(async move {
            let mut client = client;
            let result = client
                .send_object(&interface, &path, data)
                .await
                .wrap_err("can't send object");

            sent(result);
        });
    }

    fn send_object_ts(
        &self,
        rt: Handle,
        interface: String,
        path: String,
        data: AstarteObject,
        timestamp: DateTime<Utc>,
        sent: BoxedCallback,
    ) {
        let client = self.client.clone();

        rt.spawn(async move {
            let mut client = client;
            let result = client
                .send_object_with_timestamp(&interface, &path, data, timestamp)
                .await
                .wrap_err("can't send object");

            sent(result);
        });
    }

    fn set_property(
        &self,
        rt: Handle,
        interface: String,
        path: String,
        data: AstarteData,
        sent: BoxedCallback,
    ) {
        let client = self.client.clone();

        rt.spawn(async move {
            let mut client = client;
            let result = client
                .set_property(&interface, &path, data)
                .await
                .wrap_err("can't set property");

            sent(result);
        });
    }

    fn unset_property(&self, rt: Handle, interface: String, path: String, sent: BoxedCallback) {
        let client = self.client.clone();

        rt.spawn(async move {
            let mut client = client;
            let result = client
                .unset_property(&interface, &path)
                .await
                .wrap_err("can't unset property");

            sent(result);
        });
    }

    fn get_property(
        &self,
        rt: Handle,
        interface: String,
        path: String,
        sent: BoxedGetPropertyCallback,
    ) {
        let client = self.client.clone();

        rt.spawn(async move {
            let result = client
                .property(&interface, &path)
                .await
                .wrap_err("can't get property");

            sent(result);
        });
    }

    fn disconnect(&self, rt: Handle, disconnected: BoxedCallback) {
        let client = self.client.clone();

        rt.spawn(async move {
            let mut client = client;
            let result = client
                .disconnect()
                .await
                .wrap_err("can't disconnect device");

            disconnected(result);
        });
    }
}

struct GrpcMemoryStoreHandle {
    client: astarte_device_sdk::client::DeviceClient<Grpc<MemoryStore>>,
}

impl GrpcMemoryStoreHandle {
    pub(crate) async fn connect<F>(
        generic: GenericDeviceConfig,
        conn: GrpcConnectionConfig,
        exited: F,
    ) -> eyre::Result<(Self, JoinHandle<()>)>
    where
        F: FnOnce(eyre::Result<()>) + Send + 'static,
    {
        let (client, connection) = Self::mk_device(generic, conn).await?;

        let eloop = tokio::spawn(async move {
            let result = connection
                .handle_events()
                .await
                .wrap_err("handle events error");

            tokio::task::block_in_place(move || {
                exited(result);
            });
        });

        let handle = Self { client };

        Ok((handle, eloop))
    }

    async fn mk_device(
        generic: GenericDeviceConfig,
        con: GrpcConnectionConfig,
    ) -> eyre::Result<(
        astarte_device_sdk::client::DeviceClient<Grpc<MemoryStore>>,
        astarte_device_sdk::connection::DeviceConnection<Grpc<MemoryStore>>,
    )> {
        let config = GrpcConfig::new(Uuid::new_v4(), Endpoint::from_str(&con.message_hub_addr)?);

        let channel_size =
            NonZeroUsize::new(generic.channel_size).ok_or_eyre("invalid channel size")?;

        let (client, connection) = DeviceBuilder::new()
            .writable_dir(generic.writable_dir)
            .channel_size(channel_size)
            .store(MemoryStore::new())
            .interface_directory(generic.interfaces_dir)?
            .connection(config)
            .build()
            .await?;

        Ok((client, connection))
    }
}

impl DeviceMethods for GrpcMemoryStoreHandle {
    fn receive(&self, rt: Handle, received: BoxedReceiveCallback) {
        let client = self.client.clone();

        rt.spawn(async move {
            let result = client.recv().await.ok_or_eyre("can't receive event");

            received(result);
        });
    }

    fn send_individual(
        &self,
        rt: Handle,
        interface: String,
        path: String,
        data: AstarteData,
        sent: BoxedCallback,
    ) {
        let client = self.client.clone();

        rt.spawn(async move {
            let mut client = client;
            let result = client
                .send_individual(&interface, &path, data)
                .await
                .wrap_err("can't send individual");

            sent(result);
        });
    }

    fn send_individual_ts(
        &self,
        rt: Handle,
        interface: String,
        path: String,
        data: AstarteData,
        timestamp: DateTime<Utc>,
        sent: BoxedCallback,
    ) {
        let client = self.client.clone();

        rt.spawn(async move {
            let mut client = client;
            let result = client
                .send_individual_with_timestamp(&interface, &path, data, timestamp)
                .await
                .wrap_err("can't send individual");

            sent(result);
        });
    }

    fn send_object(
        &self,
        rt: Handle,
        interface: String,
        path: String,
        data: AstarteObject,
        sent: BoxedCallback,
    ) {
        let client = self.client.clone();

        rt.spawn(async move {
            let mut client = client;
            let result = client
                .send_object(&interface, &path, data)
                .await
                .wrap_err("can't send object");

            sent(result);
        });
    }

    fn send_object_ts(
        &self,
        rt: Handle,
        interface: String,
        path: String,
        data: AstarteObject,
        timestamp: DateTime<Utc>,
        sent: BoxedCallback,
    ) {
        let client = self.client.clone();

        rt.spawn(async move {
            let mut client = client;
            let result = client
                .send_object_with_timestamp(&interface, &path, data, timestamp)
                .await
                .wrap_err("can't send object");

            sent(result);
        });
    }

    fn set_property(
        &self,
        rt: Handle,
        interface: String,
        path: String,
        data: AstarteData,
        sent: BoxedCallback,
    ) {
        let client = self.client.clone();

        rt.spawn(async move {
            let mut client = client;
            let result = client
                .set_property(&interface, &path, data)
                .await
                .wrap_err("can't set property");

            sent(result);
        });
    }

    fn unset_property(&self, rt: Handle, interface: String, path: String, sent: BoxedCallback) {
        let client = self.client.clone();

        rt.spawn(async move {
            let mut client = client;
            let result = client
                .unset_property(&interface, &path)
                .await
                .wrap_err("can't unset property");

            sent(result);
        });
    }

    fn get_property(
        &self,
        rt: Handle,
        interface: String,
        path: String,
        sent: BoxedGetPropertyCallback,
    ) {
        let client = self.client.clone();

        rt.spawn(async move {
            let result = client
                .property(&interface, &path)
                .await
                .wrap_err("can't get property");

            sent(result);
        });
    }

    fn disconnect(&self, rt: Handle, disconnected: BoxedCallback) {
        let client = self.client.clone();

        rt.spawn(async move {
            let mut client = client;
            let result = client
                .disconnect()
                .await
                .wrap_err("can't disconnect device");

            disconnected(result);
        });
    }
}

type BoxedCallback = Box<dyn FnOnce(eyre::Result<()>) + Send + 'static>;
type BoxedReceiveCallback = Box<dyn FnOnce(eyre::Result<DeviceEvent>) + Send + 'static>;
type BoxedGetPropertyCallback = Box<dyn FnOnce(eyre::Result<Option<AstarteData>>) + Send + 'static>;

type DynDeviceMethod = Box<dyn DeviceMethods + Send + Sync>;

trait DeviceMethods {
    fn receive(&self, rt: Handle, received: BoxedReceiveCallback);

    fn send_individual(
        &self,
        rt: Handle,
        interface: String,
        path: String,
        data: AstarteData,
        sent: BoxedCallback,
    );

    fn send_individual_ts(
        &self,
        rt: Handle,
        interface: String,
        path: String,
        data: AstarteData,
        timestamp: DateTime<Utc>,
        sent: BoxedCallback,
    );

    fn send_object(
        &self,
        rt: Handle,
        interface: String,
        path: String,
        data: AstarteObject,
        sent: BoxedCallback,
    );

    fn send_object_ts(
        &self,
        rt: Handle,
        interface: String,
        path: String,
        data: AstarteObject,
        timestamp: DateTime<Utc>,
        sent: BoxedCallback,
    );

    fn set_property(
        &self,
        rt: Handle,
        interface: String,
        path: String,
        data: AstarteData,
        sent: BoxedCallback,
    );

    fn unset_property(&self, rt: Handle, interface: String, path: String, sent: BoxedCallback);

    fn get_property(
        &self,
        rt: Handle,
        interface: String,
        path: String,
        property: BoxedGetPropertyCallback,
    );

    fn disconnect(&self, rt: Handle, disconnected: BoxedCallback);
}

struct InnerDeviceHandle {
    client: DynDeviceMethod,
    eloop: JoinHandle<()>,
}

impl InnerDeviceHandle {
    fn new(client: DynDeviceMethod, eloop: JoinHandle<()>) -> Self {
        Self { client, eloop }
    }

    pub(crate) fn receive(&self, rt: Handle, received: BoxedReceiveCallback) {
        self.client.receive(rt, received);
    }

    pub(crate) fn send_individual(
        &self,
        rt: Handle,
        individual: IndividualSend,
        sent: BoxedCallback,
    ) {
        let IndividualSend {
            interface,
            path,
            data,
            timestamp,
        } = individual;

        if let Some(timestamp) = timestamp {
            self.client
                .send_individual_ts(rt, interface, path, data, timestamp, sent);
        } else {
            self.client.send_individual(rt, interface, path, data, sent);
        }
    }

    pub(crate) fn send_object(&self, rt: Handle, object: ObjectSend, sent: BoxedCallback) {
        let ObjectSend {
            interface,
            path,
            data,
            timestamp,
        } = object;

        if let Some(timestamp) = timestamp {
            self.client
                .send_object_ts(rt, interface, path, data, timestamp, sent);
        } else {
            self.client.send_object(rt, interface, path, data, sent);
        }
    }

    pub(crate) fn set_property(&self, rt: Handle, property: SetProperty, sent: BoxedCallback) {
        let SetProperty {
            interface,
            path,
            data,
        } = property;

        self.client.set_property(rt, interface, path, data, sent);
    }

    pub(crate) fn unset_property(
        &self,
        rt: Handle,
        property: PropertyIdentifier,
        sent: BoxedCallback,
    ) {
        let PropertyIdentifier { interface, path } = property;

        self.client.unset_property(rt, interface, path, sent);
    }

    pub(crate) fn get_property(
        &self,
        rt: Handle,
        property: PropertyIdentifier,
        sent: BoxedGetPropertyCallback,
    ) {
        let PropertyIdentifier { interface, path } = property;

        self.client.get_property(rt, interface, path, sent);
    }

    pub(crate) async fn disconnect(self, rt: Handle, disconnected: BoxedCallback) {
        let Self { client, eloop } = self;

        client.disconnect(rt, disconnected);

        if let Err(error) = eloop.await {
            error!(%error, "error while joining event loop task");
        }
    }
}

pub(crate) struct DeviceRuntimeHandle {
    rt: Runtime,
    inner: Arc<RwLock<Option<InnerDeviceHandle>>>,
}

impl DeviceRuntimeHandle {
    pub(crate) fn new() -> eyre::Result<DeviceRuntimeHandle> {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

        let inner = Arc::new(RwLock::new(None));

        Ok(Self { rt, inner })
    }

    pub(crate) fn connect<CF, EF>(&self, config: DeviceConfig, connected: CF, exited: EF)
    where
        CF: FnOnce(eyre::Result<()>) + Send + 'static,
        EF: FnOnce(eyre::Result<()>) + Send + 'static,
    {
        let inner = Arc::clone(&self.inner);

        self.rt.spawn(async move {
            let mut lock = inner.write().await;

            if lock.is_some() {
                connected(Err(eyre::eyre!("device already connected")));
                return;
            }

            // write lock held across an await point but this guarantees we connect the device only a single time
            let result = Self::build_handles(config, exited).await;

            match result {
                Ok((handle, eloop)) => {
                    *lock = Some(InnerDeviceHandle::new(handle, eloop));

                    connected(Ok(()));
                }
                Err(e) => connected(Err(e)),
            }
        });
    }

    async fn build_handles<E>(
        config: DeviceConfig,
        exited: E,
    ) -> eyre::Result<(DynDeviceMethod, JoinHandle<()>)>
    where
        E: FnOnce(eyre::Result<()>) + Send + 'static,
    {
        let DeviceConfig {
            connection,
            generic,
        } = config;

        let result = match connection {
            ConnectionConfig::Mqtt(mqtt_connection_config) => {
                let (mqtt_handle, eloop) =
                    MqttMemoryStoreHandle::connect(generic, mqtt_connection_config, exited)
                        .await
                        .wrap_err("error while connecting mqtt device")?;

                (Box::new(mqtt_handle) as DynDeviceMethod, eloop)
            }
            ConnectionConfig::Grpc(grpc_connection_config) => {
                let (grpc_handle, eloop) =
                    GrpcMemoryStoreHandle::connect(generic, grpc_connection_config, exited)
                        .await
                        .wrap_err("error while connecting grpc device")?;

                (Box::new(grpc_handle) as DynDeviceMethod, eloop)
            }
        };

        Ok(result)
    }

    pub(crate) fn receive<F>(&self, received: F)
    where
        F: FnOnce(eyre::Result<DeviceEvent>) + Send + 'static,
    {
        let inner = Arc::clone(&self.inner);

        self.rt.spawn(async move {
            let lock = inner.read().await;
            let handle = ok_or_call!(lock.as_ref().ok_or_eyre("client not connected"), received);

            // TODO remove boxing by replacing with a struct that contains a function pointer and the UserData
            handle.receive(Handle::current(), Box::new(received));
        });
    }

    pub(crate) fn send_individual<F>(&self, individual: IndividualSend, sent: F)
    where
        F: FnOnce(eyre::Result<()>) + Send + 'static,
    {
        let inner = Arc::clone(&self.inner);

        self.rt.spawn(async move {
            let lock = inner.read().await;
            let handle = ok_or_call!(lock.as_ref().ok_or_eyre("client not connected"), sent);

            // TODO remove boxing by replacing with a struct that contains a function pointer and the UserData
            handle.send_individual(Handle::current(), individual, Box::new(sent));
        });
    }

    pub(crate) fn send_object<F>(&self, object: ObjectSend, sent: F)
    where
        F: FnOnce(eyre::Result<()>) + Send + 'static,
    {
        let inner = Arc::clone(&self.inner);

        self.rt.spawn(async move {
            let lock = inner.read().await;
            let handle = ok_or_call!(lock.as_ref().ok_or_eyre("client not connected"), sent);

            // TODO remove boxing by replacing with a struct that contains a function pointer and the UserData
            handle.send_object(Handle::current(), object, Box::new(sent));
        });
    }

    pub(crate) fn set_property<F>(&self, property: SetProperty, sent: F)
    where
        F: FnOnce(eyre::Result<()>) + Send + 'static,
    {
        let inner = Arc::clone(&self.inner);

        self.rt.spawn(async move {
            let lock = inner.read().await;
            let handle = ok_or_call!(lock.as_ref().ok_or_eyre("client not connected"), sent);

            // TODO remove boxing by replacing with a struct that contains a function pointer and the UserData
            handle.set_property(Handle::current(), property, Box::new(sent));
        });
    }

    pub(crate) fn unset_property<F>(&self, property: PropertyIdentifier, sent: F)
    where
        F: FnOnce(eyre::Result<()>) + Send + 'static,
    {
        let inner = Arc::clone(&self.inner);

        self.rt.spawn(async move {
            let lock = inner.read().await;
            let handle = ok_or_call!(lock.as_ref().ok_or_eyre("client not connected"), sent);

            // TODO remove boxing by replacing with a struct that contains a function pointer and the UserData
            handle.unset_property(Handle::current(), property, Box::new(sent));
        });
    }

    pub(crate) fn get_property<F>(&self, property: PropertyIdentifier, sent: F)
    where
        F: FnOnce(eyre::Result<Option<AstarteData>>) + Send + 'static,
    {
        let inner = Arc::clone(&self.inner);

        self.rt.spawn(async move {
            let lock = inner.read().await;
            let handle = ok_or_call!(lock.as_ref().ok_or_eyre("client not connected"), sent);

            // TODO remove boxing by replacing with a struct that contains a function pointer and the UserData
            handle.get_property(Handle::current(), property, Box::new(sent));
        });
    }

    // this does not free the handle that has to be freed afterward using [`free`]
    pub(crate) fn disconnect<F>(&self, disconnected: F)
    where
        F: FnOnce(eyre::Result<()>) + Send + 'static,
    {
        let inner = Arc::clone(&self.inner);

        self.rt.spawn(async move {
            let mut lock = inner.write().await;
            let handle = ok_or_call!(lock.take().ok_or_eyre("already disconnected"), disconnected);

            // TODO remove boxing by replacing with a struct that contains a function pointer and the UserData
            handle
                .disconnect(Handle::current(), Box::new(disconnected))
                .await;
        });
    }
}
