import asyncio
from datetime import datetime, timezone

from astarte_device_python_bindings import (
    AstarteMqttDevice,
    init_device_async,
    AstarteDataWrapper,
    AstarteObjectWrapper,
)

file_path = "cred.secret"


async def handle_events_task(device: AstarteMqttDevice):
    await device.handle_events()


async def start_device(id, realm, secret, pairing):
    print(id, realm, secret, pairing)

    device = await init_device_async(
        id, realm, secret, pairing, "persistency", False, "interfaces"
    )

    asyncio.create_task(handle_events_task(device))

    print("hey")

    # while True:
    #     mapping = await device.get_mapping(
    #         "org.astarte-platform.python.examples.DeviceDatastream", "/longinteger_endpoint"
    #     )

    #     longinteger = 1 << 32

    #     print(type(longinteger))

    #     data = AstarteDataWrapper(mapping, longinteger)

    #     await device.send_individual(
    #         "org.astarte-platform.python.examples.DeviceDatastream", "/longinteger_endpoint", data
    #     )
    #     print("data sent")

    #     await asyncio.sleep(3)

    # while True:
    endpoint = "/test"
    mappings = await device.get_object_mapping_types(
        "org.astarte-platform.python.examples.DeviceAggregate"
    )

    object = AstarteObjectWrapper(
        mappings,
        {
            "double_endpoint": 5.4,
            "integer_endpoint": 42,
            "boolean_endpoint": True,
            "longinteger_endpoint": 45543543534,
            "string_endpoint": "hello",
            "binaryblob_endpoint": b"binblob",
            "datetime_endpoint": datetime(
                2022, 11, 22, 10, 11, 21, 0, tzinfo=timezone.utc
            ),
            "doublearray_endpoint": [22.2, 322.22, 12.3, 0.1],
            "integerarray_endpoint": [22, 322, 0, 10],
            "booleanarray_endpoint": [True, False, True, False],
            "longintegerarray_endpoint": [45543543534, 10, 0, 45543543534],
            "stringarray_endpoint": ["hello", " world"],
            "binaryblobarray_endpoint": [b"bin", b"blob"],
            "datetimearray_endpoint": [
                datetime(2022, 11, 22, 10, 11, 21, 0, tzinfo=timezone.utc),
                datetime(2022, 10, 21, 12, 5, 33, 0, tzinfo=timezone.utc),
            ],
        },
    )

    await device.send_object(
        "org.astarte-platform.python.examples.DeviceAggregate", endpoint, object
    )
    print("data sent")
    await asyncio.sleep(3)

    while True:
        event = await device.receive_event()

        if event.is_datastream_object():
            (object, date) = event.get_datastream_object()

            obj_dict = object.to_pydict()

            print(obj_dict, date)
        else:
            print("not handled")


def main():
    id = "MNsVDNpcTzSsZPfnRGB7yQ"
    realm = "dev"
    pairing = "https://api.eu1.astarte.cloud/pairing"

    with open(file_path, "r") as file:
        secret = file.read()
        secret = secret.strip()

    asyncio.run(start_device(id, realm, secret, pairing))


# If called as a script
if __name__ == "__main__":
    main()
