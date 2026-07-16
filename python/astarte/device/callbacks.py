"""FFI Callbacks and Future data containers for Astarte Device SDK bindings."""

from __future__ import annotations

import asyncio
from typing import TYPE_CHECKING, TypeVar, Generic

from astarte.device._ffi import ffi, get_lib, FFI
from astarte.device.data import DeviceEvent, DeviceProperty
from astarte.device.exceptions import (
    ConnectError,
    DisconnectError,
    GetPropertyError,
    HandleEventsError,
    InvalidNativeValueError,
    ReceiveError,
    SendError,
)

if TYPE_CHECKING:
    from astarte.device.device import Device

T = TypeVar('T')

class BaseFutureData[T]:
    def __init__(self, device: Device, future: asyncio.Future[T]):
        self.device = device
        self.future = future

class GetPropertyFutureData(BaseFutureData[DeviceProperty]):
    def __init__(self, future: asyncio.Future[DeviceProperty], device: Device):
        super().__init__(device, future)


class SendFutureData(BaseFutureData[None]):
    def __init__(self, future: asyncio.Future[None], device: Device):
        super().__init__(device, future)


class ReceiveFutureData(BaseFutureData[DeviceEvent]):
    def __init__(self, future: asyncio.Future[DeviceEvent], device: Device):
        super().__init__(device, future)


class DisconnectFutureData(BaseFutureData[None]):
    def __init__(self, future: asyncio.Future[None], device: Device):
        super().__init__(device, future)


class ConnectFutureData(BaseFutureData[None]):
    def __init__(
        self,
        future: asyncio.Future[None],
        device: Device,
        loop_data: FFI.CData,
    ):
        super().__init__(device, future)
        self.loop_data = loop_data


class HandleEventsFutureData(BaseFutureData[None]):
    def __init__(self, future: asyncio.Future[None], device: Device):
        super().__init__(device, future)


@ffi.callback("void(const struct NativeStringResult_NativeOption_NativeDeviceData *, UserData)")
# @ffi.def_extern()
def get_property_cbk(native_res, user_data):
    from astarte.device.device import Device

    base = Device.from_ffi_handle(user_data)
    if not isinstance(base, GetPropertyFutureData):
        base.future.get_loop().call_soon_threadsafe(
            base.future.set_exception, Exception("get_property_cbk called with wrong user data")
        )
        return
    data = base

    lib = get_lib()

    if native_res.tag == lib.Ok_NativeOption_NativeDeviceData:
        prop = DeviceProperty(native_res.ok)
        data.future.get_loop().call_soon_threadsafe(data.future.set_result, prop)
    elif native_res.tag == lib.Err_NativeOption_NativeDeviceData:
        error_str = ffi.string(native_res.err, 1024).decode()
        error = GetPropertyError(error_str)
        data.future.get_loop().call_soon_threadsafe(data.future.set_exception, error)
    else:
        data.future.get_loop().call_soon_threadsafe(
            data.future.set_exception, InvalidNativeValueError()
        )


@ffi.callback("void(const struct NativeStringResult_bool *, UserData)")
def send_cbk(native_res, user_data):
    from astarte.device.device import Device

    base = Device.from_ffi_handle(user_data)
    if not isinstance(base, SendFutureData):
        base.future.get_loop().call_soon_threadsafe(
            base.future.set_exception, Exception("send_cbk called with wrong user data")
        )
        return
    data = base

    lib = get_lib()

    if native_res.tag == lib.Ok_bool:
        data.future.get_loop().call_soon_threadsafe(data.future.set_result, None)
    elif native_res.tag == lib.Err_bool:
        error_str = ffi.string(native_res.err, 1024).decode()
        error = SendError(error_str)
        data.future.get_loop().call_soon_threadsafe(data.future.set_exception, error)
    else:
        data.future.get_loop().call_soon_threadsafe(
            data.future.set_exception, InvalidNativeValueError()
        )


@ffi.callback("void(const struct NativeStringResult_NativeManuallyDrop_NativeDeviceEvent *, UserData)")
def receive_cbk(native_res, user_data):
    from astarte.device.device import Device

    base = Device.from_ffi_handle(user_data)
    if not isinstance(base, ReceiveFutureData):
        base.future.get_loop().call_soon_threadsafe(
            base.future.set_exception, Exception("receive_cbk called with wrong user data")
        )
        return
    data = base

    lib = get_lib()

    if native_res.tag == lib.Ok_NativeManuallyDrop_NativeDeviceEvent:
        event = DeviceEvent(native_res.ok)
        data.future.get_loop().call_soon_threadsafe(data.future.set_result, event)
    elif native_res.tag == lib.Err_NativeManuallyDrop_NativeDeviceEvent:
        error_str = ffi.string(native_res.err, 1024).decode()
        error = ReceiveError(error_str)
        data.future.get_loop().call_soon_threadsafe(data.future.set_exception, error)
    else:
        data.future.get_loop().call_soon_threadsafe(
            data.future.set_exception, InvalidNativeValueError()
        )


@ffi.callback("void(const struct NativeStringResult_bool *, UserData)")
def disconnect_cbk(native_res, user_data):
    from astarte.device.device import Device

    base = Device.from_ffi_handle(user_data)
    if not isinstance(base, DisconnectFutureData):
        base.future.get_loop().call_soon_threadsafe(
            base.future.set_exception, Exception("disconnect_cbk called with wrong user data")
        )
        return

    data = base
    lib = get_lib()

    if native_res.tag == lib.Ok_bool:
        data.future.get_loop().call_soon_threadsafe(data.future.set_result, None)
    elif native_res.tag == lib.Err_bool:
        error_str = ffi.string(native_res.err, 1024).decode()
        loop_error = DisconnectError(error_str)
        data.future.get_loop().call_soon_threadsafe(data.future.set_exception, loop_error)
    else:
        data.future.get_loop().call_soon_threadsafe(
            data.future.set_exception, InvalidNativeValueError()
        )


@ffi.callback("void(const struct NativeStringResult_bool *, UserData)")
def connect_cbk(native_result, user_data):
    from astarte.device.device import Device

    base = Device.from_ffi_handle(user_data)
    if not isinstance(base, ConnectFutureData):
        base.future.get_loop().call_soon_threadsafe(
            base.future.set_exception, Exception("connect_cbk called with wrong user data, the loop callback won't be stopped")
        )
        return

    data = base
    lib = get_lib()

    if native_result.tag == lib.Ok_bool:
        data.future.get_loop().call_soon_threadsafe(data.future.set_result, None)
    elif native_result.tag == lib.Err_bool:
        error_str = ffi.string(native_result.err, 1024).decode()
        connect_error = ConnectError(error_str)
        data.future.get_loop().call_soon_threadsafe(data.future.set_exception, connect_error)
        # NOTE fail loop callback too and consume handle
        loop_data = Device.from_ffi_handle(data.loop_data)
        loop_data.future.get_loop().call_soon_threadsafe(loop_data.future.set_exception, connect_error)
    else:
        err = InvalidNativeValueError()
        data.future.get_loop().call_soon_threadsafe(data.future.set_exception, err)
        # NOTE fail loop callback too and consume handle
        loop_data = Device.from_ffi_handle(data.loop_data)
        loop_data.future.get_loop().call_soon_threadsafe(loop_data.future.set_exception, err)


@ffi.callback("void(const struct NativeStringResult_bool *, UserData)")
def loop_cbk(native_result, user_data):
    from astarte.device.device import Device

    base = Device.from_ffi_handle(user_data)
    if not isinstance(base, HandleEventsFutureData):
        base.future.get_loop().call_soon_threadsafe(
            base.future.set_exception, Exception("loop_cbk called with wrong user data")
        )
        return

    data = base
    lib = get_lib()

    if native_result.tag == lib.Ok_bool:
        data.future.get_loop().call_soon_threadsafe(data.future.set_result, None)
    elif native_result.tag == lib.Err_bool:
        error_str = ffi.string(native_result.err, 1024).decode()
        loop_error = HandleEventsError(error_str)
        data.future.get_loop().call_soon_threadsafe(data.future.set_exception, loop_error)
    else:
        data.future.get_loop().call_soon_threadsafe(
            data.future.set_exception, InvalidNativeValueError()
        )
