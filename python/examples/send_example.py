"""Example usage script for Astarte Device SDK Python bindings."""

import array
import asyncio
import datetime

from astarte.device import (
    Device,
    DeviceBinaryBlob,
    DeviceConfig,
    DeviceDataBinaryBlob,
    DeviceDataBinaryBlobArray,
    DeviceDataBoolean,
    DeviceDataBooleanArray,
    DeviceDataDateTime,
    DeviceDataDateTimeArray,
    DeviceDataDouble,
    DeviceDataDoubleArray,
    DeviceDataInteger,
    DeviceDataIntegerArray,
    DeviceDataLongInteger,
    DeviceDataLongIntegerArray,
    DeviceDataString,
    DeviceDataStringArray,
    DeviceObject,
    DeviceValueObject,
)


async def wait_handle_events(device: Device):
    try:
        events = device.loop_future()
        if events is not None:
            await events
        else:
            raise Exception("connect first")

        print("handle events joined :D")
    except Exception:
        print("got error in handle events future")


async def receive_print_data(device: Device):
    event = await device.receive_data()
    print(event.interface)
    print(event.path)
    print(event.data)

    if isinstance(event.data, DeviceValueObject):
        obj: DeviceValueObject = event.data
        print(obj.data)

        for v in obj.data.values():
            print(type(v))
            val = v.as_value()
            if isinstance(v, DeviceDataBinaryBlob):
                b: DeviceDataBinaryBlob = v
                print("bytes", bytes(b.as_value()))
            elif val is not None:
                print("scalar", val)

            vec = v.as_vector()
            if isinstance(v, DeviceDataBinaryBlobArray):
                b_vec: DeviceDataBinaryBlobArray = v
                print("bytes array", [bytes(b) for b in b_vec])
            elif vec is not None:
                print("vector", [d for d in vec])


async def main():
    config = DeviceConfig(
        device_id="DayugqhpTPi2RgkELFPj9Q",
        cred_secr="cMin6aYkLcFCqqH0LD641jaEuMoiZFRzLTE96enpEbo=",
        realm="test",
        pairing_url="http://api.astarte.localhost/pairing",
        interfaces_dir="examples/interfaces",
        writable_dir="/tmp/bindings-test"
    )

    device = Device()

    await device.connect(config)

    asyncio.create_task(wait_handle_events(device))

    print(device)

    await asyncio.sleep(2)

    await device.send_individual(
        "org.astarte-platform.rust.e2etest.DeviceDatastream",
        "/doublearray_endpoint",
        DeviceDataDoubleArray.from_array(
            array.array("d", [1.0, 1.1, 1.2, 1.3, 10999999999.49])
        ),
        datetime.datetime.now(),
    )

    obj = DeviceObject(
        {
            "double_endpoint": DeviceDataDouble(3.14),
            "integer_endpoint": DeviceDataInteger(1),
            "boolean_endpoint": DeviceDataBoolean(True),
            "longinteger_endpoint": DeviceDataLongInteger(1 << 32),
            "string_endpoint": DeviceDataString("hey"),
            "binaryblob_endpoint": DeviceDataBinaryBlob(
                DeviceBinaryBlob.from_bytes(b"tests")
            ),
            "datetime_endpoint": DeviceDataDateTime(datetime.datetime.now()),
            "doublearray_endpoint": DeviceDataDoubleArray.from_array(
                array.array("d", [1.1, 1.2, 1.3])
            ),
            "integerarray_endpoint": DeviceDataIntegerArray.from_array(
                array.array("i", [1, 2, 3])
            ),
            "booleanarray_endpoint": DeviceDataBooleanArray.from_list(
                [True, False, True]
            ),
            "longintegerarray_endpoint": DeviceDataLongIntegerArray.from_array(
                array.array("q", [1 << 33, 1 << 34])
            ),
            "stringarray_endpoint": DeviceDataStringArray(["a", "b", "c"]),
            "binaryblobarray_endpoint": DeviceDataBinaryBlobArray(
                [
                    DeviceBinaryBlob.from_bytes(b"blob1"),
                    DeviceBinaryBlob.from_bytes(b"blob2"),
                    DeviceBinaryBlob.from_bytes(b"this is a test binary blob"),
                    DeviceBinaryBlob.from_bytes(b"blob1"),
                    DeviceBinaryBlob.from_bytes(b"blob2"),
                    DeviceBinaryBlob.from_bytes(b"blob1"),
                    DeviceBinaryBlob.from_bytes(b"blob2"),
                ]
            ),
            "datetimearray_endpoint": DeviceDataDateTimeArray(
                [datetime.datetime.now(), datetime.datetime.now()]
            ),
        }
    )

    await device.send_object(
        "org.astarte-platform.rust.e2etest.DeviceAggregate",
        "/test",
        obj,
        datetime.datetime.now(),
    )

    print("setting property")

    await device.set_property(
        "org.astarte-platform.rust.e2etest.ForUpdateDeviceProperty",
        "/sensor_1/endpoint",
        DeviceDataDouble(3231.1231),
    )

    print("unsetting property")

    await device.unset_property(
        "org.astarte-platform.rust.e2etest.DeviceProperty",
        "/sensor_1/doublearray_endpoint",
    )

    print("getting property")

    prop = await device.get_property(
        "org.astarte-platform.rust.e2etest.ForUpdateDeviceProperty", "/sensor_1/endpoint"
    )
    print("property", prop)
    print("value", prop.value())

    await receive_print_data(device)

    print("stopping device...", flush=True)

    await device.disconnect()

    print("stopped device", flush=True)


if __name__ == "__main__":
    asyncio.run(main())
