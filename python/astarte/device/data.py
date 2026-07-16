"""Data and Value representations for Astarte Device SDK bindings."""

from __future__ import annotations

import array
import datetime
import struct
from abc import ABC, abstractmethod
from collections.abc import Iterator, Sequence
from types import MappingProxyType
from typing import Union, overload

from cffi import FFI

from astarte.device._ffi import ffi, get_lib
from astarte.device.exceptions import InvalidNativeValueError


class CDataRefs:
    """Utility class to maintain references to native C memory to prevent premature GC."""

    def __init__(self, root: FFI.CData):
        self._root: FFI.CData = root
        self._keepalive: list[FFI.CData] = []

    def root(self) -> FFI.CData:
        return self._root

    def ref(self, data: FFI.CData) -> CDataRefs:
        self._keepalive.append(data)
        return self

    def ref_list(self, l: list[FFI.CData]) -> CDataRefs:
        self._keepalive.extend(l)
        return self

    def child(self, other: CDataRefs) -> CDataRefs:
        self._keepalive.append(other._root)
        self._keepalive.extend(other._keepalive)
        return self

    def __getitem__(self, index):
        return self._root[index]

    def set_item_value(self, index: int, item: CDataRefs) -> CDataRefs:
        self._root[index] = item._root[0]
        self.child(item)
        return self


class DeviceBinaryBlob:
    """Binary blob data representation for Astarte values."""

    def __init__(self, view: memoryview, ptr: FFI.CData | None = None):
        self._keepalive = ptr
        self._view = view

    @staticmethod
    def from_bytes(buf: bytes) -> DeviceBinaryBlob:
        view = memoryview(buf).cast("B").toreadonly()  # type: ignore[attr-defined]
        return DeviceBinaryBlob(view)

    @staticmethod
    def from_cdata(value: FFI.CData, event: FFI.CData | None = None) -> DeviceBinaryBlob:
        data = value.data_ptr
        length: int = value.size
        buffer = ffi.buffer(data, length)
        view = memoryview(buffer).cast("B").toreadonly()  # type: ignore[attr-defined]
        return DeviceBinaryBlob(view, event)

    def to_cdata(self) -> CDataRefs:
        buffer = ffi.from_buffer("uint8_t[]", self)
        native = ffi.new("CArray_u8 *")
        native.data_ptr = buffer
        native.size = len(self)
        return CDataRefs(native).ref(buffer)

    def __bytes__(self) -> bytes:
        return self._view.tobytes()

    def __len__(self) -> int:
        return len(self._view)

    def __getitem__(self, i):
        if isinstance(i, slice):
            return [self._view[j] for j in range(*i.indices(len(self)))]
        if i < 0 or i >= len(self):
            raise IndexError("Array index out of range")
        return self._view[i]

    def __iter__(self):
        for i in range(len(self)):
            yield self._view[i]

    def __buffer__(self, flags):
        return self._view.__buffer__(flags)


DeviceDataScalarType = Union[
    float,
    int,
    bool,
    str,
    DeviceBinaryBlob,
    datetime.datetime,
]

DeviceDataVectorType = Union[
    Sequence[float],
    Sequence[int],
    Sequence[bool],
    Sequence[str],
    Sequence[DeviceBinaryBlob],
    Sequence[datetime.datetime],
]


class DeviceData(ABC):
    """Abstract base class for DeviceData variants."""

    def as_value(self) -> DeviceDataScalarType | None:
        """Return the underlying immutable scalar value if this is a scalar."""
        return None

    def as_vector(self) -> DeviceDataVectorType | None:
        """Return the underlying vector sequence if this is a vector."""
        return None

    @abstractmethod
    def to_cdata(self) -> CDataRefs:
        """Return the native CData representation without copying."""
        pass

    @staticmethod
    def from_cdata(value: FFI.CData, event: FFI.CData | None = None) -> DeviceData:
        tag = value.tag
        lib = get_lib()

        if tag == lib.Double:
            return DeviceDataDouble.from_cdata(value)
        elif tag == lib.Integer:
            return DeviceDataInteger.from_cdata(value)
        elif tag == lib.Boolean:
            return DeviceDataBoolean.from_cdata(value)
        elif tag == lib.LongInteger:
            return DeviceDataLongInteger.from_cdata(value)
        elif tag == lib.String:
            return DeviceDataString.from_cdata(value)
        elif tag == lib.BinaryBlob:
            return DeviceDataBinaryBlob.from_cdata(value, event)
        elif tag == lib.DateTime:
            return DeviceDataDateTime.from_cdata(value)
        elif tag == lib.DoubleArray:
            return DeviceDataDoubleArray.from_cdata(value, event)
        elif tag == lib.IntegerArray:
            return DeviceDataIntegerArray.from_cdata(value, event)
        elif tag == lib.BooleanArray:
            return DeviceDataBooleanArray.from_cdata(value, event)
        elif tag == lib.LongIntegerArray:
            return DeviceDataLongIntegerArray.from_cdata(value, event)
        elif tag == lib.StringArray:
            return DeviceDataStringArray.from_cdata(value)
        elif tag == lib.BinaryBlobArray:
            return DeviceDataBinaryBlobArray.from_cdata(value, event)
        elif tag == lib.DateTimeArray:
            return DeviceDataDateTimeArray.from_cdata(value)
        else:
            raise InvalidNativeValueError(f"Unknown DeviceData tag: {tag}")


class DeviceDataDouble(DeviceData):
    def __init__(self, double_val: float):
        self._value = double_val

    def as_value(self) -> float:
        return self._value

    @staticmethod
    def from_cdata(value: FFI.CData, event: FFI.CData | None = None) -> DeviceDataDouble:
        return DeviceDataDouble(value.double_)

    def to_cdata(self) -> CDataRefs:
        native = ffi.new("NativeDeviceData *")
        native.tag = get_lib().Double
        native.double_ = self._value
        return CDataRefs(native)


class DeviceDataInteger(DeviceData):
    def __init__(self, int_val: int):
        self._value = int_val

    def as_value(self) -> int:
        return self._value

    @staticmethod
    def from_cdata(value: FFI.CData, event: FFI.CData | None = None) -> DeviceDataInteger:
        return DeviceDataInteger(value.integer)

    def to_cdata(self) -> CDataRefs:
        native = ffi.new("NativeDeviceData *")
        native.tag = get_lib().Integer
        native.integer = self._value
        return CDataRefs(native)


class DeviceDataBoolean(DeviceData):
    def __init__(self, bool_val: bool):
        self._value = bool_val

    def as_value(self) -> bool:
        return self._value

    @staticmethod
    def from_cdata(value: FFI.CData, event: FFI.CData | None = None) -> DeviceDataBoolean:
        return DeviceDataBoolean(value.boolean)

    def to_cdata(self) -> CDataRefs:
        native = ffi.new("NativeDeviceData *")
        native.tag = get_lib().Boolean
        native.boolean = self._value
        return CDataRefs(native)


class DeviceDataLongInteger(DeviceData):
    def __init__(self, long_int_val: int):
        self._value = long_int_val

    def as_value(self) -> int:
        return self._value

    @staticmethod
    def from_cdata(value: FFI.CData, event: FFI.CData | None = None) -> DeviceDataLongInteger:
        return DeviceDataLongInteger(value.long_integer)

    def to_cdata(self) -> CDataRefs:
        native = ffi.new("NativeDeviceData *")
        native.tag = get_lib().LongInteger
        native.long_integer = self._value
        return CDataRefs(native)


class DeviceDataString(DeviceData):
    def __init__(self, value: str):
        self._value = value

    def as_value(self) -> str:
        return self._value

    @staticmethod
    def from_cdata(value: FFI.CData, event: FFI.CData | None = None) -> DeviceDataString:
        string = ffi.string(value.string).decode()
        return DeviceDataString(string)

    def to_cdata(self) -> CDataRefs:
        string = ffi.new("char[]", self._value.encode())
        native = ffi.new("NativeDeviceData *")
        native.tag = get_lib().String
        native.string = string
        return CDataRefs(native).ref(string)


class DeviceDataBinaryBlob(DeviceData):
    def __init__(self, blob: DeviceBinaryBlob):
        self._value = blob

    def as_value(self) -> DeviceBinaryBlob:
        return self._value

    @staticmethod
    def from_cdata(value: FFI.CData, event: FFI.CData | None = None) -> DeviceDataBinaryBlob:
        blob = DeviceBinaryBlob.from_cdata(value.binary_blob, event)
        return DeviceDataBinaryBlob(blob)

    def to_cdata(self) -> CDataRefs:
        blob_cdata = self._value.to_cdata()
        native = ffi.new("NativeDeviceData *")
        native.tag = get_lib().BinaryBlob
        native.binary_blob = blob_cdata.root()[0]
        return CDataRefs(native).child(blob_cdata)


class DeviceDataDateTime(DeviceData):
    def __init__(self, value: datetime.datetime):
        self._value = value

    def as_value(self) -> datetime.datetime:
        return self._value

    @staticmethod
    def from_cdata(value: FFI.CData, event: FFI.CData | None = None) -> DeviceDataDateTime:
        dt = datetime.datetime.fromtimestamp(value.date_time / 1000.0)
        return DeviceDataDateTime(dt)

    def to_cdata(self) -> CDataRefs:
        native = ffi.new("NativeDeviceData *")
        native.tag = get_lib().DateTime
        native.date_time = int(self._value.timestamp() * 1000)
        return CDataRefs(native)


class DeviceDataDoubleArray(DeviceData, Sequence[float]):
    def __init__(self, view: memoryview[float], event: FFI.CData | None = None):
        if view.format != "d":
            raise ValueError("expected a view of type d")
        self._event = event
        self._view: memoryview[float] = view.toreadonly()  # type: ignore[attr-defined]

    def as_vector(self) -> Sequence[float]:
        return self

    @staticmethod
    def from_array(arr: array.array) -> DeviceDataDoubleArray:
        if arr.typecode != "d":
            raise ValueError("expected an array of type d")
        view: memoryview[float] = memoryview(arr)
        return DeviceDataDoubleArray(view)

    @staticmethod
    def from_cdata(value: FFI.CData, event: FFI.CData | None = None) -> DeviceDataDoubleArray:
        data_ptr = value.double_array.data_ptr
        length = value.double_array.size
        buffer = ffi.buffer(data_ptr, length * struct.calcsize("d"))
        view: memoryview[float] = memoryview(buffer).cast("d")
        return DeviceDataDoubleArray(view, event)

    def to_cdata(self) -> CDataRefs:
        buffer = ffi.from_buffer("double[]", self)
        native = ffi.new("NativeDeviceData *")
        native.tag = get_lib().DoubleArray
        native.double_array.data_ptr = buffer
        native.double_array.size = len(self)
        return CDataRefs(native).ref(buffer)

    def __len__(self) -> int:
        return len(self._view)

    @overload
    def __getitem__(self, i: int) -> float: ...

    @overload
    def __getitem__(self, i: slice) -> Sequence[float]: ...

    def __getitem__(self, i: int | slice) -> float | Sequence[float]:
        if isinstance(i, slice):
            return array.array("d", self._view[i])
        return self._view[i]

    def __buffer__(self, flags):
        return memoryview(self._view).__buffer__(flags)


class DeviceDataIntegerArray(DeviceData, Sequence[int]):
    def __init__(self, view: memoryview[int], event: FFI.CData | None = None):
        if view.format != "i":
            raise ValueError("expected a view of type i")
        self._event = event
        self._view: memoryview[int] = view.toreadonly()

    def as_vector(self) -> Sequence[int]:
        return self

    @staticmethod
    def from_array(arr: array.array) -> DeviceDataIntegerArray:
        if arr.typecode != "i":
            raise ValueError("expected an array of type i")
        view: memoryview[int] = memoryview(arr)
        return DeviceDataIntegerArray(view)

    @staticmethod
    def from_cdata(value: FFI.CData, event: FFI.CData | None = None) -> DeviceDataIntegerArray:
        data_ptr = value.integer_array.data_ptr
        length = value.integer_array.size
        buffer = ffi.buffer(data_ptr, length * struct.calcsize("i"))
        view: memoryview[int] = memoryview(buffer).cast("i")
        return DeviceDataIntegerArray(view, event)

    def to_cdata(self) -> CDataRefs:
        buffer = ffi.from_buffer("int32_t[]", self)
        native = ffi.new("NativeDeviceData *")
        native.tag = get_lib().IntegerArray
        native.integer_array.data_ptr = buffer
        native.integer_array.size = len(self)
        return CDataRefs(native).ref(buffer)

    def __len__(self) -> int:
        return len(self._view)

    @overload
    def __getitem__(self, i: int) -> int: ...

    @overload
    def __getitem__(self, i: slice) -> Sequence[int]: ...

    def __getitem__(self, i: int | slice) -> int | Sequence[int]:
        if isinstance(i, slice):
            return array.array("i", self._view[i])
        return self._view[i]

    def __buffer__(self, flags):
        return memoryview(self._view).__buffer__(flags)


class DeviceDataBooleanArray(DeviceData, Sequence[bool]):
    def __init__(self, view: memoryview[bool], event: FFI.CData | None = None):
        if view.format != "?":
            raise ValueError("expected a view of type ?")
        self._event = event
        self._view: memoryview[bool] = view.toreadonly()  # type: ignore[attr-defined]

    def as_vector(self) -> Sequence[bool]:
        return self

    @staticmethod
    def from_list(bools: list[bool]) -> DeviceDataBooleanArray:
        native_bools = ffi.new("bool[]", bools)
        buffer = ffi.buffer(native_bools)
        view: memoryview[bool] = memoryview(buffer).cast("?")
        return DeviceDataBooleanArray(view)

    @staticmethod
    def from_cdata(value: FFI.CData, event: FFI.CData | None = None) -> DeviceDataBooleanArray:
        data_ptr = value.boolean_array.data_ptr
        length = value.boolean_array.size
        buffer = ffi.buffer(data_ptr, length * struct.calcsize("?"))
        view: memoryview[bool] = memoryview(buffer).cast("?")
        return DeviceDataBooleanArray(view, event)

    def to_cdata(self) -> CDataRefs:
        buffer = ffi.from_buffer("bool[]", self)
        native = ffi.new("NativeDeviceData *")
        native.tag = get_lib().BooleanArray
        native.boolean_array.data_ptr = buffer
        native.boolean_array.size = len(self)
        return CDataRefs(native).ref(buffer)

    def __len__(self) -> int:
        return len(self._view)

    @overload
    def __getitem__(self, i: int) -> bool: ...

    @overload
    def __getitem__(self, i: slice) -> Sequence[bool]: ...

    def __getitem__(self, i: int | slice) -> bool | Sequence[bool]:
        if isinstance(i, slice):
            return list(self._view[i])
        return self._view[i]

    def __buffer__(self, flags):
        return memoryview(self._view).__buffer__(flags)


class DeviceDataLongIntegerArray(DeviceData, Sequence[int]):
    def __init__(self, view: memoryview[int], event: FFI.CData | None = None):
        if view.format != "q":
            raise ValueError("expected a view of type q")
        self._event = event
        self._view: memoryview[int] = view.toreadonly()

    def as_vector(self) -> Sequence[int]:
        return self

    @staticmethod
    def from_array(arr: array.array) -> DeviceDataLongIntegerArray:
        if arr.typecode != "q":
            raise ValueError("expected an array of type q")
        view: memoryview[int] = memoryview(arr)
        return DeviceDataLongIntegerArray(view)

    @staticmethod
    def from_cdata(value: FFI.CData, event: FFI.CData | None = None) -> DeviceDataLongIntegerArray:
        data_ptr = value.long_integer_array.data_ptr
        length = value.long_integer_array.size
        buffer = ffi.buffer(data_ptr, length * struct.calcsize("q"))
        view: memoryview[int] = memoryview(buffer).cast("q")
        return DeviceDataLongIntegerArray(view, event)

    def to_cdata(self) -> CDataRefs:
        buffer = ffi.from_buffer("int64_t[]", self)
        native = ffi.new("NativeDeviceData *")
        native.tag = get_lib().LongIntegerArray
        native.long_integer_array.data_ptr = buffer
        native.long_integer_array.size = len(self)
        return CDataRefs(native).ref(buffer)

    def __len__(self) -> int:
        return len(self._view)

    @overload
    def __getitem__(self, i: int) -> int: ...

    @overload
    def __getitem__(self, i: slice) -> Sequence[int]: ...

    def __getitem__(self, i: int | slice) -> int | Sequence[int]:
        if isinstance(i, slice):
            return array.array("q", self._view[i])
        return self._view[i]

    def __buffer__(self, flags):
        return memoryview(self._view).__buffer__(flags)


class DeviceDataStringArray(DeviceData, Sequence[str]):
    def __init__(self, strings: list[str]):
        self._strings = strings

    def as_vector(self) -> Sequence[str]:
        return self

    @staticmethod
    def from_cdata(value: FFI.CData, event: FFI.CData | None = None) -> DeviceDataStringArray:
        ptr = value.string_array.data
        length = value.string_array.size
        strings = [ffi.string(ptr[i]).decode() for i in range(length)]
        return DeviceDataStringArray(strings)

    def to_cdata(self) -> CDataRefs:
        cdata_list = [ffi.new("char[]", s.encode()) for s in self._strings]
        native_strings = ffi.new("char*[]", cdata_list)
        native = ffi.new("NativeDeviceData *")
        native.tag = get_lib().StringArray
        native.string_array.data = native_strings
        native.string_array.size = len(self)
        return CDataRefs(native).ref_list(cdata_list).ref(native_strings)

    def __len__(self) -> int:
        return len(self._strings)

    @overload
    def __getitem__(self, i: int) -> str: ...

    @overload
    def __getitem__(self, i: slice) -> Sequence[str]: ...

    def __getitem__(self, i: int | slice) -> str | Sequence[str]:
        return self._strings[i]

    def __iter__(self) -> Iterator[str]:
        return iter(self._strings)


class DeviceDataBinaryBlobArray(DeviceData, Sequence[DeviceBinaryBlob]):
    def __init__(self, blobs: list[DeviceBinaryBlob]):
        self._blobs = blobs

    def as_vector(self) -> Sequence[DeviceBinaryBlob]:
        return self

    @staticmethod
    def from_cdata(value: FFI.CData, event: FFI.CData | None = None) -> DeviceDataBinaryBlobArray:
        ptr = value.binary_blob_array.data_ptr
        length: int = value.binary_blob_array.size
        blobs = [DeviceBinaryBlob.from_cdata(ptr[i], event) for i in range(length)]
        return DeviceDataBinaryBlobArray(blobs)

    def to_cdata(self) -> CDataRefs:
        native_blobs = CDataRefs(ffi.new("struct CArray_u8[]", len(self._blobs)))
        for i, blob in enumerate(self._blobs):
            blob_cdata = blob.to_cdata()
            native_blobs.set_item_value(i, blob_cdata)

        native = ffi.new("NativeDeviceData *")
        native.tag = get_lib().BinaryBlobArray
        native.binary_blob_array.data_ptr = native_blobs.root()
        native.binary_blob_array.size = len(self._blobs)
        return CDataRefs(native).child(native_blobs)

    def __len__(self) -> int:
        return len(self._blobs)

    @overload
    def __getitem__(self, i: int) -> DeviceBinaryBlob: ...

    @overload
    def __getitem__(self, i: slice) -> Sequence[DeviceBinaryBlob]: ...

    def __getitem__(self, i: int | slice) -> DeviceBinaryBlob | Sequence[DeviceBinaryBlob]:
        return self._blobs[i]

    def __iter__(self) -> Iterator[DeviceBinaryBlob]:
        return iter(self._blobs)


class DeviceDataDateTimeArray(DeviceData, Sequence[datetime.datetime]):
    def __init__(self, datetimes: list[datetime.datetime]):
        self._datetimes = datetimes

    def as_vector(self) -> Sequence[datetime.datetime]:
        return self

    @staticmethod
    def from_cdata(value: FFI.CData, event: FFI.CData | None = None) -> DeviceDataDateTimeArray:
        data = value.date_time_array.data_ptr
        length: int = value.date_time_array.size
        datetimes = [datetime.datetime.fromtimestamp(data[i] / 1000.0) for i in range(length)]
        return DeviceDataDateTimeArray(datetimes)

    def to_cdata(self) -> CDataRefs:
        cdata_list = [int(d.timestamp() * 1000.0) for d in self._datetimes]
        native_datetimes = CDataRefs(ffi.new("NativeTimestamp[]", cdata_list))
        native = ffi.new("NativeDeviceData *")
        native.tag = get_lib().DateTimeArray
        native.date_time_array.data_ptr = native_datetimes.root()
        native.date_time_array.size = len(self)
        return CDataRefs(native).child(native_datetimes)

    def __len__(self) -> int:
        return len(self._datetimes)

    @overload
    def __getitem__(self, i: int) -> datetime.datetime: ...

    @overload
    def __getitem__(self, i: slice) -> Sequence[datetime.datetime]: ...

    def __getitem__(self, i: int | slice) -> datetime.datetime | Sequence[datetime.datetime]:
        return self._datetimes[i]

    def __iter__(self) -> Iterator[datetime.datetime]:
        return iter(self._datetimes)


class DeviceObject:
    """Represents a set of endpoints mapped to DeviceData values for aggregate interfaces."""

    def __init__(self, data: dict[str, DeviceData]):
        self.data = MappingProxyType(data)

    @staticmethod
    def from_cdata(entries: FFI.CData, event: FFI.CData | None = None) -> DeviceObject:
        result_map = {}
        length = entries.size
        for i in range(length):
            c_entry = entries.data_ptr[i]
            path = ffi.string(c_entry.path).decode()
            value = DeviceData.from_cdata(c_entry.value, event)
            result_map[path] = value
        return DeviceObject(result_map)

    def to_cdata(self) -> CDataRefs:
        data_ptr = CDataRefs(ffi.new("NativeObjectEntry[]", len(self.data)))
        for i, (path, data) in enumerate(self.data.items()):
            c_path = ffi.new("char[]", path.encode())
            c_data = data.to_cdata()
            entry = ffi.new("NativeObjectEntry *")
            entry.path = c_path
            entry.value = c_data[0]
            data_ptr.set_item_value(i, CDataRefs(entry).ref(c_path).child(c_data))

        obj = ffi.new("CArray_NativeObjectEntry *")
        obj.data_ptr = data_ptr.root()
        obj.size = len(self.data)
        return CDataRefs(obj).child(data_ptr)


class DeviceValue(ABC):
    """Abstract base class for DeviceValue variants."""

    @staticmethod
    def from_cdata(value: FFI.CData, keepalive: FFI.CData | None = None) -> DeviceValue:
        tag = value.tag
        lib = get_lib()

        if tag == lib.Individual:
            return DeviceValueIndividual.from_cdata(value, keepalive)
        elif tag == lib.Object:
            return DeviceValueObject.from_cdata(value, keepalive)
        elif tag == lib.PropertySet:
            return DeviceValuePropertySet.from_cdata(value, keepalive)
        elif tag == lib.PropertyUnset:
            return DeviceValuePropertyUnset.from_cdata(value)
        else:
            raise InvalidNativeValueError(f"Unknown DeviceValue tag: {tag}")


class DeviceValueIndividual(DeviceValue):
    def __init__(self, data: DeviceData, timestamp: datetime.datetime):
        self._data = data
        self._timestamp = timestamp

    @staticmethod
    def from_cdata(value: FFI.CData, keepalive: FFI.CData | None = None) -> DeviceValueIndividual:
        data = DeviceData.from_cdata(value.individual.data, keepalive)
        timestamp = datetime.datetime.fromtimestamp(value.individual.timestamp / 1000.0)
        return DeviceValueIndividual(data, timestamp)

    def to_cdata(self) -> CDataRefs:
        value = ffi.new("NativeValue *")
        data = self._data.to_cdata()
        timestamp = int(self._timestamp.timestamp() * 1000)

        individual_body = ffi.new("Individual_Body")
        individual_body.data = data[0]
        individual_body.timestamp = timestamp

        value.tag = get_lib().Individual
        value.individual = individual_body
        return CDataRefs(value).ref(individual_body).child(data)

    @property
    def data(self) -> DeviceData:
        return self._data

    @property
    def timestamp(self) -> datetime.datetime:
        return self._timestamp


class DeviceValueObject(DeviceValue):
    def __init__(self, data: DeviceObject, timestamp: datetime.datetime):
        self._data = data
        self._timestamp = timestamp

    @staticmethod
    def from_cdata(value: FFI.CData, keepalive: FFI.CData | None = None) -> DeviceValueObject:
        obj = DeviceObject.from_cdata(value.object.data, keepalive)
        timestamp = datetime.datetime.fromtimestamp(value.object.timestamp / 1000.0)
        return DeviceValueObject(obj, timestamp)

    def to_cdata(self) -> CDataRefs:
        value = ffi.new("NativeValue *")
        data = self._data.to_cdata()
        timestamp = int(self._timestamp.timestamp() * 1000)

        object_body = ffi.new("Object_Body")
        object_body.data = data[0]
        object_body.timestamp = timestamp

        value.tag = get_lib().Object
        value.object = object_body
        return CDataRefs(value).ref(object_body).child(data)

    @property
    def data(self) -> MappingProxyType[str, DeviceData]:
        return self._data.data

    @property
    def timestamp(self) -> datetime.datetime:
        return self._timestamp


class DeviceValuePropertySet(DeviceValue):
    def __init__(self, property_val: DeviceData):
        self._property = property_val

    @staticmethod
    def from_cdata(value: FFI.CData, keepalive: FFI.CData | None = None) -> DeviceValuePropertySet:
        property_val = DeviceData.from_cdata(value.property_set, keepalive)
        return DeviceValuePropertySet(property_val)

    def to_cdata(self) -> CDataRefs:
        value = ffi.new("NativeValue *")
        data = self._property.to_cdata()
        value.tag = get_lib().PropertySet
        value.property_set = data[0]
        return CDataRefs(value).child(data)

    @property
    def property(self) -> DeviceData:
        return self._property


class DeviceValuePropertyUnset(DeviceValue):
    def __init__(self):
        pass

    @staticmethod
    def from_cdata(value: FFI.CData, keepalive: FFI.CData | None = None) -> DeviceValuePropertyUnset:
        return DeviceValuePropertyUnset()

    def to_cdata(self) -> CDataRefs:
        value = ffi.new("NativeValue *")
        value.tag = get_lib().PropertyUnset
        return CDataRefs(value)


class DeviceProperty:
    """Property value returned from an Astarte device."""

    def __init__(self, ok_data: FFI.CData):
        lib = get_lib()
        if ok_data.tag == lib.Some_NativeDeviceData:
            keepalive = ffi.gc(ffi.addressof(ok_data), lib.device_handle_free_get_property)
            self._data = DeviceData.from_cdata(ok_data.some, keepalive)
        elif ok_data.tag == lib.None_NativeDeviceData:
            self._data = None
        else:
            raise InvalidNativeValueError("Unexpected property native tag")

    def value(self) -> DeviceData | None:
        return self._data


class DeviceEvent:
    """Event received from Astarte device."""

    def __init__(self, ok_data: FFI.CData):
        lib = get_lib()
        self._ptr = ffi.gc(ffi.addressof(ok_data), lib.device_handle_free_device_event)
        self._interface = ffi.string(self._ptr.interface).decode()
        self._path = ffi.string(self._ptr.path).decode()
        self._data = DeviceValue.from_cdata(self._ptr.data, self._ptr)

    @property
    def interface(self) -> str:
        return self._interface

    @property
    def path(self) -> str:
        return self._path

    @property
    def data(self) -> DeviceValue:
        return self._data
