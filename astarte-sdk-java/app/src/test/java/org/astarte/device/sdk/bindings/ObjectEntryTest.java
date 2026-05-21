package org.astarte.device.sdk.bindings;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class ObjectEntryTest {

    private static ObjectEntry entry(String key, AstarteVal value) {
        return new ObjectEntry(key, value);
    }

    @Test void fields_are_accessible() {
        ObjectEntry e = entry("temperature", new AstarteVal.Double(23.5));
        assertEquals("temperature", e.key);
        assertInstanceOf(AstarteVal.Double.class, e.value);
    }

    @Test void getters_match_fields() {
        ObjectEntry e = entry("count", new AstarteVal.Integer(7));
        assertEquals(e.key, e.key());
        assertEquals(e.value, e.value());
    }

    @Test void equals_same_instance() {
        ObjectEntry e = entry("k", new AstarteVal.Integer(1));
        assertEquals(e, e);
    }

    @Test void equals_two_equal_instances() {
        assertEquals(
            entry("k", new AstarteVal.Integer(1)),
            entry("k", new AstarteVal.Integer(1))
        );
    }

    @Test void not_equals_different_key() {
        assertNotEquals(
            entry("a", new AstarteVal.Integer(1)),
            entry("b", new AstarteVal.Integer(1))
        );
    }

    @Test void not_equals_different_value() {
        assertNotEquals(
            entry("k", new AstarteVal.Integer(1)),
            entry("k", new AstarteVal.Integer(2))
        );
    }

    @Test void not_equals_null() {
        assertNotEquals(null, entry("k", new AstarteVal.Integer(0)));
    }

    @Test void hashcode_consistent_with_equals() {
        ObjectEntry a = entry("key", new AstarteVal.IString("hello"));
        ObjectEntry b = entry("key", new AstarteVal.IString("hello"));
        assertEquals(a, b);
        assertEquals(a.hashCode(), b.hashCode());
    }

    @Test void toString_contains_key_and_value() {
        String s = entry("sensor", new AstarteVal.Boolean(true)).toString();
        assertTrue(s.contains("sensor"));
        assertTrue(s.contains("Boolean") || s.contains("true"));
    }
}
