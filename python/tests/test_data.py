"""Unit tests for astarte.device.data module."""

import array
import datetime
import unittest

from astarte.device import (
    DeviceBinaryBlob,
    DeviceDataBoolean,
    DeviceDataBooleanArray,
    DeviceDataDateTime,
    DeviceDataDouble,
    DeviceDataDoubleArray,
    DeviceDataInteger,
    DeviceDataIntegerArray,
    DeviceDataLongInteger,
    DeviceDataLongIntegerArray,
    DeviceDataString,
    DeviceDataStringArray,
    DeviceObject,
)


class TestDeviceData(unittest.TestCase):
    def test_device_data_scalars(self):
        d_double = DeviceDataDouble(3.14)
        self.assertEqual(d_double.as_value(), 3.14)

        d_int = DeviceDataInteger(42)
        self.assertEqual(d_int.as_value(), 42)

        d_bool = DeviceDataBoolean(True)
        self.assertTrue(d_bool.as_value())

        d_long = DeviceDataLongInteger(1 << 35)
        self.assertEqual(d_long.as_value(), 1 << 35)

        d_str = DeviceDataString("hello astarte")
        self.assertEqual(d_str.as_value(), "hello astarte")

        now = datetime.datetime.now()
        d_dt = DeviceDataDateTime(now)
        self.assertEqual(d_dt.as_value(), now)

    def test_binary_blob(self):
        raw_bytes = b"test_blob_payload"
        blob = DeviceBinaryBlob.from_bytes(raw_bytes)
        self.assertEqual(bytes(blob), raw_bytes)
        self.assertEqual(len(blob), len(raw_bytes))
        self.assertEqual(blob[0], ord("t"))

    def test_device_data_arrays(self):
        arr_d = DeviceDataDoubleArray.from_array(array.array("d", [1.0, 2.5, 3.7]))
        self.assertEqual(len(arr_d), 3)

        arr_i = DeviceDataIntegerArray.from_array(array.array("i", [10, 20, 30]))
        self.assertEqual(len(arr_i), 3)

        arr_b = DeviceDataBooleanArray.from_list([True, False, True])
        self.assertEqual(len(arr_b), 3)

        arr_q = DeviceDataLongIntegerArray.from_array(array.array("q", [100, 200]))
        self.assertEqual(len(arr_q), 2)

        arr_s = DeviceDataStringArray(["alpha", "beta"])
        self.assertEqual(len(arr_s), 2)
        self.assertEqual(arr_s[0], "alpha")
        self.assertEqual(list(arr_s), ["alpha", "beta"])

    def test_device_object(self):
        obj = DeviceObject(
            {
                "temp": DeviceDataDouble(21.5),
                "status": DeviceDataString("OK"),
            }
        )
        self.assertIn("temp", obj.data)
        self.assertEqual(obj.data["temp"].as_value(), 21.5)
        self.assertEqual(obj.data["status"].as_value(), "OK")


if __name__ == "__main__":
    unittest.main()
