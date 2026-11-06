var label = new DisplayLabel { Text = "  Tới Nơi  " };
Check(label.Text == "Tới Nơi", "field setter normalized value");
try { label.Text = " "; throw new Exception("Whitespace accepted."); } catch (ArgumentException) { }
Check(label.Text == "Tới Nơi", "Invalid set leaves previous value");
var quote = new QuoteView(120000m, DateTimeOffset.UnixEpoch.AddMinutes(5));
Check(quote.IsHighFare, "Extension property");
Check(quote.IsExpiredAt(DateTimeOffset.UnixEpoch.AddMinutes(5)), "Expiry boundary");
int evaluations = 0;
string ComputeLabel() { evaluations++; return "New label"; }
DisplayLabel? missing = null; missing?.Text = ComputeLabel();
Check(evaluations == 0, "Conditional assignment skips RHS for null");
label?.Text = ComputeLabel(); Check(evaluations == 1 && label is { Text: "New label" }, "Conditional assignment for non-null");
Console.WriteLine("PASS: 7 C# 14 checks");
static void Check(bool ok, string name) { if (!ok) throw new Exception(name); }
sealed class DisplayLabel
{
    public string Text
    {
        get;
        set
        {
            if (string.IsNullOrWhiteSpace(value)) throw new ArgumentException("Label required.");
            field = value.Trim();
        }
    } = "Tới Nơi";
}
sealed record QuoteView(decimal Fare, DateTimeOffset ExpiresAt);
static class QuoteExtensions
{
    extension(QuoteView quote)
    {
        public bool IsHighFare => quote.Fare >= 100000m;
        public bool IsExpiredAt(DateTimeOffset now) => now >= quote.ExpiresAt;
    }
}
