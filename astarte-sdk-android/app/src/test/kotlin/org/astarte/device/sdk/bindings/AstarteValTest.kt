package org.astarte.device.sdk.bindings

import org.junit.Assert.*
import org.junit.Test

class AstarteValTest {

    // ── Double ──────────────────────────────────────────────────────────────

    @Test fun double_field() {
        assertEquals(3.14, AstarteVal.Double(3.14).value, 0.0)
    }

    @Test fun double_equals_same_value() {
        assertEquals(AstarteVal.Double(1.0), AstarteVal.Double(1.0))
    }

    @Test fun double_not_equals_different_value() {
        assertNotEquals(AstarteVal.Double(1.0), AstarteVal.Double(2.0))
    }

    @Test fun double_hashcode_consistent() {
        assertEquals(AstarteVal.Double(1.0).hashCode(), AstarteVal.Double(1.0).hashCode())
    }

    @Test fun double_toString_contains_name() {
        assertTrue(AstarteVal.Double(1.0).toString().contains("Double"))
    }

    // ── Integer ─────────────────────────────────────────────────────────────

    @Test fun integer_field() {
        assertEquals(42, AstarteVal.Integer(42).value)
    }

    @Test fun integer_equals_same_value() {
        assertEquals(AstarteVal.Integer(42), AstarteVal.Integer(42))
    }

    @Test fun integer_not_equals_different_value() {
        assertNotEquals(AstarteVal.Integer(1), AstarteVal.Integer(2))
    }

    @Test fun integer_hashcode_consistent() {
        assertEquals(AstarteVal.Integer(7).hashCode(), AstarteVal.Integer(7).hashCode())
    }

    @Test fun integer_toString_contains_name() {
        assertTrue(AstarteVal.Integer(0).toString().contains("Integer"))
    }

    // ── Boolean ──────────────────────────────────────────────────────────────

    @Test fun boolean_field_true() {
        assertTrue(AstarteVal.Boolean(true).value)
    }

    @Test fun boolean_field_false() {
        assertFalse(AstarteVal.Boolean(false).value)
    }

    @Test fun boolean_equals_same_value() {
        assertEquals(AstarteVal.Boolean(true), AstarteVal.Boolean(true))
    }

    @Test fun boolean_not_equals_different_value() {
        assertNotEquals(AstarteVal.Boolean(true), AstarteVal.Boolean(false))
    }

    @Test fun boolean_hashcode_consistent() {
        assertEquals(AstarteVal.Boolean(false).hashCode(), AstarteVal.Boolean(false).hashCode())
    }

    // ── LongInteger ──────────────────────────────────────────────────────────

    @Test fun long_integer_field() {
        assertEquals(Long.MAX_VALUE, AstarteVal.LongInteger(Long.MAX_VALUE).value)
    }

    @Test fun long_integer_equals_same_value() {
        assertEquals(AstarteVal.LongInteger(100L), AstarteVal.LongInteger(100L))
    }

    @Test fun long_integer_not_equals_different_value() {
        assertNotEquals(AstarteVal.LongInteger(1L), AstarteVal.LongInteger(2L))
    }

    @Test fun long_integer_hashcode_consistent() {
        assertEquals(AstarteVal.LongInteger(0L).hashCode(), AstarteVal.LongInteger(0L).hashCode())
    }

    // ── IString ──────────────────────────────────────────────────────────────

    @Test fun istring_field() {
        assertEquals("hello", AstarteVal.IString("hello").value)
    }

    @Test fun istring_equals_same_value() {
        assertEquals(AstarteVal.IString("abc"), AstarteVal.IString("abc"))
    }

    @Test fun istring_not_equals_different_value() {
        assertNotEquals(AstarteVal.IString("a"), AstarteVal.IString("b"))
    }

    @Test fun istring_hashcode_consistent() {
        assertEquals(AstarteVal.IString("x").hashCode(), AstarteVal.IString("x").hashCode())
    }

    // ── BinaryBlob ───────────────────────────────────────────────────────────
    // data class with ByteArray uses reference equality; test field content directly.

    @Test fun binary_blob_field_content() {
        val data = byteArrayOf(0x01, 0x02, 0x03)
        assertArrayEquals(data, AstarteVal.BinaryBlob(data).value)
    }

    @Test fun binary_blob_same_instance_equals_itself() {
        val v = AstarteVal.BinaryBlob(byteArrayOf(1, 2))
        assertEquals(v, v)
    }

    // ── DateTime ─────────────────────────────────────────────────────────────

    @Test fun datetime_field() {
        assertEquals(1_700_000_000_000L, AstarteVal.DateTime(1_700_000_000_000L).value)
    }

    @Test fun datetime_equals_same_value() {
        assertEquals(AstarteVal.DateTime(0L), AstarteVal.DateTime(0L))
    }

    @Test fun datetime_not_equals_different_value() {
        assertNotEquals(AstarteVal.DateTime(1L), AstarteVal.DateTime(2L))
    }

    // ── DoubleArray ──────────────────────────────────────────────────────────

    @Test fun double_array_field_content() {
        val arr = doubleArrayOf(1.0, 2.0, 3.0)
        assertArrayEquals(arr, AstarteVal.DoubleArray(arr).value, 0.0)
    }

    @Test fun double_array_same_instance_equals_itself() {
        val v = AstarteVal.DoubleArray(doubleArrayOf(1.0, 2.0))
        assertEquals(v, v)
    }

    // ── IntegerArray ─────────────────────────────────────────────────────────

    @Test fun integer_array_field_content() {
        val arr = intArrayOf(10, 20, 30)
        assertArrayEquals(arr, AstarteVal.IntegerArray(arr).value)
    }

    @Test fun integer_array_same_instance_equals_itself() {
        val v = AstarteVal.IntegerArray(intArrayOf(1, 2))
        assertEquals(v, v)
    }

    // ── BooleanArray ─────────────────────────────────────────────────────────

    @Test fun boolean_array_field_content() {
        val arr = booleanArrayOf(true, false, true)
        assertArrayEquals(arr.map { it }.toTypedArray(), AstarteVal.BooleanArray(arr).value.map { it }.toTypedArray())
    }

    @Test fun boolean_array_same_instance_equals_itself() {
        val v = AstarteVal.BooleanArray(booleanArrayOf(true, false))
        assertEquals(v, v)
    }

    // ── LongIntegerArray ─────────────────────────────────────────────────────

    @Test fun long_integer_array_field_content() {
        val arr = longArrayOf(100L, 200L)
        assertArrayEquals(arr, AstarteVal.LongIntegerArray(arr).value)
    }

    @Test fun long_integer_array_same_instance_equals_itself() {
        val v = AstarteVal.LongIntegerArray(longArrayOf(1L, 2L))
        assertEquals(v, v)
    }

    // ── StringArray ──────────────────────────────────────────────────────────

    @Test fun string_array_field_content() {
        val list = listOf("a", "b", "c")
        assertEquals(list, AstarteVal.StringArray(list).value)
    }

    @Test fun string_array_equals_same_content() {
        assertEquals(AstarteVal.StringArray(listOf("x", "y")), AstarteVal.StringArray(listOf("x", "y")))
    }

    @Test fun string_array_not_equals_different_content() {
        assertNotEquals(AstarteVal.StringArray(listOf("a")), AstarteVal.StringArray(listOf("b")))
    }

    // ── DateTimeArray ────────────────────────────────────────────────────────

    @Test fun datetime_array_field_content() {
        val arr = longArrayOf(1000L, 2000L)
        assertArrayEquals(arr, AstarteVal.DateTimeArray(arr).value)
    }

    @Test fun datetime_array_same_instance_equals_itself() {
        val v = AstarteVal.DateTimeArray(longArrayOf(1L, 2L))
        assertEquals(v, v)
    }

    // ── Cross-type inequality ────────────────────────────────────────────────

    @Test fun different_variants_are_not_equal() {
        assertNotEquals(AstarteVal.Integer(1), AstarteVal.Double(1.0))
    }

    @Test fun variant_not_equals_null() {
        assertNotEquals(null, AstarteVal.Integer(0))
    }

    @Test fun same_instance_equals_itself() {
        val v: AstarteVal = AstarteVal.Integer(5)
        assertEquals(v, v)
    }
}
