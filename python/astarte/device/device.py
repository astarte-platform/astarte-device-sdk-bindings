"""Main Device client class for Astarte Device SDK bindings."""

from __future__ import annotations

import asyncio
import datetime
from typing import Any

from cffi import FFI

from astarte.device._ffi import ffi, get_lib
from astarte.device.callbacks import (
    BaseFutureData,
    ConnectFutureData,
    DisconnectFutureData,
    GetPropertyFutureData,
    HandleEventsFutureData,
    ReceiveFutureData,
    SendFutureData,
    connect_cbk,
    disconnect_cbk,
    get_property_cbk,
    loop_cbk,
    receive_cbk,
    send_cbk,
)
from astarte.device.config import DeviceConfig
from astarte.device.data import DeviceData, DeviceEvent, DeviceObject, DeviceProperty


class Device:
    """Python wrapper for an Astarte device client."""

    def __init__(self):
        self._loop_future: asyncio.Future[None] | None = None
        self._handles: set[FFI.CData] = set()
        self._ptr = get_lib().device_handle_init()

    def __del__(self):
        if len(self._handles) != 0:
            print("Warning: some CFFI handles are still present during Device garbage collection")

        get_lib().device_handle_free(self._ptr)

    def ffi_handle(self, handle_data: Any) -> FFI.CData:
        """Create and store a raw FFI pointer handle to prevent Python GC."""
        c_handle = ffi.new_handle(handle_data)
        self._handles.add(c_handle)
        return c_handle

    @staticmethod
    def from_ffi_handle(c_handle: FFI.CData) -> BaseFutureData:
        """Retrieve target Python data from raw FFI pointer handle and unregister it."""
        handle_data: BaseFutureData = ffi.from_handle(c_handle)
        handle_data.device._handles.remove(c_handle)
        return handle_data

    def loop_future(self) -> asyncio.Future[None] | None:
        """Return the asyncio Future associated with the background event loop."""
        return self._loop_future

    def connect(self, config: DeviceConfig) -> asyncio.Future[None]:
        """Connect the device using the provided configuration."""
        native_config = config.to_cdata()
        loop = asyncio.get_running_loop()
        lib = get_lib()

        # Handle events future
        loop_future = loop.create_future()
        loop_data = HandleEventsFutureData(loop_future, self)
        loop_data_handle = self.ffi_handle(loop_data)
        self._loop_future = loop_future

        # Connect future
        connect_future = loop.create_future()
        connect_data_handle = self.ffi_handle(
            ConnectFutureData(connect_future, self, loop_data)
        )

        lib.device_handle_connect(
            self._ptr,
            native_config.root(),
            connect_cbk,
            connect_data_handle,
            loop_cbk,
            loop_data_handle,
        )

        return connect_future

    def disconnect(self) -> asyncio.Future[None]:
        """Disconnect the device."""
        loop = asyncio.get_running_loop()
        future = loop.create_future()
        user_data = self.ffi_handle(DisconnectFutureData(future, self))
        lib = get_lib()

        lib.device_handle_disconnect(self._ptr, disconnect_cbk, user_data)
        return future

    def receive_data(self) -> asyncio.Future[DeviceEvent]:
        """Wait for and receive the next event from the Astarte cluster."""
        loop = asyncio.get_running_loop()
        future = loop.create_future()
        handle = self.ffi_handle(ReceiveFutureData(future, self))
        lib = get_lib()

        lib.device_handle_receive(self._ptr, receive_cbk, handle)
        return future

    def send_individual(
        self,
        interface: str,
        path: str,
        data: DeviceData,
        timestamp: datetime.datetime | None = None,
    ) -> asyncio.Future[None]:
        """Send an individual datastream value to Astarte."""
        lib = get_lib()
        interface_cstr = ffi.new("char[]", interface.encode())
        path_cstr = ffi.new("char[]", path.encode())
        data_refs = data.to_cdata()
        data_c = data_refs.root()[0]

        individual_data = ffi.new("NativeIndividualSend *")
        individual_data.interface = interface_cstr
        individual_data.path = path_cstr
        individual_data.data = data_c

        if timestamp is None:
            individual_data.timestamp.tag = lib.None_NativeTimestamp
        else:
            individual_data.timestamp.tag = lib.Some_NativeTimestamp
            individual_data.timestamp.some = int(timestamp.timestamp() * 1000.0)

        loop = asyncio.get_running_loop()
        future = loop.create_future()
        handle = self.ffi_handle(SendFutureData(future, self))

        lib.device_handle_send_individual(self._ptr, individual_data, send_cbk, handle)
        return future

    def send_object(
        self,
        interface: str,
        path: str,
        obj: DeviceObject,
        timestamp: datetime.datetime | None = None,
    ) -> asyncio.Future[None]:
        """Send an aggregate object value to Astarte."""
        lib = get_lib()
        interface_cstr = ffi.new("char[]", interface.encode())
        path_cstr = ffi.new("char[]", path.encode())
        object_refs = obj.to_cdata()
        object_c = object_refs.root()[0]

        object_data = ffi.new("NativeObjectSend *")
        object_data.interface = interface_cstr
        object_data.path = path_cstr
        object_data.data = object_c

        if timestamp is None:
            object_data.timestamp.tag = lib.None_NativeTimestamp
        else:
            object_data.timestamp.tag = lib.Some_NativeTimestamp
            object_data.timestamp.some = int(timestamp.timestamp() * 1000.0)

        loop = asyncio.get_running_loop()
        future = loop.create_future()
        handle = self.ffi_handle(SendFutureData(future, self))

        lib.device_handle_send_object(self._ptr, object_data, send_cbk, handle)
        return future

    def set_property(
        self, interface: str, path: str, value: DeviceData
    ) -> asyncio.Future[None]:
        """Set a property value on an Astarte property interface."""
        lib = get_lib()
        interface_cstr = ffi.new("char[]", interface.encode())
        path_cstr = ffi.new("char[]", path.encode())
        individual_refs = value.to_cdata()
        individual_c = individual_refs.root()[0]

        property_data = ffi.new("NativeSetProperty *")
        property_data.interface = interface_cstr
        property_data.path = path_cstr
        property_data.data = individual_c

        loop = asyncio.get_running_loop()
        future = loop.create_future()
        handle = self.ffi_handle(SendFutureData(future, self))

        lib.device_handle_set_property(self._ptr, property_data, send_cbk, handle)
        return future

    def unset_property(self, interface: str, path: str) -> asyncio.Future[None]:
        """Unset a property value on an Astarte property interface."""
        lib = get_lib()
        interface_cstr = ffi.new("char[]", interface.encode())
        path_cstr = ffi.new("char[]", path.encode())

        unset_data = ffi.new("NativePropertyIdentifier *")
        unset_data.interface = interface_cstr
        unset_data.path = path_cstr

        loop = asyncio.get_running_loop()
        future = loop.create_future()
        handle = self.ffi_handle(SendFutureData(future, self))

        lib.device_handle_unset_property(self._ptr, unset_data, send_cbk, handle)
        return future

    def get_property(
        self, interface: str, path: str
    ) -> asyncio.Future[DeviceProperty]:
        """Retrieve a stored property value from an Astarte property interface."""
        lib = get_lib()
        interface_cstr = ffi.new("char[]", interface.encode())
        path_cstr = ffi.new("char[]", path.encode())

        property_data = ffi.new("NativePropertyIdentifier *")
        property_data.interface = interface_cstr
        property_data.path = path_cstr

        loop = asyncio.get_running_loop()
        future = loop.create_future()
        handle = self.ffi_handle(GetPropertyFutureData(future, self))

        lib.device_handle_get_property(
            self._ptr, property_data, get_property_cbk, handle
        )
        return future
