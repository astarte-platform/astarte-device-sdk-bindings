package org.astarte.device.sdk.bindings;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class AstarteConfigTest {

    private static final String REALM    = "myrealm";
    private static final String DEV_ID   = "device-id-123";
    private static final String SECRET   = "secret-abc";
    private static final String PAIRING  = "http://api.astarte.example/pairing";

    private static AstarteConfig cfg(boolean ignoreSsl) {
        return new AstarteConfig(REALM, DEV_ID, SECRET, PAIRING, ignoreSsl);
    }

    // ── Field access ─────────────────────────────────────────────────────────

    @Test void fields_are_accessible() {
        AstarteConfig c = cfg(false);
        assertEquals(REALM,   c.realm);
        assertEquals(DEV_ID,  c.deviceId);
        assertEquals(SECRET,  c.credentialsSecret);
        assertEquals(PAIRING, c.pairingUrl);
        assertFalse(c.ignoreSsl);
    }

    @Test void getters_match_fields() {
        AstarteConfig c = cfg(true);
        assertEquals(c.realm,              c.realm());
        assertEquals(c.deviceId,           c.deviceId());
        assertEquals(c.credentialsSecret,  c.credentialsSecret());
        assertEquals(c.pairingUrl,         c.pairingUrl());
        assertEquals(c.ignoreSsl,          c.ignoreSsl());
    }

    @Test void ignore_ssl_true() {
        assertTrue(cfg(true).ignoreSsl);
    }

    @Test void ignore_ssl_false() {
        assertFalse(cfg(false).ignoreSsl);
    }

    // ── equals ───────────────────────────────────────────────────────────────

    @Test void equals_same_instance() {
        AstarteConfig c = cfg(false);
        assertEquals(c, c);
    }

    @Test void equals_two_equal_instances() {
        assertEquals(cfg(false), cfg(false));
    }

    @Test void not_equals_different_realm() {
        AstarteConfig a = new AstarteConfig("realm-a", DEV_ID, SECRET, PAIRING, false);
        AstarteConfig b = new AstarteConfig("realm-b", DEV_ID, SECRET, PAIRING, false);
        assertNotEquals(a, b);
    }

    @Test void not_equals_different_device_id() {
        AstarteConfig a = new AstarteConfig(REALM, "dev-a", SECRET, PAIRING, false);
        AstarteConfig b = new AstarteConfig(REALM, "dev-b", SECRET, PAIRING, false);
        assertNotEquals(a, b);
    }

    @Test void not_equals_different_secret() {
        AstarteConfig a = new AstarteConfig(REALM, DEV_ID, "secret-1", PAIRING, false);
        AstarteConfig b = new AstarteConfig(REALM, DEV_ID, "secret-2", PAIRING, false);
        assertNotEquals(a, b);
    }

    @Test void not_equals_different_pairing_url() {
        AstarteConfig a = new AstarteConfig(REALM, DEV_ID, SECRET, "http://host-a/pairing", false);
        AstarteConfig b = new AstarteConfig(REALM, DEV_ID, SECRET, "http://host-b/pairing", false);
        assertNotEquals(a, b);
    }

    @Test void not_equals_different_ignore_ssl() {
        assertNotEquals(cfg(true), cfg(false));
    }

    @Test void not_equals_null() {
        assertNotEquals(null, cfg(false));
    }

    // ── hashCode ─────────────────────────────────────────────────────────────

    @Test void hashcode_consistent_with_equals() {
        assertEquals(cfg(true).hashCode(), cfg(true).hashCode());
    }

    @Test void equal_configs_have_same_hashcode() {
        assertEquals(cfg(false).hashCode(), cfg(false).hashCode());
    }

    // ── toString ─────────────────────────────────────────────────────────────

    @Test void toString_contains_realm() {
        assertTrue(cfg(false).toString().contains(REALM));
    }

    @Test void toString_contains_device_id() {
        assertTrue(cfg(false).toString().contains(DEV_ID));
    }
}
