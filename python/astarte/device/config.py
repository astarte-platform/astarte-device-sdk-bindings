"""Device configuration class for Astarte Device SDK bindings."""

from __future__ import annotations

from astarte.device._ffi import ffi, get_lib
from astarte.device.data import CDataRefs


class DeviceConfig:
    """Configuration parameter container for instantiating/connecting an Astarte Device."""

    def __init__(
        self,
        device_id: str,
        cred_secr: str,
        realm: str,
        pairing_url: str,
        interfaces_dir: str,
        writable_dir: str = "",
        channel_size: int = 0,
    ):
        self._device_id = device_id
        self._cred_secr = cred_secr
        self._realm = realm
        self._pairing_url = pairing_url
        self._interfaces_dir = interfaces_dir
        self._writable_dir = writable_dir
        self._channel_size = channel_size

    def to_cdata(self) -> CDataRefs:
        """Convert configuration to native CData representation."""
        lib = get_lib()

        device_id = ffi.new("char[]", self._device_id.encode())
        cred_secr = ffi.new("char[]", self._cred_secr.encode())
        realm = ffi.new("char[]", self._realm.encode())
        pairing_url = ffi.new("char[]", self._pairing_url.encode())
        interfaces_dir = ffi.new("char[]", self._interfaces_dir.encode())
        writable_dir = ffi.new("char[]", self._writable_dir.encode())

        native_config = ffi.new("NativeDeviceConfig *")
        native_config.generic.interfaces_dir = interfaces_dir
        native_config.generic.channel_size = self._channel_size
        native_config.generic.writable_dir = writable_dir
        native_config.connection.tag = lib.Mqtt
        native_config.connection.mqtt.device_id = device_id
        native_config.connection.mqtt.cred_secr = cred_secr
        native_config.connection.mqtt.realm = realm
        native_config.connection.mqtt.pairing_url = pairing_url

        return CDataRefs(native_config).ref_list(
            [device_id, cred_secr, realm, pairing_url, interfaces_dir, writable_dir]
        )
