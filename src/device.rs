use std::{
    num::{NonZero, NonZeroUsize},
    str::FromStr,
    sync::{Arc, atomic::AtomicBool},
};

use astarte_device_sdk::{
    AstarteData, Client, DeviceEvent, EventLoop,
    builder::{DEFAULT_CHANNEL_SIZE, DeviceBuilder},
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
    runtime::Runtime,
    sync::{Mutex, OnceCell},
    task,
};
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

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct NativeDeviceHandle(*mut DeviceRuntimeHandle);

impl NativeDeviceHandle {
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

// FIXME this drop impl does nothing this is by design
// so that when we pass this inside NativeStringResult<NativeDeviceHandle>
// we know that this won't get dropped
// this is because the error that contains the cstring still needs to be freed
// to avoid doing this workaround we could
// - have a new type that does not require cdrop for the ok type but still owns and drops the error
// - provide a free function that does not drop the native device handle (which is only dropped when passed to disconnect)
//   and pass the result as value so that the drop does not get called by rust
impl CDrop for NativeDeviceHandle {
    fn do_drop(&mut self) -> Result<(), ffi_convert::CDropError> {
        // let ptr: *mut DeviceHandle = unsafe { mem::transmute(self.0) };
        // let _box = unsafe { Box::from_raw(ptr) };

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

#[derive(Debug, Clone)]
struct DeviceClientData<C> {
    client: C,
}

impl<C> DeviceClientData<C> {
    fn new(client: C) -> Self {
        Self { client }
    }

    pub(crate) async fn receive(&self) -> eyre::Result<DeviceEvent>
    where
        C: Client,
    {
        self.client.recv().await.wrap_err("can't receive event")
    }

    pub(crate) async fn send_individual(&mut self, individual: IndividualSend) -> eyre::Result<()>
    where
        C: Client,
    {
        let IndividualSend {
            interface,
            path,
            data,
            timestamp,
        } = individual;

        let result = if let Some(timestamp) = timestamp {
            self.client
                .send_individual_with_timestamp(&interface, &path, data, timestamp)
                .await
        } else {
            self.client.send_individual(&interface, &path, data).await
        };

        result.wrap_err("can't send individual")
    }

    pub(crate) async fn send_object(&mut self, object: ObjectSend) -> eyre::Result<()>
    where
        C: Client,
    {
        let ObjectSend {
            interface,
            path,
            data,
            timestamp,
        } = object;

        let result = if let Some(timestamp) = timestamp {
            self.client
                .send_object_with_timestamp(&interface, &path, data, timestamp)
                .await
        } else {
            self.client.send_object(&interface, &path, data).await
        };

        result.wrap_err("can't send object")
    }

    pub(crate) async fn set_property(&mut self, individual: SetProperty) -> eyre::Result<()>
    where
        C: Client,
    {
        let SetProperty {
            interface,
            path,
            data,
        } = individual;

        self.client
            .set_property(&interface, &path, data)
            .await
            .wrap_err("can't set property")
    }

    pub(crate) async fn unset_property(&mut self, property: PropertyIdentifier) -> eyre::Result<()>
    where
        C: Client,
    {
        let PropertyIdentifier { interface, path } = property;

        self.client
            .unset_property(&interface, &path)
            .await
            .wrap_err("can't unset property")
    }

    pub(crate) async fn get_property(
        &mut self,
        property: PropertyIdentifier,
    ) -> eyre::Result<Option<AstarteData>>
    where
        C: PropAccess,
    {
        let PropertyIdentifier { interface, path } = property;

        self.client
            .property(&interface, &path)
            .await
            .wrap_err("can't get property")
    }

    pub(crate) async fn disconnect(mut self) -> eyre::Result<()>
    where
        C: ClientConnection,
    {
        self.client
            .disconnect()
            .await
            .wrap_err("can't disconnect astarte client")
    }
}

struct MqttMemoryStoreHandle {
    client:
        DeviceClientData<astarte_device_sdk::client::DeviceClient<Mqtt<MemoryStore, PairingApi>>>,
    loop_handle: Arc<Mutex<Option<task::JoinHandle<()>>>>,
}

impl MqttMemoryStoreHandle {
    pub(crate) async fn connect<F>(
        generic: GenericDeviceConfig,
        conn: MqttConnectionConfig,
        exited: F,
    ) -> eyre::Result<Self>
    where
        F: FnOnce(eyre::Result<()>) + Send + 'static,
    {
        let (client, connection) = Self::mk_device(generic, conn).await?;

        let loop_handle = tokio::spawn(async move {
            let result = connection
                .handle_events()
                .await
                .wrap_err("handle events error");

            tokio::task::block_in_place(move || {
                exited(result);
            });
        });

        Ok(Self {
            client: DeviceClientData::new(client),
            loop_handle: Arc::new(Mutex::new(Some(loop_handle))),
        })
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

    pub(crate) async fn disconnect(&self) -> eyre::Result<()> {
        self.client.clone().disconnect().await?;

        let loop_handle = self
            .loop_handle
            .lock()
            .await
            .take()
            .ok_or(eyre::eyre!("already disconnected"))?;

        loop_handle.await.wrap_err("can't join handle_events")
    }
}

impl DeviceMethods for MqttMemoryStoreHandle {
    fn receive(&self, rt: &Runtime, received: BoxedReceiveCallback) {
        let client = self.client.clone();

        rt.spawn(async move {
            let result = client.receive().await;

            received(result);
        });
    }

    fn send_individual(&self, rt: &Runtime, individual: IndividualSend, sent: BoxedCallback) {
        let mut client = self.client.clone();

        rt.spawn(async move {
            let result = client.send_individual(individual).await;

            sent(result);
        });
    }

    fn send_object(&self, rt: &Runtime, object: ObjectSend, sent: BoxedCallback) {
        let mut client = self.client.clone();

        rt.spawn(async move {
            let result = client.send_object(object).await;

            sent(result);
        });
    }

    fn set_property(&self, rt: &Runtime, property: SetProperty, sent: BoxedCallback) {
        let mut client = self.client.clone();

        rt.spawn(async move {
            let result = client.set_property(property).await;

            sent(result);
        });
    }

    fn unset_property(&self, rt: &Runtime, property: PropertyIdentifier, sent: BoxedCallback) {
        let mut client = self.client.clone();

        rt.spawn(async move {
            let result = client.unset_property(property).await;

            sent(result);
        });
    }

    fn get_property(
        &self,
        rt: &Runtime,
        property: PropertyIdentifier,
        sent: BoxedGetPropertyCallback,
    ) {
        let mut client = self.client.clone();

        rt.spawn(async move {
            let result = client.get_property(property).await;

            sent(result);
        });
    }

    fn disconnect(&self, rt: &Runtime, disconnected: BoxedCallback) {
        let client = self.client.clone();
        let handle = Arc::clone(&self.loop_handle);

        rt.spawn(async move {
            ok_or_call!(client.disconnect().await, disconnected);

            let result = handle
                .lock()
                .await
                .take()
                .ok_or_eyre("already disconnected");

            let handle = ok_or_call!(result, disconnected);

            let result = handle.await.wrap_err("can't join handle events task");

            disconnected(result);
        });
    }
}

struct GrpcMemoryStoreHandle {
    client: DeviceClientData<astarte_device_sdk::client::DeviceClient<Grpc<MemoryStore>>>,
    loop_handle: Arc<Mutex<Option<task::JoinHandle<()>>>>,
}

impl GrpcMemoryStoreHandle {
    pub(crate) async fn connect<F>(
        generic: GenericDeviceConfig,
        conn: GrpcConnectionConfig,
        exited: F,
    ) -> eyre::Result<Self>
    where
        F: FnOnce(eyre::Result<()>) + Send + 'static,
    {
        let (client, connection) = Self::mk_device(generic, conn).await?;

        let loop_handle = tokio::spawn(async move {
            let result = connection
                .handle_events()
                .await
                .wrap_err("handle events error");

            tokio::task::block_in_place(move || {
                exited(result);
            });
        });

        Ok(Self {
            client: DeviceClientData::new(client),
            loop_handle: Arc::new(Mutex::new(Some(loop_handle))),
        })
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

    pub(crate) async fn disconnect(&self) -> eyre::Result<()> {
        self.client.clone().disconnect().await?;

        let loop_handle = self
            .loop_handle
            .lock()
            .await
            .take()
            .ok_or(eyre::eyre!("already disconnected"))?;

        loop_handle.await.wrap_err("can't join handle_events")
    }
}

impl DeviceMethods for GrpcMemoryStoreHandle {
    fn receive(&self, rt: &Runtime, received: BoxedReceiveCallback) {
        let client = self.client.clone();

        rt.spawn(async move {
            let result = client.receive().await;

            received(result);
        });
    }

    fn send_individual(&self, rt: &Runtime, individual: IndividualSend, sent: BoxedCallback) {
        let mut client = self.client.clone();

        rt.spawn(async move {
            let result = client.send_individual(individual).await;

            sent(result);
        });
    }

    fn send_object(&self, rt: &Runtime, object: ObjectSend, sent: BoxedCallback) {
        let mut client = self.client.clone();

        rt.spawn(async move {
            let result = client.send_object(object).await;

            sent(result);
        });
    }

    fn set_property(&self, rt: &Runtime, property: SetProperty, sent: BoxedCallback) {
        let mut client = self.client.clone();

        rt.spawn(async move {
            let result = client.set_property(property).await;

            sent(result);
        });
    }

    fn unset_property(&self, rt: &Runtime, property: PropertyIdentifier, sent: BoxedCallback) {
        let mut client = self.client.clone();

        rt.spawn(async move {
            let result = client.unset_property(property).await;

            sent(result);
        });
    }

    fn get_property(
        &self,
        rt: &Runtime,
        property: PropertyIdentifier,
        sent: BoxedGetPropertyCallback,
    ) {
        let mut client = self.client.clone();

        rt.spawn(async move {
            let result = client.get_property(property).await;

            sent(result);
        });
    }

    fn disconnect(&self, rt: &Runtime, disconnected: BoxedCallback) {
        let client = self.client.clone();
        let handle = Arc::clone(&self.loop_handle);

        rt.spawn(async move {
            ok_or_call!(client.disconnect().await, disconnected);

            let result = handle
                .lock()
                .await
                .take()
                .ok_or_eyre("already disconnected");

            let handle = ok_or_call!(result, disconnected);

            let result = handle.await.wrap_err("can't join handle events task");

            disconnected(result);
        });
    }
}

type BoxedCallback = Box<dyn FnOnce(eyre::Result<()>) + Send + 'static>;
type BoxedReceiveCallback = Box<dyn FnOnce(eyre::Result<DeviceEvent>) + Send + 'static>;
type BoxedGetPropertyCallback = Box<dyn FnOnce(eyre::Result<Option<AstarteData>>) + Send + 'static>;

trait DeviceMethods {
    // fn connect(&self, config: DeviceConfig, connected: BoxedCallback, exited: BoxedCallback);

    fn receive(&self, rt: &Runtime, received: BoxedReceiveCallback);

    fn send_individual(&self, rt: &Runtime, individual: IndividualSend, sent: BoxedCallback);

    fn send_object(&self, rt: &Runtime, object: ObjectSend, sent: BoxedCallback);

    fn set_property(&self, rt: &Runtime, property: SetProperty, sent: BoxedCallback);

    fn unset_property(&self, rt: &Runtime, property: PropertyIdentifier, sent: BoxedCallback);

    fn get_property(
        &self,
        rt: &Runtime,
        property: PropertyIdentifier,
        sent: BoxedGetPropertyCallback,
    );

    fn disconnect(&self, rt: &Runtime, disconnected: BoxedCallback);
}

type DynDeviceMethod = Box<dyn DeviceMethods + Send + Sync>;

pub struct DeviceRuntimeHandle {
    rt: Runtime,
    inner: Arc<OnceCell<DynDeviceMethod>>,
}

impl DeviceRuntimeHandle {
    pub fn new() -> eyre::Result<DeviceRuntimeHandle> {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

        let inner = Arc::new(OnceCell::new());

        Ok(Self { rt, inner })
    }

    pub fn connect<CF, EF>(&self, config: DeviceConfig, connected: CF, exited: EF)
    where
        CF: FnOnce(eyre::Result<()>) + Send + 'static,
        EF: FnOnce(eyre::Result<()>) + Send + 'static,
    {
        let inner = Arc::clone(&self.inner);

        self.rt.spawn(async move {
            if inner.initialized() {
                connected(Err(eyre::eyre!("device already configured")));
                return;
            }

            // FIXME if the OnceCell was already initialized we would never call the [`exited`] callback
            // this will cause problems since if we return Ok from connncted we should also call exited
            // so this was the easiest fix i could find
            let mut init_here = false;

            let res = inner
                .get_or_try_init(|| async {
                    init_here = true;

                    let DeviceConfig {
                        connection,
                        generic,
                    } = config;

                    let handle = match connection {
                        ConnectionConfig::Mqtt(mqtt_connection_config) => {
                            let mqtt_handle = MqttMemoryStoreHandle::connect(
                                generic,
                                mqtt_connection_config,
                                exited,
                            )
                            .await
                            .wrap_err("error while connecting mqtt device")?;

                            Box::new(mqtt_handle) as DynDeviceMethod
                        }
                        ConnectionConfig::Grpc(grpc_connection_config) => {
                            let grpc_handle = GrpcMemoryStoreHandle::connect(
                                generic,
                                grpc_connection_config,
                                exited,
                            )
                            .await
                            .wrap_err("error while connecting grpc device")?;

                            Box::new(grpc_handle) as DynDeviceMethod
                        }
                    };

                    eyre::Result::Ok(handle)
                })
                .await;

            if !init_here {
                connected(Err(eyre::eyre!("already initialized elsewhere")));
                return;
            }

            connected(res.map(|_| ()));
        });
    }

    pub fn receive<F>(&self, received: F)
    where
        F: FnOnce(eyre::Result<DeviceEvent>) + Send + 'static,
    {
        let client = ok_or_call!(
            self.inner.get().ok_or_eyre("client not connected"),
            received
        );

        // TODO remove boxing by replacing with a struct that conataines a function pointer and the UserData
        client.receive(&self.rt, Box::new(received));
    }

    pub fn send_individual<F>(&self, individual: IndividualSend, sent: F)
    where
        F: FnOnce(eyre::Result<()>) + Send + 'static,
    {
        let client = ok_or_call!(self.inner.get().ok_or_eyre("client not connected"), sent);

        // TODO remove boxing by replacing with a struct that conataines a function pointer and the UserData
        client.send_individual(&self.rt, individual, Box::new(sent));
    }

    pub fn send_object<F>(&self, object: ObjectSend, sent: F)
    where
        F: FnOnce(eyre::Result<()>) + Send + 'static,
    {
        let client = ok_or_call!(self.inner.get().ok_or_eyre("client not connected"), sent);

        // TODO remove boxing by replacing with a struct that conataines a function pointer and the UserData
        client.send_object(&self.rt, object, Box::new(sent));
    }

    pub fn set_property<F>(&self, property: SetProperty, sent: F)
    where
        F: FnOnce(eyre::Result<()>) + Send + 'static,
    {
        let client = ok_or_call!(self.inner.get().ok_or_eyre("client not connected"), sent);

        // TODO remove boxing by replacing with a struct that conataines a function pointer and the UserData
        client.set_property(&self.rt, property, Box::new(sent));
    }

    pub fn unset_property<F>(&self, property: PropertyIdentifier, sent: F)
    where
        F: FnOnce(eyre::Result<()>) + Send + 'static,
    {
        let client = ok_or_call!(self.inner.get().ok_or_eyre("client not connected"), sent);

        // TODO remove boxing by replacing with a struct that conataines a function pointer and the UserData
        client.unset_property(&self.rt, property, Box::new(sent));
    }

    pub fn get_property<F>(&self, property: PropertyIdentifier, sent: F)
    where
        F: FnOnce(eyre::Result<Option<AstarteData>>) + Send + 'static,
    {
        let client = ok_or_call!(self.inner.get().ok_or_eyre("client not connected"), sent);

        // TODO remove boxing by replacing with a struct that conataines a function pointer and the UserData
        client.get_property(&self.rt, property, Box::new(sent));
    }

    // this does not free the handle that has to be freed after using [`free`]
    pub fn disconnect<F>(&self, disconnected: F)
    where
        F: FnOnce(eyre::Result<()>) + Send + 'static,
    {
        let client = ok_or_call!(
            self.inner.get().ok_or_eyre("client not connected"),
            disconnected
        );

        client.disconnect(&self.rt, Box::new(disconnected));
    }
}
