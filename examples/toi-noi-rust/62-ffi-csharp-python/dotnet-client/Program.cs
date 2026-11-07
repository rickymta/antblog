using System.Reflection;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using ToiNoi.FeeClient;

// Đường dẫn thư viện do tiến trình cha (cargo run -- --with-dotnet) đưa qua FEE_FFI_LIB.
var libPath = Environment.GetEnvironmentVariable("FEE_FFI_LIB")
    ?? throw new InvalidOperationException("Thiếu biến FEE_FFI_LIB (đường dẫn libfee_ffi).");
NativeLibrary.SetDllImportResolver(
    Assembly.GetExecutingAssembly(),
    (name, _, _) => name == "fee_ffi" ? NativeLibrary.Load(libPath) : IntPtr.Zero);

try
{
    Run();
    Console.WriteLine("DOTNET OK");
}
catch (Exception e)
{
    Console.Error.WriteLine($"FAIL: {e.Message}");
    return 1;
}
return 0;

static void Check(bool condition, string what)
{
    if (!condition) throw new InvalidOperationException($"sai: {what}");
    Console.WriteLine($"  ok  {what}");
}

static string Describe(QuoteHandle quote)
{
    IntPtr text = Native.fee_quote_describe(quote);
    if (text == IntPtr.Zero) throw new InvalidOperationException("describe trả null");
    try
    {
        return Marshal.PtrToStringUTF8(text)!;
    }
    finally
    {
        // Chuỗi do Rust cấp thì trả cho Rust, không dùng Marshal.FreeHGlobal hay FreeCoTaskMem.
        Native.fee_string_free(text);
    }
}

// Tạo handle rồi bỏ quên, không Dispose. NoInlining để JIT không giữ biến sống tới hết hàm gọi.
[MethodImpl(MethodImplOptions.NoInlining)]
static void ForgetHandles(int count)
{
    for (int i = 0; i < count; i++)
    {
        _ = Native.fee_quote_new("TN-0001", 5, out _);
    }
}

static void Run()
{
    Console.WriteLine("== 1. Chữ ký, struct ==");
    Check(Native.fee_abi_version() == 1, "fee_abi_version() = 1");
    Check(Marshal.SizeOf<FeeSplit>() == 24, "FeeSplit: 24 byte, khớp C");
    var status = Native.fee_split(25_500, out FeeSplit split);
    Check(status == FeeStatus.Ok, "fee_split(25500) = Ok");
    Check(split.FareDong == 25_500 && split.PlatformFeeDong == 5_100 && split.DriverNetDong == 20_400,
        "struct nhận về: cước 25.500, phí 5.100, tài xế 20.400");
    Check(Native.fee_split(-1, out _) == FeeStatus.InvalidArgument, "cước âm -> InvalidArgument (mã lỗi, không exception)");

    Console.WriteLine("== 2. Handle và chuỗi UTF-8 ==");
    ulong before = Native.fee_live_allocations();
    IntPtr raw;
    using (QuoteHandle quote = Native.fee_quote_new("TN-0001", 5, out status))
    {
        Check(status == FeeStatus.Ok && !quote.IsInvalid, "fee_quote_new trả handle hợp lệ");
        Check(Native.fee_live_allocations() == before + 1, "Rust đang giữ một báo giá");
        string shown = Describe(quote);
        Console.WriteLine($"      {shown}");
        Check(shown == "TN-0001: 5 km, cước 25.500 VND, phí nền tảng 5.100 VND, tài xế nhận 20.400 VND",
            "chuỗi do Rust cấp, đọc bằng PtrToStringUTF8, trả lại bằng fee_string_free");
        Check(Native.fee_quote_split(quote, out FeeSplit fromHandle) == FeeStatus.Ok && fromHandle.DriverNetDong == 20_400,
            "fee_quote_split qua handle");
        raw = quote.DangerousGetHandle();
    }
    Check(Native.fee_live_allocations() == before, "hết khối using: Dispose đã gọi fee_quote_free");

    using (QuoteHandle vn = Native.fee_quote_new("TN-Đà Nẵng-01", 5, out _))
    {
        Check(Describe(vn).StartsWith("TN-Đà Nẵng-01: 5 km", StringComparison.Ordinal), "mã chuyến có dấu đi vòng qua UTF-8 nguyên vẹn");
    }
    using (QuoteHandle wrong = Native.fee_quote_new_utf16("TN-0001", 5, out _))
    {
        string shown = Describe(wrong);
        Console.WriteLine($"      marshal nhầm UTF-16: {shown}");
        Check(shown.StartsWith("T: 5 km", StringComparison.Ordinal), "marshal UTF-16 vào `const char *`: Rust chỉ thấy chữ «T» (không crash, dữ liệu sai)");
    }

    Console.WriteLine("== 3. Double free và dùng sau free ==");
    Check(Native.fee_quote_free(raw) == FeeStatus.BadHandle, "free thủ công lần hai trên con trỏ đã giải phóng -> BadHandle");
    QuoteHandle twice = Native.fee_quote_new("TN-0001", 5, out _);
    twice.Dispose();
    twice.Dispose();
    Check(Native.fee_live_allocations() == before, "Dispose hai lần: fee_quote_free chỉ chạy một lần");
    bool disposedThrown = false;
    try { _ = Native.fee_quote_split(twice, out _); }
    catch (ObjectDisposedException) { disposedThrown = true; }
    Check(disposedThrown, "dùng handle đã Dispose -> ObjectDisposedException ở phía .NET, không gọi sang Rust");

    Console.WriteLine("== 4. Lỗi đầu vào ==");
    using (QuoteHandle none = Native.fee_quote_new(null, 5, out status))
    {
        Check(none.IsInvalid && status == FeeStatus.NullPointer, "trip_id = null -> handle không hợp lệ, NullPointer");
    }
    using (QuoteHandle zero = Native.fee_quote_new("TN-0001", 0, out status))
    {
        Check(zero.IsInvalid && status == FeeStatus.InvalidArgument, "0 km -> InvalidArgument");
    }

    Console.WriteLine("== 5. Quên Dispose: finalizer của SafeHandle ==");
    ForgetHandles(1_000);
    for (int i = 0; i < 20 && Native.fee_live_allocations() != before; i++)
    {
        GC.Collect();
        GC.WaitForPendingFinalizers();
    }
    Check(Native.fee_live_allocations() == before, "1.000 handle bị bỏ quên đều được finalizer trả về cho Rust");
}
