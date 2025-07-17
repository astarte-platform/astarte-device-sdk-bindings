use astarte_device_sdk::{
    aggregate::AstarteObject,
    astarte_interfaces::{
        schema::MappingType, AggregationIndividual, Endpoint, InterfaceMapping, MappingPath, Schema,
    },
    builder::DeviceBuilder,
    chrono::{DateTime, Utc},
    prelude::{DeviceIntrospection, DynamicIntrospection},
    store::SqliteStore,
    transport::mqtt::{Mqtt, MqttConfig},
    types::AstarteData,
    Client, DeviceClient, DeviceConnection, DeviceEvent, EventLoop,
};
use pyo3::{
    create_exception,
    exceptions::{PyBaseException, PyValueError},
    prelude::*,
    types::PyDateTime,
    IntoPyObjectExt,
};
use std::{collections::HashMap, mem};

create_exception!(
    astarte_device_python_bindings,
    AstarteBaseSdkError,
    PyBaseException,
    "Base error of the astarte sdk"
);

create_exception!(
    astarte_device_python_bindings,
    AstarteSdkSendError,
    AstarteBaseSdkError,
    "Error while sending data to astarte"
);

create_exception!(
    astarte_device_python_bindings,
    AstarteSdkInitError,
    AstarteBaseSdkError,
    "Error while initializing the astarte device"
);

create_exception!(
    astarte_device_python_bindings,
    AstarteSdkHandleEventsError,
    AstarteBaseSdkError,
    "Error while handling the events of the sdk"
);

create_exception!(
    astarte_device_python_bindings,
    AstarteSdkReceiveEventsError,
    AstarteBaseSdkError,
    "Error while receiving events from astarte"
);

#[pyclass(frozen)]
struct AstarteDataWrapper(AstarteData);

impl AstarteDataWrapper {
    fn new(data: AstarteData) -> Self {
        Self(data)
    }
}

fn try_double_from_f64(val: f64) -> PyResult<astarte_device_sdk::types::Double> {
    astarte_device_sdk::types::Double::try_from(val).map_err(|e| {
        PyValueError::new_err(format!(
            "Error while extracting astarte double from f64 '{}'",
            e
        ))
    })
}

fn try_double_vec_from_f64_vec(val: Vec<f64>) -> PyResult<Vec<astarte_device_sdk::types::Double>> {
    val.into_iter().map(try_double_from_f64).collect()
}

fn f64_vec_from_double_vec(val: &Vec<astarte_device_sdk::types::Double>) -> &Vec<f64> {
    // SAFETY the [`astarte_device_sdk::types::Double`] type has a transparent representation
    // it has the same bit layout so a conversion of this type doesn't cause any problems
    unsafe { mem::transmute(val) }
}

// FIXME maybe i can replace the Bound objects with Py
fn try_astarte_data_from_any(
    mapping_type_hint: MappingType,
    any: &Bound<'_, PyAny>,
) -> PyResult<AstarteData> {
    let data = match mapping_type_hint {
        MappingType::Double => AstarteData::Double(try_double_from_f64(any.extract::<f64>()?)?),
        MappingType::Integer => AstarteData::Integer(any.extract::<i32>()?),
        MappingType::Boolean => AstarteData::Boolean(any.extract::<bool>()?),
        MappingType::LongInteger => AstarteData::LongInteger(any.extract::<i64>()?),
        MappingType::String => AstarteData::String(any.extract::<String>()?),
        MappingType::BinaryBlob => AstarteData::BinaryBlob(any.extract::<Vec<u8>>()?),
        MappingType::DateTime => AstarteData::DateTime(any.extract::<DateTime<Utc>>()?),
        MappingType::DoubleArray => {
            AstarteData::DoubleArray(try_double_vec_from_f64_vec(any.extract::<Vec<f64>>()?)?)
        }
        MappingType::IntegerArray => AstarteData::IntegerArray(any.extract::<Vec<i32>>()?),
        MappingType::BooleanArray => AstarteData::BooleanArray(any.extract::<Vec<bool>>()?),
        MappingType::LongIntegerArray => AstarteData::LongIntegerArray(any.extract::<Vec<i64>>()?),
        MappingType::StringArray => AstarteData::StringArray(any.extract::<Vec<String>>()?),
        MappingType::BinaryBlobArray => {
            AstarteData::BinaryBlobArray(any.extract::<Vec<Vec<u8>>>()?)
        }
        MappingType::DateTimeArray => {
            AstarteData::DateTimeArray(any.extract::<Vec<DateTime<Utc>>>()?)
        }
    };

    Ok(data)
}

fn astarte_data_to_pyany<'p>(
    py: Python<'p>,
    astarte_data: &AstarteData,
) -> PyResult<Bound<'p, PyAny>> {
    match &astarte_data {
        AstarteData::Double(i) => i.into_bound_py_any(py),
        AstarteData::Integer(i) => i.into_bound_py_any(py),
        AstarteData::Boolean(i) => i.into_bound_py_any(py),
        AstarteData::LongInteger(i) => i.into_bound_py_any(py),
        AstarteData::String(i) => i.into_bound_py_any(py),
        AstarteData::BinaryBlob(i) => i.into_bound_py_any(py),
        AstarteData::DateTime(i) => i.into_bound_py_any(py),
        AstarteData::DoubleArray(i) => f64_vec_from_double_vec(i).into_bound_py_any(py),
        AstarteData::IntegerArray(i) => i.into_bound_py_any(py),
        AstarteData::BooleanArray(i) => i.into_bound_py_any(py),
        AstarteData::LongIntegerArray(i) => i.into_bound_py_any(py),
        AstarteData::StringArray(i) => i.into_bound_py_any(py),
        AstarteData::BinaryBlobArray(i) => i.into_bound_py_any(py),
        AstarteData::DateTimeArray(i) => i.into_bound_py_any(py),
    }
}

#[pymethods]
impl AstarteDataWrapper {
    #[new]
    fn try_from_any(
        hint: &Bound<'_, AstarteMappingType>,
        any: &Bound<'_, PyAny>,
    ) -> PyResult<Self> {
        try_astarte_data_from_any(hint.get().ty, any).map(Self)
    }

    fn to_pyany<'p>(&self, py: Python<'p>) -> PyResult<Bound<'p, PyAny>> {
        astarte_data_to_pyany(py, &self.0)
    }
}

#[pyclass(frozen)]
struct AstarteObjectWrapper(AstarteObject);

impl AstarteObjectWrapper {
    fn new(inner: AstarteObject) -> Self {
        Self(inner)
    }
}

#[pymethods]
impl AstarteObjectWrapper {
    #[new]
    fn try_from_entries(
        hints: &Bound<'_, AstarteObjectMappings>,
        entries: HashMap<String, Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        let object = entries
            .iter()
            .map(|(path, data)| {
                let mapping_type_hint = hints.get().find(path).ok_or(PyValueError::new_err(
                    format!("no matching mapping for passed path '{}'", path),
                ))?;

                Ok((
                    path.to_string(),
                    try_astarte_data_from_any(mapping_type_hint, data)?,
                ))
            })
            .collect::<PyResult<AstarteObject>>()?;

        Ok(AstarteObjectWrapper::new(object))
    }

    fn to_pydict<'p>(&self, py: Python<'p>) -> PyResult<HashMap<&str, Bound<'p, PyAny>>> {
        self.0
            .iter()
            .map(|(endpoint, data)| Ok((endpoint.as_str(), astarte_data_to_pyany(py, data)?)))
            .collect()
    }
}

#[pyclass]
struct AstarteMqttDevice {
    client: Box<DeviceClient<Mqtt<SqliteStore>>>,
    connection: Option<Box<DeviceConnection<Mqtt<SqliteStore>>>>,
}

#[pyclass(frozen)]
struct AstarteMappingType {
    ty: MappingType,
}

impl AstarteMappingType {
    fn new(ty: MappingType) -> Self {
        Self { ty }
    }
}

struct AstarteObjectMapping {
    endpoint: Endpoint,
    ty: MappingType,
}

impl AstarteObjectMapping {
    fn new(endpoint: Endpoint, ty: MappingType) -> Self {
        Self { endpoint, ty }
    }
}

#[pyclass(frozen)]
struct AstarteObjectMappings {
    list: Vec<AstarteObjectMapping>,
}

impl AstarteObjectMappings {
    fn new(list: Vec<AstarteObjectMapping>) -> Self {
        Self { list }
    }

    fn find(&self, path: &str) -> Option<MappingType> {
        self.list
            .iter()
            .find(|mapping| mapping.endpoint.iter().last().is_some_and(|l| *l == path))
            .map(|m| m.ty)
    }
}

#[pyclass(frozen)]
struct AstarteEvent {
    event: DeviceEvent,
}

impl AstarteEvent {
    fn new(event: DeviceEvent) -> Self {
        Self { event }
    }
}

#[pymethods]
impl AstarteEvent {
    fn get_interface(&self) -> &str {
        &self.event.interface
    }

    fn get_path(&self) -> &str {
        &self.event.path
    }

    fn is_datastream_object(&self) -> bool {
        matches!(self.event.data, astarte_device_sdk::Value::Object { .. })
    }

    fn get_datastream_object<'p>(
        &self,
        py: Python<'p>,
    ) -> PyResult<(AstarteObjectWrapper, Bound<'p, PyDateTime>)> {
        let astarte_device_sdk::Value::Object { data, timestamp } = &self.event.data else {
            return Err(PyValueError::new_err("The event is not a dastream object"));
        };

        Ok((
            AstarteObjectWrapper::new(data.clone()),
            timestamp.into_pyobject(py)?,
        ))
    }

    fn is_datastream_individual(&self) -> bool {
        matches!(
            self.event.data,
            astarte_device_sdk::Value::Individual { .. }
        )
    }

    fn get_datastream_individual<'p>(
        &self,
        py: Python<'p>,
    ) -> PyResult<(AstarteDataWrapper, Bound<'p, PyDateTime>)> {
        let astarte_device_sdk::Value::Individual { data, timestamp } = &self.event.data else {
            return Err(PyValueError::new_err(
                "The event is not a dastream individual",
            ));
        };

        Ok((
            AstarteDataWrapper::new(data.clone()),
            timestamp.into_pyobject(py)?,
        ))
    }

    fn is_property_individual(&self) -> bool {
        matches!(self.event.data, astarte_device_sdk::Value::Property(..))
    }

    fn get_property_individual(&self) -> PyResult<Option<AstarteDataWrapper>> {
        let astarte_device_sdk::Value::Property(data) = &self.event.data else {
            return Err(PyValueError::new_err(
                "The self event is not a property individual",
            ));
        };

        Ok(data.clone().map(AstarteDataWrapper::new))
    }
}

impl AstarteMqttDevice {
    fn new(
        client: DeviceClient<Mqtt<SqliteStore>>,
        connection: DeviceConnection<Mqtt<SqliteStore>>,
    ) -> Self {
        Self {
            client: Box::new(client),
            connection: Some(Box::new(connection)),
        }
    }
}

#[pymethods]
impl AstarteMqttDevice {
    fn get_individual_mapping_type<'p>(
        &self,
        py: Python<'p>,
        interface_name: String,
        path: String,
    ) -> PyResult<Bound<'p, PyAny>> {
        let client = *self.client.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let mapping_path = MappingPath::try_from(path.as_str())
                .map_err(|e| PyValueError::new_err(format!("error while converting path {}", e)))?;

            let mapping = client
                .get_interface(&interface_name, |i| {
                    i.and_then(|i| {
                        let result =
                            if i.is_datastream_individual() {
                                // we keep the none result and this will result in a missing interface error
                                i.as_datastream_individual().map(|i| {
                                    i.mapping(&mapping_path).ok_or(PyValueError::new_err(
                                    "The passed interface does not contain the path specified",
                                ))
                                .map(|e| e.mapping_type())
                                })
                            } else if i.is_properties() {
                                // we keep the none result and this will result in a missing interface error
                                i.as_properties().map(|i| {
                                    i.mapping(&mapping_path).ok_or(PyValueError::new_err(
                                    "The passed interface does not contain the path specified",
                                ))
                                .map(|e| e.mapping_type())
                                })
                            } else {
                                Some(Err(PyValueError::new_err(
                                    "The passed interface refers to an object interface",
                                )))
                            };

                        result
                    })
                })
                .await
                .ok_or(PyValueError::new_err(
                    "The passed interface was not found in the device introspection",
                ))??;

            Ok(AstarteMappingType::new(mapping))
        })
    }

    // NOTE we do not check the object base path currently but it is checked when we actually perform the send
    fn get_object_mapping_types<'p>(
        &self,
        py: Python<'p>,
        interface_name: String,
    ) -> PyResult<Bound<'p, PyAny>> {
        let client = *self.client.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let mappings = client
                .get_interface(&interface_name, |i| {
                    i.and_then(|i| {
                        let result = if i.is_datastream_object() {
                            i.as_datastream_object().map(|o| {
                                let mappings: Vec<AstarteObjectMapping> = o
                                    .iter_mappings()
                                    .map(|m| {
                                        AstarteObjectMapping::new(
                                            m.endpoint().to_owned(),
                                            m.mapping_type(),
                                        )
                                    })
                                    .collect();

                                Ok(mappings)
                            })
                        } else {
                            Some(Err(PyValueError::new_err(
                                "The passed interface refers to an individual interface",
                            )))
                        };

                        result
                    })
                })
                .await
                .ok_or(PyValueError::new_err(
                    "The passed interface was not found in the device introspection",
                ))??;

            Ok(AstarteObjectMappings::new(mappings))
        })
    }

    // should be called in a loop it only receives one event
    fn receive_event<'p>(&mut self, py: Python<'p>) -> PyResult<Bound<'p, PyAny>> {
        let client = *self.client.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let event = client.recv().await.map_err(|e| {
                AstarteSdkReceiveEventsError::new_err(format!("error while receiving events {}", e))
            })?;

            Ok(AstarteEvent::new(event))
        })
    }

    fn handle_events<'p>(&mut self, py: Python<'p>) -> PyResult<Bound<'p, PyAny>> {
        let connection: Box<DeviceConnection<Mqtt<SqliteStore>>> =
            self.connection
                .take()
                .ok_or(AstarteSdkHandleEventsError::new_err(
                    "Handle events must be called once",
                ))?;

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            connection.handle_events().await.map_err(|e| {
                AstarteSdkHandleEventsError::new_err(format!(
                    "error while handling sdk events {}",
                    e
                ))
            })?;

            Ok(())
        })
    }

    fn send_individual<'p>(
        &self,
        py: Python<'p>,
        interface_name: String,
        path: String,
        data: Bound<'p, AstarteDataWrapper>,
    ) -> PyResult<Bound<'p, PyAny>> {
        let mut client: DeviceClient<Mqtt<SqliteStore>> = *self.client.clone();
        let data = data.get().0.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .send_individual(&interface_name, &path, data)
                .await
                .map_err(|e| {
                    AstarteSdkSendError::new_err(format!("send individual error {}", e))
                })?;

            Ok(())
        })
    }

    fn send_individual_with_timestamp<'p>(
        &self,
        py: Python<'p>,
        interface_name: String,
        path: String,
        data: Bound<'p, AstarteDataWrapper>,
        timestamp: DateTime<Utc>,
    ) -> PyResult<Bound<'p, PyAny>> {
        let mut client: DeviceClient<Mqtt<SqliteStore>> = *self.client.clone();
        let data = data.get().0.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .send_individual_with_timestamp(&interface_name, &path, data, timestamp)
                .await
                .map_err(|e| {
                    AstarteSdkSendError::new_err(format!("send individual error {}", e))
                })?;

            Ok(())
        })
    }

    fn send_object<'p>(
        &self,
        py: Python<'p>,
        interface_name: String,
        path: String,
        data: Bound<'p, AstarteObjectWrapper>,
    ) -> PyResult<Bound<'p, PyAny>> {
        let mut client: DeviceClient<Mqtt<SqliteStore>> = *self.client.clone();
        let data = data.get().0.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .send_object(&interface_name, &path, data)
                .await
                .map_err(|e| AstarteSdkSendError::new_err(format!("send object error {}", e)))?;

            Ok(())
        })
    }

    fn send_object_with_timestamp<'p>(
        &self,
        py: Python<'p>,
        interface_name: String,
        path: String,
        data: Bound<'p, AstarteObjectWrapper>,
        timestamp: DateTime<Utc>,
    ) -> PyResult<Bound<'p, PyAny>> {
        let mut client: DeviceClient<Mqtt<SqliteStore>> = *self.client.clone();
        let data = data.get().0.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .send_object_with_timestamp(&interface_name, &path, data, timestamp)
                .await
                .map_err(|e| AstarteSdkSendError::new_err(format!("send object error {}", e)))?;

            Ok(())
        })
    }

    fn set_property<'p>(
        &self,
        py: Python<'p>,
        interface_name: String,
        path: String,
        data: Bound<'p, AstarteDataWrapper>,
    ) -> PyResult<Bound<'p, PyAny>> {
        let mut client: DeviceClient<Mqtt<SqliteStore>> = *self.client.clone();
        let data = data.get().0.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .set_property(&interface_name, &path, data)
                .await
                .map_err(|e| AstarteSdkSendError::new_err(format!("set property error {}", e)))?;

            Ok(())
        })
    }

    fn unset_property<'p>(
        &self,
        py: Python<'p>,
        interface_name: String,
        path: String,
    ) -> PyResult<Bound<'p, PyAny>> {
        let mut client: DeviceClient<Mqtt<SqliteStore>> = *self.client.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .unset_property(&interface_name, &path)
                .await
                .map_err(|e| AstarteSdkSendError::new_err(format!("unset property error {}", e)))?;

            Ok(())
        })
    }

    fn add_interface<'p>(
        &self,
        py: Python<'p>,
        interface_json: String,
    ) -> PyResult<Bound<'p, PyAny>> {
        let mut client: DeviceClient<Mqtt<SqliteStore>> = *self.client.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let result = client
                .add_interface_from_str(&interface_json)
                .await
                .map_err(|e| {
                    AstarteSdkSendError::new_err(format!("error while adding interface {}", e))
                })?;

            Ok(result)
        })
    }

    fn remove_interface<'p>(
        &self,
        py: Python<'p>,
        interface_name: String,
    ) -> PyResult<Bound<'p, PyAny>> {
        let mut client: DeviceClient<Mqtt<SqliteStore>> = *self.client.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let result = client
                .remove_interface(&interface_name)
                .await
                .map_err(|e| {
                    AstarteSdkSendError::new_err(format!(
                        "error while removing the interface {}",
                        e
                    ))
                })?;

            Ok(result)
        })
    }
}

#[pyfunction]
fn init_device_async(
    py: Python,
    device_id: String,
    realm: String,
    credential_secret: String,
    pairing_url: String,
    persistency_dir: String,
    ignore_ssl_errors: bool,
    interfaces_directory: Option<String>,
) -> PyResult<Bound<PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        init_device_mqtt(
            device_id,
            realm,
            credential_secret,
            pairing_url,
            persistency_dir,
            ignore_ssl_errors,
            interfaces_directory,
        )
        .await
    })
}

//#[pyclass]
//struct MqttDeviceBuilder {
//    builder: DeviceBuilder<MqttConfig, SqliteStore>,
//}

//#[pymethods]
//impl MqttDeviceBuilder {
//    fn add_interface() {
//
//    }
//}

async fn init_device_mqtt(
    device_id: String,
    realm: String,
    credential_secret: String,
    pairing_url: String,
    persistency_dir: String,
    ignore_ssl_errors: bool,
    interfaces_directory: Option<String>,
) -> PyResult<AstarteMqttDevice> {
    let mut mqtt_config =
        MqttConfig::with_credential_secret(realm, device_id, credential_secret, pairing_url);

    if ignore_ssl_errors {
        mqtt_config.ignore_ssl_errors();
    }

    let mut builder = DeviceBuilder::new()
        .store_dir(persistency_dir)
        .await
        .map_err(|e| AstarteSdkInitError::new_err(format!("persistency dir error {}", e)))?;

    if let Some(interfaces_directory) = interfaces_directory {
        builder = builder
            .interface_directory(interfaces_directory)
            .map_err(|e| AstarteSdkInitError::new_err(format!("interface add error {}", e)))?;
    }

    let (client, connection) =
        builder.connection(mqtt_config).build().await.map_err(|e| {
            AstarteSdkInitError::new_err(format!("device first connect error {}", e))
        })?;

    let device = AstarteMqttDevice::new(client, connection);

    Ok(device)
}

/// A Python module implemented in Rust.
#[pymodule]
fn astarte_device_python_bindings(m: &pyo3::Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(init_device_async, m)?)?;
    m.add_class::<AstarteMqttDevice>()?;
    m.add_class::<AstarteDataWrapper>()?;
    m.add_class::<AstarteObjectWrapper>()?;
    Ok(())
}
