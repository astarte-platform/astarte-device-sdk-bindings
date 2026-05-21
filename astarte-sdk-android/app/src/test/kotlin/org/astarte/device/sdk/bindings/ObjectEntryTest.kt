package org.astarte.device.sdk.bindings

import org.junit.Assert.*
import org.junit.Test

class ObjectEntryTest {

    private fun entry(key: String, value: AstarteVal) = ObjectEntry(key, value)

    @Test fun fields_are_accessible() {
        val e = entry("temperature", AstarteVal.Double(23.5))
        assertEquals("temperature", e.key)
        assertTrue(e.value is AstarteVal.Double)
    }

    @Test fun equals_same_instance() {
        val e = entry("k", AstarteVal.Integer(1))
        assertEquals(e, e)
    }

    @Test fun equals_two_equal_instances() {
        assertEquals(
            entry("k", AstarteVal.Integer(1)),
            entry("k", AstarteVal.Integer(1))
        )
    }

    @Test fun not_equals_different_key() {
        assertNotEquals(
            entry("a", AstarteVal.Integer(1)),
            entry("b", AstarteVal.Integer(1))
        )
    }

    @Test fun not_equals_different_value() {
        assertNotEquals(
            entry("k", AstarteVal.Integer(1)),
            entry("k", AstarteVal.Integer(2))
        )
    }

    @Test fun not_equals_null() {
        assertNotEquals(null, entry("k", AstarteVal.Integer(0)))
    }

    @Test fun hashcode_consistent_with_equals() {
        val a = entry("key", AstarteVal.IString("hello"))
        val b = entry("key", AstarteVal.IString("hello"))
        assertEquals(a, b)
        assertEquals(a.hashCode(), b.hashCode())
    }

    @Test fun toString_contains_key_and_value() {
        val s = entry("sensor", AstarteVal.Boolean(true)).toString()
        assertTrue(s.contains("sensor"))
        assertTrue(s.contains("Boolean") || s.contains("true"))
    }
}
