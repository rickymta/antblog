TripView?[] inputs = [null, new("TN-1", null), new("TN-2", "DR-1"), new("TN-3", "")];
foreach (var trip in inputs) Console.WriteLine(Describe(trip));
if (args.Contains("--verify"))
{
    Check(Describe(null) == "Không tìm thấy chuyến", "Missing trip");
    Check(Describe(new("TN-1", null)) == "Đang tìm tài xế", "Unassigned");
    Check(Describe(new("TN-2", "DR-1")) == "Tài xế: DR-1", "Assigned");
    Check(Describe(new("TN-3", " ")) == "Dữ liệu tài xế không hợp lệ", "Whitespace");
    Check(Describe(new("TN-4", "")) == "Dữ liệu tài xế không hợp lệ", "Empty string");
    Check(DescribeDistance(null) == "Thiếu khoảng cách", "Missing value");
    Check(DescribeDistance(0m) == "Trong miền" && DescribeDistance(100m) == "Trong miền", "Inclusive bounds");
    Check(DescribeDistance(-1m) == "Ngoài miền", "Invalid distance");
    Console.WriteLine("PASS: 8 nullable/pattern checks");
}
static string Describe(TripView? trip) => trip switch
{
    null => "Không tìm thấy chuyến",
    { DriverId: null } => "Đang tìm tài xế",
    { DriverId: var driver } when !string.IsNullOrWhiteSpace(driver) => $"Tài xế: {driver}",
    _ => "Dữ liệu tài xế không hợp lệ"
};
static string DescribeDistance(decimal? distance) => distance switch
{
    null => "Thiếu khoảng cách",
    >= 0m and <= 100m => "Trong miền",
    _ => "Ngoài miền"
};
static void Check(bool ok, string name) { if (!ok) throw new Exception(name); }
sealed record TripView(string Id, string? DriverId);
