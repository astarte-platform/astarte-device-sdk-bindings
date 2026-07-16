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

use std::ffi::{CStr, CString, c_char};

use astarte_device_sdk::{
    AstarteData, DeviceEvent, Value,
    aggregate::AstarteObject,
    chrono::{DateTime, Utc},
    types::Double,
};
use ffi_convert::{AsRust, AsRustError, CArray, CDrop, CReprOf, CStringArray};

use crate::NativeOption;

// NOTE only milliseconds precision
/// C-compatible timestamp wrapper holding milliseconds since UNIX epoch
#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct NativeTimestamp(i64);

impl AsRust<DateTime<Utc>> for NativeTimestamp {
    fn as_rust(&self) -> Result<DateTime<Utc>, ffi_convert::AsRustError> {
        DateTime::from_timestamp_millis(self.0)
            .ok_or(AsRustError::Other("can't convert timestamp".into()))
    }
}

impl CDrop for NativeTimestamp {
    fn do_drop(&mut self) -> Result<(), ffi_convert::CDropError> {
        Ok(())
    }
}

impl CReprOf<DateTime<Utc>> for NativeTimestamp {
    fn c_repr_of(input: DateTime<Utc>) -> Result<Self, ffi_convert::CReprOfError> {
        Ok(Self(input.timestamp_millis()))
    }
}

/// C-compatible representation for sending individual datastream events
#[repr(C)]
#[derive(AsRust, CReprOf, CDrop, Debug)]
#[target_type(IndividualSend)]
pub struct NativeIndividualSend {
    /// C string pointer to target interface name
    pub interface: *const c_char,
    /// C string pointer to target path
    pub path: *const c_char,
    /// Native device data value to send
    pub data: NativeDeviceData,
    /// Optional timestamp for the data sample
    pub timestamp: NativeOption<NativeTimestamp>,
}

/// Data payload for sending individual datastream events
#[derive(Clone, Debug)]
pub struct IndividualSend {
    /// Target interface name
    pub interface: String,
    /// Target path
    pub path: String,
    /// Data value to send
    pub data: AstarteData,
    /// Optional timestamp for the data sample
    pub timestamp: Option<DateTime<Utc>>,
}

/// C-compatible representation for setting a property value
#[repr(C)]
#[derive(AsRust, CReprOf, CDrop, Debug)]
#[target_type(SetProperty)]
pub struct NativeSetProperty {
    /// C string pointer to target interface name
    pub interface: *const c_char,
    /// C string pointer to property path
    pub path: *const c_char,
    /// Native device data value for property
    pub data: NativeDeviceData,
}

/// Data payload for setting a property value
#[derive(Clone, Debug)]
pub struct SetProperty {
    /// Target interface name
    pub interface: String,
    /// Property path
    pub path: String,
    /// Data value for property
    pub data: AstarteData,
}

/// C-compatible property identifier
#[repr(C)]
#[derive(AsRust, CReprOf, CDrop, Debug)]
#[target_type(PropertyIdentifier)]
pub struct NativePropertyIdentifier {
    /// C string pointer to target interface name
    pub interface: *const c_char,
    /// C string pointer to property path
    pub path: *const c_char,
}

/// Property identifier
#[derive(Clone, Debug)]
pub struct PropertyIdentifier {
    /// Target interface name
    pub interface: String,
    /// Property path
    pub path: String,
}

/// C-compatible representation for sending object datastream events
#[repr(C)]
#[derive(CDrop, Debug)]
pub struct NativeObjectSend {
    /// C string pointer to target interface name
    pub interface: *const c_char,
    /// C string pointer to object path
    pub path: *const c_char,
    /// Array of key-value object entries
    pub data: CArray<NativeObjectEntry>,
    /// Optional timestamp for the object data sample
    pub timestamp: NativeOption<NativeTimestamp>,
}

impl CReprOf<ObjectSend> for NativeObjectSend {
    fn c_repr_of(input: ObjectSend) -> Result<Self, ffi_convert::CReprOfError> {
        let interface = CString::c_repr_of(input.interface)?.into_raw();
        let path = CString::c_repr_of(input.path)?.into_raw();
        let key_values: Vec<(String, AstarteData)> = input.data.into_key_values().collect();
        let data = CArray::c_repr_of(key_values)?;
        let timestamp = NativeOption::c_repr_of(input.timestamp)?;

        Ok(Self {
            interface,
            path,
            data,
            timestamp,
        })
    }
}

impl AsRust<ObjectSend> for NativeObjectSend {
    fn as_rust(&self) -> Result<ObjectSend, AsRustError> {
        use ffi_convert::RawBorrow;

        let interface = unsafe { CStr::raw_borrow(self.interface) }?.as_rust()?;
        let path = unsafe { CStr::raw_borrow(self.path) }?.as_rust()?;
        let data = AstarteObject::from_iter(self.data.as_rust()?);
        let timestamp = self.timestamp.as_rust()?;

        Ok(ObjectSend {
            interface,
            path,
            data,
            timestamp,
        })
    }
}

/// Data payload for sending object datastream events
#[derive(Clone, Debug)]
pub struct ObjectSend {
    /// Target interface name
    pub interface: String,
    /// Object path
    pub path: String,
    /// Object data aggregate containing key-value pairs
    pub data: AstarteObject,
    /// Optional timestamp for the object data sample
    pub timestamp: Option<DateTime<Utc>>,
}

/*
 * NOTE by using ffi_convert you should copy data when converting to rust and to ffi compatible structs
 * since we don't want that for events data (it could be large) we'll also have borrowed struct that won't be dropped by us but will be
 * handled by the calling language
 */

/// C-compatible device event received from Astarte
#[repr(C)]
#[derive(CReprOf, CDrop, AsRust, Debug)]
#[target_type(DeviceEvent)]
pub struct NativeDeviceEvent {
    /// C string pointer to interface name
    pub interface: *const c_char,
    /// C string pointer to path
    pub path: *const c_char,
    /// Event value payload
    pub data: NativeValue,
}

/// C-compatible event payload value
#[repr(C)]
#[derive(CDrop, Debug)]
pub enum NativeValue {
    /// Individual datastream value variant
    Individual {
        /// Native data payload
        data: NativeDeviceData,
        /// Timestamp in milliseconds since UNIX epoch
        timestamp: i64,
    },
    /// Object datastream value variant
    Object {
        /// Array of key-value object entries
        data: CArray<NativeObjectEntry>,
        /// Timestamp in milliseconds since UNIX epoch
        timestamp: i64,
    },
    /// Property set value variant with payload
    PropertySet(NativeDeviceData),
    /// Property unset variant
    PropertyUnset,
}

impl CReprOf<Value> for NativeValue {
    fn c_repr_of(input: Value) -> Result<Self, ffi_convert::CReprOfError> {
        let native = match input {
            Value::Individual { data, timestamp } => Self::Individual {
                data: NativeDeviceData::c_repr_of(data)?,
                timestamp: timestamp.timestamp_millis(),
            },
            Value::Object { data, timestamp } => Self::Object {
                data: CArray::c_repr_of(data.into_key_values().collect())?,
                timestamp: timestamp.timestamp_millis(),
            },
            Value::Property(Some(data)) => Self::PropertySet(NativeDeviceData::c_repr_of(data)?),
            Value::Property(None) => Self::PropertyUnset,
        };

        Ok(native)
    }
}

impl AsRust<Value> for NativeValue {
    fn as_rust(&self) -> Result<Value, AsRustError> {
        let val = match self {
            Self::Individual { data, timestamp } => {
                let data = data.as_rust()?;
                let timestamp = DateTime::from_timestamp_millis(*timestamp)
                    .ok_or_else(|| AsRustError::Other("can't convert timestamp".into()))?;
                Value::Individual { data, timestamp }
            }
            Self::Object { data, timestamp } => {
                let key_values: Vec<(String, AstarteData)> = data.as_rust()?;
                let data = AstarteObject::from_iter(key_values);
                let timestamp = DateTime::from_timestamp_millis(*timestamp)
                    .ok_or_else(|| AsRustError::Other("can't convert timestamp".into()))?;
                Value::Object { data, timestamp }
            }
            Self::PropertySet(data) => {
                let data = data.as_rust()?;
                Value::Property(Some(data))
            }
            Self::PropertyUnset => Value::Property(None),
        };

        Ok(val)
    }
}

/// C-compatible single key-value entry within an object datastream
#[repr(C)]
#[derive(CDrop, Debug)]
pub struct NativeObjectEntry {
    path: *const c_char,
    value: NativeDeviceData,
}

impl CReprOf<(String, AstarteData)> for NativeObjectEntry {
    fn c_repr_of((path, data): (String, AstarteData)) -> Result<Self, ffi_convert::CReprOfError> {
        let path = CString::c_repr_of(path)?.into_raw();
        let value = NativeDeviceData::c_repr_of(data)?;

        Ok(Self { path, value })
    }
}

impl AsRust<(String, AstarteData)> for NativeObjectEntry {
    fn as_rust(&self) -> Result<(String, AstarteData), AsRustError> {
        use ffi_convert::RawBorrow;

        let path = unsafe { CStr::raw_borrow(self.path) }?.as_rust()?;
        let value = self.value.as_rust()?;

        Ok((path, value))
    }
}

/// C-compatible representation of Astarte data types
#[repr(C)]
#[derive(CDrop, Debug)]
pub enum NativeDeviceData {
    /// Double-precision floating point number
    Double(f64),
    /// 32-bit signed integer
    Integer(i32),
    /// Boolean value
    Boolean(bool),
    /// 64-bit signed integer
    LongInteger(i64),
    /// C string pointer
    String(*const c_char),
    /// Binary blob byte array
    BinaryBlob(CArray<u8>),
    /// Timestamp value
    DateTime(NativeTimestamp),
    /// Array of double-precision floating point numbers
    DoubleArray(CArray<f64>),
    /// Array of 32-bit signed integers
    IntegerArray(CArray<i32>),
    /// Array of boolean values
    BooleanArray(CArray<bool>),
    /// Array of 64-bit signed integers
    LongIntegerArray(CArray<i64>),
    /// Array of C strings
    StringArray(CStringArray),
    /// Array of binary blobs
    BinaryBlobArray(CArray<CArray<u8>>),
    /// Array of timestamps
    DateTimeArray(CArray<NativeTimestamp>),
}

impl AsRust<AstarteData> for NativeDeviceData {
    fn as_rust(&self) -> Result<AstarteData, AsRustError> {
        let native = match self {
            NativeDeviceData::Double(double) => AstarteData::Double(
                Double::try_from(*double)
                    .map_err(|_| AsRustError::Other("invalid double value".into()))?,
            ),
            NativeDeviceData::Integer(integer) => AstarteData::Integer(*integer),
            NativeDeviceData::Boolean(boolean) => AstarteData::Boolean(*boolean),
            NativeDeviceData::LongInteger(long_integer) => AstarteData::LongInteger(*long_integer),
            NativeDeviceData::String(string) => {
                AstarteData::String(unsafe { CStr::from_ptr(*string) }.as_rust()?)
            }
            NativeDeviceData::BinaryBlob(blob) => AstarteData::BinaryBlob(blob.as_rust()?),
            NativeDeviceData::DateTime(date_time) => AstarteData::DateTime(date_time.as_rust()?),
            NativeDeviceData::DoubleArray(doubles) => {
                let vec: Vec<f64> = doubles.as_rust()?;
                let doubles = vec
                    .into_iter()
                    .map(Double::try_from)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| AsRustError::Other("invalid double value".into()))?;

                AstarteData::DoubleArray(doubles)
            }
            NativeDeviceData::IntegerArray(items) => AstarteData::IntegerArray(items.as_rust()?),
            NativeDeviceData::BooleanArray(items) => AstarteData::BooleanArray(items.as_rust()?),
            NativeDeviceData::LongIntegerArray(items) => {
                AstarteData::LongIntegerArray(items.as_rust()?)
            }
            NativeDeviceData::StringArray(items) => AstarteData::StringArray(items.as_rust()?),
            NativeDeviceData::BinaryBlobArray(items) => {
                AstarteData::BinaryBlobArray(items.as_rust()?)
            }
            NativeDeviceData::DateTimeArray(items) => AstarteData::DateTimeArray(items.as_rust()?),
        };

        Ok(native)
    }
}

impl CReprOf<AstarteData> for NativeDeviceData {
    fn c_repr_of(input: AstarteData) -> Result<Self, ffi_convert::CReprOfError> {
        let native = match input {
            AstarteData::Double(double) => Self::Double(f64::from(double)),
            AstarteData::Integer(integer) => Self::Integer(integer),
            AstarteData::Boolean(boolean) => Self::Boolean(boolean),
            AstarteData::LongInteger(long_integer) => Self::LongInteger(long_integer),
            AstarteData::String(string) => Self::String(CString::c_repr_of(string)?.into_raw()),
            AstarteData::BinaryBlob(items) => Self::BinaryBlob(CArray::c_repr_of(items)?),
            AstarteData::DateTime(date_time) => {
                Self::DateTime(NativeTimestamp::c_repr_of(date_time)?)
            }
            AstarteData::DoubleArray(doubles) => Self::DoubleArray(CArray::c_repr_of(
                doubles.into_iter().map(f64::from).collect(),
            )?),
            AstarteData::IntegerArray(items) => Self::IntegerArray(CArray::c_repr_of(items)?),
            AstarteData::BooleanArray(items) => Self::BooleanArray(CArray::c_repr_of(items)?),
            AstarteData::LongIntegerArray(items) => {
                Self::LongIntegerArray(CArray::c_repr_of(items)?)
            }
            AstarteData::StringArray(items) => Self::StringArray(CStringArray::c_repr_of(items)?),
            AstarteData::BinaryBlobArray(items) => Self::BinaryBlobArray(CArray::c_repr_of(items)?),
            AstarteData::DateTimeArray(date_times) => {
                Self::DateTimeArray(CArray::c_repr_of(date_times)?)
            }
        };

        Ok(native)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use astarte_device_sdk::{
        AstarteData, Value,
        aggregate::AstarteObject,
        chrono::{DateTime, Utc},
        types::Double,
    };
    use ffi_convert::{AsRust, CReprOf};

    /// Compare two DateTime values ignoring sub-millisecond precision
    fn assert_date_time_eq(expected: &DateTime<Utc>, actual: &DateTime<Utc>) {
        assert_eq!(
            expected.timestamp_millis(),
            actual.timestamp_millis(),
            "DateTime values differ beyond millisecond precision: expected={:?}, actual={:?}",
            expected,
            actual
        );
    }

    /// Compare two DateTimeArray values ignoring sub-millisecond precision
    fn assert_date_time_array_eq(expected: &[DateTime<Utc>], actual: &[DateTime<Utc>]) {
        assert_eq!(
            expected.len(),
            actual.len(),
            "DateTimeArray lengths differ: expected={}, actual={}",
            expected.len(),
            actual.len()
        );
        for (i, (exp, act)) in expected.iter().zip(actual.iter()).enumerate() {
            assert_eq!(
                exp.timestamp_millis(),
                act.timestamp_millis(),
                "DateTimeArray element {} differs beyond millisecond precision: expected={:?}, actual={:?}",
                i,
                exp,
                act
            );
        }
    }

    fn make_astarte_data_variants() -> Box<[AstarteData]> {
        Box::new([
            AstarteData::Double(Double::try_from(std::f64::consts::PI).unwrap()),
            AstarteData::Integer(42),
            AstarteData::Boolean(true),
            AstarteData::LongInteger(123_456_789_012_345),
            AstarteData::String("hello".to_string()),
            AstarteData::BinaryBlob(vec![0u8, 1, 2, 255]),
            AstarteData::DateTime(Utc::now()),
            AstarteData::DoubleArray(vec![
                Double::try_from(1.0).unwrap(),
                Double::try_from(2.5).unwrap(),
                Double::try_from(std::f64::consts::PI).unwrap(),
            ]),
            AstarteData::IntegerArray(vec![1, 2, 3]),
            AstarteData::BooleanArray(vec![true, false, true]),
            AstarteData::LongIntegerArray(vec![10i64, 20, 30]),
            AstarteData::StringArray(vec!["a".to_string(), "b".to_string()]),
            AstarteData::BinaryBlobArray(vec![vec![1u8, 2], vec![3, 4]]),
            AstarteData::DateTimeArray(vec![Utc::now()]),
        ])
    }

    #[test]
    fn test_native_timestamp_roundtrip() {
        let dt = Utc::now();
        let native = NativeTimestamp::c_repr_of(dt).unwrap();
        let dt2 = native.as_rust().unwrap();
        assert_eq!(dt.timestamp_millis(), dt2.timestamp_millis());
    }

    #[test]
    fn test_native_device_data_roundtrip() {
        for data in make_astarte_data_variants() {
            let native = NativeDeviceData::c_repr_of(data.clone()).unwrap();
            let data2 = native.as_rust().unwrap();
            match (&data, &data2) {
                (AstarteData::DateTime(exp), AstarteData::DateTime(act)) => {
                    assert_date_time_eq(exp, act);
                }
                (AstarteData::DateTimeArray(exp), AstarteData::DateTimeArray(act)) => {
                    assert_date_time_array_eq(exp, act);
                }
                (exp, act) => {
                    assert_eq!(exp, act);
                }
            }
        }
    }

    #[test]
    fn test_native_value_roundtrip() {
        let ts = Utc::now();

        let individual = Value::Individual {
            data: AstarteData::Integer(42),
            timestamp: ts,
        };
        let native = NativeValue::c_repr_of(individual.clone()).unwrap();
        let val = native.as_rust().unwrap();
        match (&individual, &val) {
            (
                Value::Individual {
                    data: exp_data,
                    timestamp: exp_ts,
                },
                Value::Individual {
                    data: act_data,
                    timestamp: act_ts,
                },
            ) => {
                assert_eq!(exp_data, act_data);
                assert_date_time_eq(exp_ts, act_ts);
            }
            _ => panic!("expected Individual variant"),
        }

        let obj_data = AstarteObject::from_iter([("key".to_string(), AstarteData::Boolean(true))]);
        let object = Value::Object {
            data: obj_data,
            timestamp: ts,
        };
        let native = NativeValue::c_repr_of(object.clone()).unwrap();
        let val = native.as_rust().unwrap();
        match (&object, &val) {
            (
                Value::Object {
                    data: exp_data,
                    timestamp: exp_ts,
                },
                Value::Object {
                    data: act_data,
                    timestamp: act_ts,
                },
            ) => {
                assert_eq!(exp_data, act_data);
                assert_date_time_eq(exp_ts, act_ts);
            }
            _ => panic!("expected Object variant"),
        }

        let prop_some = Value::Property(Some(AstarteData::String("x".to_string())));
        let native = NativeValue::c_repr_of(prop_some.clone()).unwrap();
        let val = native.as_rust().unwrap();
        assert_eq!(val, prop_some);

        let prop_none = Value::Property(None);
        let native = NativeValue::c_repr_of(prop_none.clone()).unwrap();
        let val = native.as_rust().unwrap();
        assert_eq!(val, prop_none);
    }

    #[test]
    fn test_native_object_entry_roundtrip() {
        let path = "test/path".to_string();
        let data = AstarteData::Integer(7);
        let native = NativeObjectEntry::c_repr_of((path.clone(), data.clone())).unwrap();
        let (path2, data2) = native.as_rust().unwrap();
        assert_eq!(path, path2);
        assert_eq!(data, data2);
    }

    #[test]
    fn test_native_individual_send_roundtrip() {
        let data = AstarteData::Integer(99);
        let ts = Utc::now();
        let send = IndividualSend {
            interface: "test.interface".to_string(),
            path: "/path".to_string(),
            data,
            timestamp: Some(ts),
        };
        let native = NativeIndividualSend::c_repr_of(send.clone()).unwrap();
        let send2 = native.as_rust().unwrap();
        assert_eq!(send.interface, send2.interface);
        assert_eq!(send.path, send2.path);
        assert_eq!(send.data, send2.data);
        if let (Some(exp), Some(act)) = (send.timestamp, send2.timestamp) {
            assert_date_time_eq(&exp, &act);
        } else {
            assert_eq!(send.timestamp, send2.timestamp);
        }
    }

    #[test]
    fn test_native_set_property_roundtrip() {
        let data = AstarteData::String("val".to_string());
        let set = SetProperty {
            interface: "test.interface".to_string(),
            path: "/prop".to_string(),
            data,
        };
        let native = NativeSetProperty::c_repr_of(set.clone()).unwrap();
        let set2 = native.as_rust().unwrap();
        assert_eq!(set.interface, set2.interface);
        assert_eq!(set.path, set2.path);
        assert_eq!(set.data, set2.data);
    }

    #[test]
    fn test_native_property_identifier_roundtrip() {
        let id = PropertyIdentifier {
            interface: "test.interface".to_string(),
            path: "/path".to_string(),
        };
        let native = NativePropertyIdentifier::c_repr_of(id.clone()).unwrap();
        let id2 = native.as_rust().unwrap();
        assert_eq!(id.interface, id2.interface);
        assert_eq!(id.path, id2.path);
    }

    #[test]
    fn test_native_object_send_roundtrip() {
        let data = AstarteObject::from_iter([
            ("key1".to_string(), AstarteData::Integer(1)),
            ("key2".to_string(), AstarteData::Boolean(false)),
        ]);
        let ts = Utc::now();
        let send = ObjectSend {
            interface: "test.obj".to_string(),
            path: "/obj".to_string(),
            data,
            timestamp: Some(ts),
        };
        let native = NativeObjectSend::c_repr_of(send.clone()).unwrap();
        let send2 = native.as_rust().unwrap();
        assert_eq!(send.interface, send2.interface);
        assert_eq!(send.path, send2.path);
        if let (Some(exp), Some(act)) = (send.timestamp, send2.timestamp) {
            assert_date_time_eq(&exp, &act);
        } else {
            assert_eq!(send.timestamp, send2.timestamp);
        }
        let key_values: Vec<_> = send.data.into_key_values().collect();
        let key_values2: Vec<_> = send2.data.into_key_values().collect();
        assert_eq!(key_values, key_values2);
    }
}
