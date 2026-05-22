using System;
using System.Runtime.InteropServices;
using AstarteDeviceSdkBindings;
using Xunit;

namespace AstarteSdk.Tests;

/// <summary>
/// Round-trip tests: encode a value with WireWriter, then decode with WireReader
/// and verify the result equals the original. No native library required.
/// </summary>
public class WireCodecTests
{
    private static AstarteVal ValRoundTrip(AstarteVal val)
    {
        using var writer = new WireWriter(val.WireEncodedSize());
        val.WireEncodeTo(writer);
        byte[] bytes = writer.ToArray();
        GCHandle pin = GCHandle.Alloc(bytes, GCHandleType.Pinned);
        try
        {
            var reader = new WireReader(pin.AddrOfPinnedObject(), (UIntPtr)bytes.Length);
            return AstarteVal.Decode(reader);
        }
        finally { pin.Free(); }
    }

    // ── AstarteVal round-trips ─────────────────────────────────────────────

    [Fact] public void Double_RoundTrip() =>
        Assert.Equal(new AstarteVal.Double(3.14), ValRoundTrip(new AstarteVal.Double(3.14)));

    [Fact] public void Integer_RoundTrip() =>
        Assert.Equal(new AstarteVal.Integer(-7), ValRoundTrip(new AstarteVal.Integer(-7)));

    [Fact] public void Boolean_True_RoundTrip() =>
        Assert.Equal(new AstarteVal.Boolean(true), ValRoundTrip(new AstarteVal.Boolean(true)));

    [Fact] public void Boolean_False_RoundTrip() =>
        Assert.Equal(new AstarteVal.Boolean(false), ValRoundTrip(new AstarteVal.Boolean(false)));

    [Fact] public void LongInteger_RoundTrip() =>
        Assert.Equal(new AstarteVal.LongInteger(long.MinValue), ValRoundTrip(new AstarteVal.LongInteger(long.MinValue)));

    [Fact] public void IString_RoundTrip() =>
        Assert.Equal(new AstarteVal.IString("hello, 世界"), ValRoundTrip(new AstarteVal.IString("hello, 世界")));

    [Fact] public void IString_Empty_RoundTrip() =>
        Assert.Equal(new AstarteVal.IString(""), ValRoundTrip(new AstarteVal.IString("")));

    [Fact] public void BinaryBlob_RoundTrip()
    {
        var data = new byte[] { 0xDE, 0xAD, 0xBE, 0xEF };
        var decoded = Assert.IsType<AstarteVal.BinaryBlob>(ValRoundTrip(new AstarteVal.BinaryBlob(data)));
        Assert.Equal(data, decoded.Value);
    }

    [Fact] public void BinaryBlob_Empty_RoundTrip()
    {
        var decoded = Assert.IsType<AstarteVal.BinaryBlob>(ValRoundTrip(new AstarteVal.BinaryBlob(Array.Empty<byte>())));
        Assert.Empty(decoded.Value);
    }

    [Fact] public void DateTime_RoundTrip() =>
        Assert.Equal(new AstarteVal.DateTime(1_700_000_000_000L), ValRoundTrip(new AstarteVal.DateTime(1_700_000_000_000L)));

    [Fact] public void DoubleArray_RoundTrip()
    {
        var data = new double[] { 1.0, 2.5, -3.14 };
        var decoded = Assert.IsType<AstarteVal.DoubleArray>(ValRoundTrip(new AstarteVal.DoubleArray(data)));
        Assert.Equal(data, decoded.Value);
    }

    [Fact] public void DoubleArray_Empty_RoundTrip()
    {
        var decoded = Assert.IsType<AstarteVal.DoubleArray>(ValRoundTrip(new AstarteVal.DoubleArray(Array.Empty<double>())));
        Assert.Empty(decoded.Value);
    }

    [Fact] public void IntegerArray_RoundTrip()
    {
        var data = new int[] { 1, 2, 3 };
        var decoded = Assert.IsType<AstarteVal.IntegerArray>(ValRoundTrip(new AstarteVal.IntegerArray(data)));
        Assert.Equal(data, decoded.Value);
    }

    [Fact] public void BooleanArray_RoundTrip()
    {
        var data = new bool[] { true, false, true };
        var decoded = Assert.IsType<AstarteVal.BooleanArray>(ValRoundTrip(new AstarteVal.BooleanArray(data)));
        Assert.Equal(data, decoded.Value);
    }

    [Fact] public void LongIntegerArray_RoundTrip()
    {
        var data = new long[] { 100L, -200L };
        var decoded = Assert.IsType<AstarteVal.LongIntegerArray>(ValRoundTrip(new AstarteVal.LongIntegerArray(data)));
        Assert.Equal(data, decoded.Value);
    }

    [Fact] public void StringArray_RoundTrip()
    {
        var data = new string[] { "a", "b", "c" };
        var decoded = Assert.IsType<AstarteVal.StringArray>(ValRoundTrip(new AstarteVal.StringArray(data)));
        Assert.Equal(data, decoded.Value);
    }

    [Fact] public void DateTimeArray_RoundTrip()
    {
        var data = new long[] { 0L, 1000L, -1000L };
        var decoded = Assert.IsType<AstarteVal.DateTimeArray>(ValRoundTrip(new AstarteVal.DateTimeArray(data)));
        Assert.Equal(data, decoded.Value);
    }

    // ── AstarteConfig round-trip ──────────────────────────────────────────

    [Fact] public void Config_RoundTrip()
    {
        var original = new AstarteConfig("realm", "device-id", "secret", "http://host/pairing", true);
        using var writer = new WireWriter(original.WireEncodedSize());
        original.WireEncodeTo(writer);
        byte[] bytes = writer.ToArray();
        GCHandle pin = GCHandle.Alloc(bytes, GCHandleType.Pinned);
        AstarteConfig decoded;
        try
        {
            var reader = new WireReader(pin.AddrOfPinnedObject(), (UIntPtr)bytes.Length);
            decoded = AstarteConfig.Decode(reader);
        }
        finally { pin.Free(); }
        Assert.Equal(original, decoded);
    }

    // ── ObjectEntry round-trip ────────────────────────────────────────────

    [Fact] public void ObjectEntry_RoundTrip()
    {
        var entries = new ObjectEntry[]
        {
            new ObjectEntry("count",  new AstarteVal.Integer(7)),
            new ObjectEntry("label",  new AstarteVal.IString("hi")),
            new ObjectEntry("active", new AstarteVal.Boolean(true)),
        };
        int totalSize = 4;
        foreach (var e in entries) totalSize += e.WireEncodedSize();
        using var writer = new WireWriter(totalSize);
        writer.WriteI32(entries.Length);
        foreach (var e in entries) e.WireEncodeTo(writer);
        byte[] bytes = writer.ToArray();
        GCHandle pin = GCHandle.Alloc(bytes, GCHandleType.Pinned);
        ObjectEntry[] decoded;
        try
        {
            var reader = new WireReader(pin.AddrOfPinnedObject(), (UIntPtr)bytes.Length);
            decoded = reader.ReadEncodedArray<ObjectEntry>(r => ObjectEntry.Decode(r));
        }
        finally { pin.Free(); }

        Assert.Equal(entries.Length, decoded.Length);
        for (int i = 0; i < entries.Length; i++)
        {
            Assert.Equal(entries[i].Key,   decoded[i].Key);
            Assert.Equal(entries[i].Value, decoded[i].Value);
        }
    }
}
