package org.astarte.device.sdk.bindings;

import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.*;

class AstarteValTest {

    // ── Double ───────────────────────────────────────────────────────────────

    @Test void double_field() {
        assertEquals(3.14, new AstarteVal.Double(3.14).value, 0.0);
    }

    @Test void double_equals_same_value() {
        assertEquals(new AstarteVal.Double(1.0), new AstarteVal.Double(1.0));
    }

    @Test void double_not_equals_different_value() {
        assertNotEquals(new AstarteVal.Double(1.0), new AstarteVal.Double(2.0));
    }

    @Test void double_hashcode_consistent() {
        assertEquals(new AstarteVal.Double(1.0).hashCode(), new AstarteVal.Double(1.0).hashCode());
    }

    @Test void double_toString_contains_name() {
        assertTrue(new AstarteVal.Double(1.0).toString().contains("Double"));
    }

    // ── Integer ─────────────────────────────────────────────────────────────

    @Test void iinteger_field() {
        assertEquals(42, new AstarteVal.Integer(42).value);
    }

    @Test void iinteger_equals_same_value() {
        assertEquals(new AstarteVal.Integer(42), new AstarteVal.Integer(42));
    }

    @Test void iinteger_not_equals_different_value() {
        assertNotEquals(new AstarteVal.Integer(1), new AstarteVal.Integer(2));
    }

    @Test void iinteger_hashcode_consistent() {
        assertEquals(new AstarteVal.Integer(7).hashCode(), new AstarteVal.Integer(7).hashCode());
    }

    @Test void iinteger_toString_contains_name() {
        assertTrue(new AstarteVal.Integer(0).toString().contains("Integer"));
    }

    // ── Boolean ──────────────────────────────────────────────────────────────

    @Test void boolean_field_true() {
        assertTrue(new AstarteVal.Boolean(true).value);
    }

    @Test void boolean_field_false() {
        assertFalse(new AstarteVal.Boolean(false).value);
    }

    @Test void boolean_equals_same_value() {
        assertEquals(new AstarteVal.Boolean(true), new AstarteVal.Boolean(true));
    }

    @Test void boolean_not_equals_different_value() {
        assertNotEquals(new AstarteVal.Boolean(true), new AstarteVal.Boolean(false));
    }

    @Test void boolean_hashcode_consistent() {
        assertEquals(new AstarteVal.Boolean(false).hashCode(), new AstarteVal.Boolean(false).hashCode());
    }

    // ── LongInteger ──────────────────────────────────────────────────────────

    @Test void long_integer_field() {
        assertEquals(Long.MAX_VALUE, new AstarteVal.LongInteger(Long.MAX_VALUE).value);
    }

    @Test void long_integer_equals_same_value() {
        assertEquals(new AstarteVal.LongInteger(100L), new AstarteVal.LongInteger(100L));
    }

    @Test void long_integer_not_equals_different_value() {
        assertNotEquals(new AstarteVal.LongInteger(1L), new AstarteVal.LongInteger(2L));
    }

    @Test void long_integer_hashcode_consistent() {
        assertEquals(new AstarteVal.LongInteger(0L).hashCode(), new AstarteVal.LongInteger(0L).hashCode());
    }

    // ── IString ──────────────────────────────────────────────────────────────

    @Test void istring_field() {
        assertEquals("hello", new AstarteVal.IString("hello").value);
    }

    @Test void istring_equals_same_value() {
        assertEquals(new AstarteVal.IString("abc"), new AstarteVal.IString("abc"));
    }

    @Test void istring_not_equals_different_value() {
        assertNotEquals(new AstarteVal.IString("a"), new AstarteVal.IString("b"));
    }

    @Test void istring_hashcode_consistent() {
        assertEquals(new AstarteVal.IString("x").hashCode(), new AstarteVal.IString("x").hashCode());
    }

    // ── BinaryBlob ───────────────────────────────────────────────────────────

    @Test void binary_blob_field_content() {
        byte[] data = {0x01, 0x02, 0x03};
        assertTrue(Arrays.equals(data, new AstarteVal.BinaryBlob(data).value));
    }

    @Test void binary_blob_equals_same_content() {
        assertEquals(new AstarteVal.BinaryBlob(new byte[]{1, 2}), new AstarteVal.BinaryBlob(new byte[]{1, 2}));
    }

    @Test void binary_blob_not_equals_different_content() {
        assertNotEquals(new AstarteVal.BinaryBlob(new byte[]{1}), new AstarteVal.BinaryBlob(new byte[]{2}));
    }

    @Test void binary_blob_hashcode_consistent() {
        assertEquals(
            new AstarteVal.BinaryBlob(new byte[]{9}).hashCode(),
            new AstarteVal.BinaryBlob(new byte[]{9}).hashCode()
        );
    }

    // ── DateTime ─────────────────────────────────────────────────────────────

    @Test void datetime_field() {
        assertEquals(1_700_000_000_000L, new AstarteVal.DateTime(1_700_000_000_000L).value);
    }

    @Test void datetime_equals_same_value() {
        assertEquals(new AstarteVal.DateTime(0L), new AstarteVal.DateTime(0L));
    }

    @Test void datetime_not_equals_different_value() {
        assertNotEquals(new AstarteVal.DateTime(1L), new AstarteVal.DateTime(2L));
    }

    // ── DoubleArray ──────────────────────────────────────────────────────────

    @Test void double_array_field_content() {
        double[] arr = {1.0, 2.0, 3.0};
        assertTrue(Arrays.equals(arr, new AstarteVal.DoubleArray(arr).value));
    }

    @Test void double_array_equals_same_content() {
        assertEquals(new AstarteVal.DoubleArray(new double[]{1.0, 2.0}), new AstarteVal.DoubleArray(new double[]{1.0, 2.0}));
    }

    @Test void double_array_not_equals_different_content() {
        assertNotEquals(new AstarteVal.DoubleArray(new double[]{1.0}), new AstarteVal.DoubleArray(new double[]{2.0}));
    }

    // ── IntegerArray ─────────────────────────────────────────────────────────

    @Test void integer_array_field_content() {
        int[] arr = {10, 20, 30};
        assertTrue(Arrays.equals(arr, new AstarteVal.IntegerArray(arr).value));
    }

    @Test void integer_array_equals_same_content() {
        assertEquals(new AstarteVal.IntegerArray(new int[]{1, 2}), new AstarteVal.IntegerArray(new int[]{1, 2}));
    }

    @Test void integer_array_not_equals_different_content() {
        assertNotEquals(new AstarteVal.IntegerArray(new int[]{1}), new AstarteVal.IntegerArray(new int[]{9}));
    }

    // ── BooleanArray ─────────────────────────────────────────────────────────

    @Test void boolean_array_field_content() {
        boolean[] arr = {true, false, true};
        assertTrue(Arrays.equals(arr, new AstarteVal.BooleanArray(arr).value));
    }

    @Test void boolean_array_equals_same_content() {
        assertEquals(new AstarteVal.BooleanArray(new boolean[]{true, false}), new AstarteVal.BooleanArray(new boolean[]{true, false}));
    }

    @Test void boolean_array_not_equals_different_content() {
        assertNotEquals(new AstarteVal.BooleanArray(new boolean[]{true}), new AstarteVal.BooleanArray(new boolean[]{false}));
    }

    // ── LongIntegerArray ─────────────────────────────────────────────────────

    @Test void long_integer_array_field_content() {
        long[] arr = {100L, 200L};
        assertTrue(Arrays.equals(arr, new AstarteVal.LongIntegerArray(arr).value));
    }

    @Test void long_integer_array_equals_same_content() {
        assertEquals(new AstarteVal.LongIntegerArray(new long[]{1L, 2L}), new AstarteVal.LongIntegerArray(new long[]{1L, 2L}));
    }

    // ── StringArray ──────────────────────────────────────────────────────────

    @Test void string_array_field_content() {
        java.util.List<String> list = Arrays.asList("a", "b", "c");
        assertEquals(list, new AstarteVal.StringArray(list).value);
    }

    @Test void string_array_equals_same_content() {
        assertEquals(
            new AstarteVal.StringArray(Arrays.asList("x", "y")),
            new AstarteVal.StringArray(Arrays.asList("x", "y"))
        );
    }

    @Test void string_array_not_equals_different_content() {
        assertNotEquals(
            new AstarteVal.StringArray(Arrays.asList("a")),
            new AstarteVal.StringArray(Arrays.asList("b"))
        );
    }

    // ── DateTimeArray ────────────────────────────────────────────────────────

    @Test void datetime_array_field_content() {
        long[] arr = {1000L, 2000L};
        assertTrue(Arrays.equals(arr, new AstarteVal.DateTimeArray(arr).value));
    }

    @Test void datetime_array_equals_same_content() {
        assertEquals(new AstarteVal.DateTimeArray(new long[]{1L, 2L}), new AstarteVal.DateTimeArray(new long[]{1L, 2L}));
    }

    // ── Cross-type inequality ────────────────────────────────────────────────

    @Test void different_variants_are_not_equal() {
        assertNotEquals(new AstarteVal.Integer(1), new AstarteVal.Double(1.0));
    }

    @Test void variant_not_equals_null() {
        assertNotEquals(null, new AstarteVal.Integer(0));
    }

    @Test void same_instance_equals_itself() {
        AstarteVal v = new AstarteVal.Integer(5);
        assertEquals(v, v);
    }
}
