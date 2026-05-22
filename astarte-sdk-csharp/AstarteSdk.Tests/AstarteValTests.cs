using AstarteDeviceSdkBindings;
using Xunit;

namespace AstarteSdk.Tests;

public class AstarteValTests
{
    // ── Double ───────────────────────────────────────────────────────────────

    [Fact] public void Double_Field() => Assert.Equal(3.14, new AstarteVal.Double(3.14).Value);

    [Fact] public void Double_EqualsSameValue() =>
        Assert.Equal(new AstarteVal.Double(1.0), new AstarteVal.Double(1.0));

    [Fact] public void Double_NotEqualsDifferentValue() =>
        Assert.NotEqual(new AstarteVal.Double(1.0), new AstarteVal.Double(2.0));

    [Fact] public void Double_HashcodeConsistent() =>
        Assert.Equal(new AstarteVal.Double(1.0).GetHashCode(), new AstarteVal.Double(1.0).GetHashCode());

    [Fact] public void Double_ToStringContainsName() =>
        Assert.Contains("Double", new AstarteVal.Double(1.0).ToString());

    // ── Integer ─────────────────────────────────────────────────────────────

    [Fact] public void Integer_Field() => Assert.Equal(42, new AstarteVal.Integer(42).Value);

    [Fact] public void Integer_EqualsSameValue() =>
        Assert.Equal(new AstarteVal.Integer(42), new AstarteVal.Integer(42));

    [Fact] public void Integer_NotEqualsDifferentValue() =>
        Assert.NotEqual(new AstarteVal.Integer(1), new AstarteVal.Integer(2));

    [Fact] public void Integer_HashcodeConsistent() =>
        Assert.Equal(new AstarteVal.Integer(7).GetHashCode(), new AstarteVal.Integer(7).GetHashCode());

    [Fact] public void Integer_ToStringContainsName() =>
        Assert.Contains("Integer", new AstarteVal.Integer(0).ToString());

    // ── Boolean ──────────────────────────────────────────────────────────────

    [Fact] public void Boolean_FieldTrue() => Assert.True(new AstarteVal.Boolean(true).Value);
    [Fact] public void Boolean_FieldFalse() => Assert.False(new AstarteVal.Boolean(false).Value);

    [Fact] public void Boolean_EqualsSameValue() =>
        Assert.Equal(new AstarteVal.Boolean(true), new AstarteVal.Boolean(true));

    [Fact] public void Boolean_NotEqualsDifferentValue() =>
        Assert.NotEqual(new AstarteVal.Boolean(true), new AstarteVal.Boolean(false));

    [Fact] public void Boolean_HashcodeConsistent() =>
        Assert.Equal(new AstarteVal.Boolean(false).GetHashCode(), new AstarteVal.Boolean(false).GetHashCode());

    // ── LongInteger ──────────────────────────────────────────────────────────

    [Fact] public void LongInteger_Field() => Assert.Equal(long.MaxValue, new AstarteVal.LongInteger(long.MaxValue).Value);

    [Fact] public void LongInteger_EqualsSameValue() =>
        Assert.Equal(new AstarteVal.LongInteger(100L), new AstarteVal.LongInteger(100L));

    [Fact] public void LongInteger_NotEqualsDifferentValue() =>
        Assert.NotEqual(new AstarteVal.LongInteger(1L), new AstarteVal.LongInteger(2L));

    [Fact] public void LongInteger_HashcodeConsistent() =>
        Assert.Equal(new AstarteVal.LongInteger(0L).GetHashCode(), new AstarteVal.LongInteger(0L).GetHashCode());

    // ── IString ──────────────────────────────────────────────────────────────

    [Fact] public void IString_Field() => Assert.Equal("hello", new AstarteVal.IString("hello").Value);

    [Fact] public void IString_EqualsSameValue() =>
        Assert.Equal(new AstarteVal.IString("abc"), new AstarteVal.IString("abc"));

    [Fact] public void IString_NotEqualsDifferentValue() =>
        Assert.NotEqual(new AstarteVal.IString("a"), new AstarteVal.IString("b"));

    [Fact] public void IString_HashcodeConsistent() =>
        Assert.Equal(new AstarteVal.IString("x").GetHashCode(), new AstarteVal.IString("x").GetHashCode());

    // ── BinaryBlob ───────────────────────────────────────────────────────────
    // Note: C# records use reference equality for array fields, so BinaryBlob
    // instances with the same bytes are not record-equal. Compare .Value content.

    [Fact] public void BinaryBlob_FieldContent() =>
        Assert.Equal(new byte[] { 0x01, 0x02, 0x03 }, new AstarteVal.BinaryBlob(new byte[] { 0x01, 0x02, 0x03 }).Value);

    [Fact] public void BinaryBlob_SameContent()
    {
        var arr = new byte[] { 1, 2 };
        var v = new AstarteVal.BinaryBlob(arr);
        Assert.Equal(arr, v.Value);
    }

    [Fact] public void BinaryBlob_DifferentContent_NotEqual() =>
        Assert.NotEqual(new byte[] { 1 }, new AstarteVal.BinaryBlob(new byte[] { 2 }).Value);

    [Fact] public void BinaryBlob_HashcodeConsistent()
    {
        var v = new AstarteVal.BinaryBlob(new byte[] { 9 });
        Assert.Equal(v.GetHashCode(), v.GetHashCode());
    }

    // ── DateTime ─────────────────────────────────────────────────────────────

    [Fact] public void DateTime_Field() => Assert.Equal(1_700_000_000_000L, new AstarteVal.DateTime(1_700_000_000_000L).Value);

    [Fact] public void DateTime_EqualsSameValue() =>
        Assert.Equal(new AstarteVal.DateTime(0L), new AstarteVal.DateTime(0L));

    [Fact] public void DateTime_NotEqualsDifferentValue() =>
        Assert.NotEqual(new AstarteVal.DateTime(1L), new AstarteVal.DateTime(2L));

    // ── DoubleArray ──────────────────────────────────────────────────────────

    [Fact] public void DoubleArray_FieldContent() =>
        Assert.Equal(new double[] { 1.0, 2.0, 3.0 }, new AstarteVal.DoubleArray(new double[] { 1.0, 2.0, 3.0 }).Value);

    [Fact] public void DoubleArray_SameContent()
    {
        var arr = new double[] { 1.0, 2.0 };
        Assert.Equal(arr, new AstarteVal.DoubleArray(arr).Value);
    }

    [Fact] public void DoubleArray_DifferentContent_NotEqual() =>
        Assert.NotEqual(new double[] { 1.0 }, new AstarteVal.DoubleArray(new double[] { 2.0 }).Value);

    // ── IntegerArray ─────────────────────────────────────────────────────────

    [Fact] public void IntegerArray_FieldContent() =>
        Assert.Equal(new int[] { 10, 20, 30 }, new AstarteVal.IntegerArray(new int[] { 10, 20, 30 }).Value);

    [Fact] public void IntegerArray_SameContent()
    {
        var arr = new int[] { 1, 2 };
        Assert.Equal(arr, new AstarteVal.IntegerArray(arr).Value);
    }

    [Fact] public void IntegerArray_DifferentContent_NotEqual() =>
        Assert.NotEqual(new int[] { 1 }, new AstarteVal.IntegerArray(new int[] { 9 }).Value);

    // ── BooleanArray ─────────────────────────────────────────────────────────

    [Fact] public void BooleanArray_FieldContent() =>
        Assert.Equal(new bool[] { true, false, true }, new AstarteVal.BooleanArray(new bool[] { true, false, true }).Value);

    [Fact] public void BooleanArray_SameContent()
    {
        var arr = new bool[] { true, false };
        Assert.Equal(arr, new AstarteVal.BooleanArray(arr).Value);
    }

    [Fact] public void BooleanArray_DifferentContent_NotEqual() =>
        Assert.NotEqual(new bool[] { true }, new AstarteVal.BooleanArray(new bool[] { false }).Value);

    // ── LongIntegerArray ─────────────────────────────────────────────────────

    [Fact] public void LongIntegerArray_FieldContent() =>
        Assert.Equal(new long[] { 100L, 200L }, new AstarteVal.LongIntegerArray(new long[] { 100L, 200L }).Value);

    [Fact] public void LongIntegerArray_SameContent()
    {
        var arr = new long[] { 1L, 2L };
        Assert.Equal(arr, new AstarteVal.LongIntegerArray(arr).Value);
    }

    // ── StringArray ──────────────────────────────────────────────────────────

    [Fact] public void StringArray_FieldContent() =>
        Assert.Equal(new string[] { "a", "b", "c" }, new AstarteVal.StringArray(new string[] { "a", "b", "c" }).Value);

    [Fact] public void StringArray_SameContent()
    {
        var arr = new string[] { "x", "y" };
        Assert.Equal(arr, new AstarteVal.StringArray(arr).Value);
    }

    [Fact] public void StringArray_DifferentContent_NotEqual() =>
        Assert.NotEqual(new string[] { "a" }, new AstarteVal.StringArray(new string[] { "b" }).Value);

    // ── DateTimeArray ────────────────────────────────────────────────────────

    [Fact] public void DateTimeArray_FieldContent() =>
        Assert.Equal(new long[] { 1000L, 2000L }, new AstarteVal.DateTimeArray(new long[] { 1000L, 2000L }).Value);

    [Fact] public void DateTimeArray_SameContent()
    {
        var arr = new long[] { 1L, 2L };
        Assert.Equal(arr, new AstarteVal.DateTimeArray(arr).Value);
    }

    // ── Cross-type inequality ────────────────────────────────────────────────

    [Fact] public void DifferentVariants_AreNotEqual() =>
        Assert.NotEqual<AstarteVal>(new AstarteVal.Integer(1), new AstarteVal.Double(1.0));

    [Fact] public void Variant_NotEqualsNull() =>
        Assert.False(new AstarteVal.Integer(0).Equals((object?)null));

    [Fact] public void SameInstance_EqualsSelf()
    {
        AstarteVal v = new AstarteVal.Integer(5);
        Assert.Equal(v, v);
    }
}
