using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;

namespace ToiNoi.FeeClient;

// Mã kết quả: cùng giá trị với FeeStatus trong fee_ffi.h (int32).
internal enum FeeStatus : int
{
    Ok = 0,
    NullPointer = 1,
    InvalidArgument = 2,
    InvalidUtf8 = 3,
    BadHandle = 4,
    Panic = 99,
}

// Bố cục khớp struct FeeSplit của C: ba int64 liền nhau, 24 byte. Sequential là mặc định, ghi rõ cho dễ đọc.
[StructLayout(LayoutKind.Sequential)]
internal struct FeeSplit
{
    public long FareDong;
    public long PlatformFeeDong;
    public long DriverNetDong;
}

// Handle tới FeeQuote nằm trên heap của Rust. SafeHandle bảo đảm fee_quote_free chạy đúng một lần:
// qua Dispose, hoặc qua finalizer nếu quên Dispose.
internal sealed class QuoteHandle : SafeHandleZeroOrMinusOneIsInvalid
{
    public QuoteHandle() : base(ownsHandle: true) { }

    protected override bool ReleaseHandle() => Native.fee_quote_free(handle) == FeeStatus.Ok;
}

internal static partial class Native
{
    private const string Lib = "fee_ffi";

    [LibraryImport(Lib)]
    internal static partial uint fee_abi_version();

    [LibraryImport(Lib)]
    internal static partial ulong fee_live_allocations();

    // out FeeSplit: struct blittable, con trỏ tới biến cục bộ được ghim trong lúc gọi.
    [LibraryImport(Lib)]
    internal static partial FeeStatus fee_split(long fareDong, out FeeSplit result);

    // StringMarshalling.Utf8: .NET đổi string UTF-16 sang byte UTF-8 kết thúc NUL cho `const char *`.
    [LibraryImport(Lib, StringMarshalling = StringMarshalling.Utf8)]
    internal static partial QuoteHandle fee_quote_new(string? tripId, uint km, out FeeStatus status);

    // Cố ý sai: cùng hàm nhưng marshal UTF-16. Rust đọc `const char *` và gặp NUL sau chữ cái đầu.
    [LibraryImport(Lib, EntryPoint = "fee_quote_new", StringMarshalling = StringMarshalling.Utf16)]
    internal static partial QuoteHandle fee_quote_new_utf16(string? tripId, uint km, out FeeStatus status);

    [LibraryImport(Lib)]
    internal static partial FeeStatus fee_quote_split(QuoteHandle quote, out FeeSplit result);

    // Trả IntPtr, không trả string: chuỗi do Rust cấp, .NET không được tự giải phóng nó.
    [LibraryImport(Lib)]
    internal static partial IntPtr fee_quote_describe(QuoteHandle quote);

    [LibraryImport(Lib)]
    internal static partial FeeStatus fee_string_free(IntPtr text);

    // Bản IntPtr dành cho ReleaseHandle và cho thí nghiệm double free thủ công.
    [LibraryImport(Lib)]
    internal static partial FeeStatus fee_quote_free(IntPtr quote);
}
