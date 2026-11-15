---
slug: architecture-ring-buffer-gps
title: "Ring buffer trong Tới Nơi: giữ các điểm GPS gần nhất của tài xế"
summary: "Theo dõi một bộ đệm vòng khi ghi đè điểm GPS cũ, hiện thực bằng C# trên .NET 10 và chọn chính sách phù hợp cho bản đồ, lịch sử chuyến và thanh toán."
category: architecture
level: intermediate
tags: [architecture, dotnet, ring-buffer, gps, telemetry]
language: vi
---

Chấm vị trí của tài xế D9 vừa nhảy sang con phố kế bên. Để hiểu chuyện gì xảy ra, màn hình điều phối muốn vẽ đoạn đường qua vài điểm GPS gần nhất. Chỉ lưu vị trí cuối thì thiếu ngữ cảnh, còn giữ mọi điểm trong bộ nhớ sẽ khiến lượng dữ liệu tăng theo thời gian tài xế trực tuyến. Một lựa chọn cho backend .NET 10 của Tới Nơi là giữ một cửa sổ ngắn cho từng tài xế bằng *ring buffer* (bộ đệm vòng).

## Bốn ô nhớ đi hết một vòng

Giả sử bộ đệm có bốn ô, đánh số từ 0 đến 3. Mỗi lần nhận một điểm, ta ghi vào ô `next` rồi tính vị trí ghi kế tiếp bằng `(next + 1) % capacity`. Biến `count` đếm số điểm hợp lệ và dừng ở 4. Bảng dưới dùng số thứ tự của điểm GPS thay cho toạ độ.

| Điểm vừa ghi | Nội dung vật lý của bốn ô | Ô ghi kế tiếp | Thứ tự đọc từ cũ tới mới |
|---|---|---:|---|
| 101, 102, 103 | 101, 102, 103, trống | 3 | 101, 102, 103 |
| 104 | 101, 102, 103, 104 | 0 | 101, 102, 103, 104 |
| 105 | 105, 102, 103, 104 | 1 | 102, 103, 104, 105 |

Sau điểm 104, con trỏ đã trở về 0 như lúc khởi tạo. Chính `count` phân biệt hai trạng thái đầy và rỗng. Sau điểm 105, ô 0 chứa điểm mới nhất. Nếu vẽ theo thứ tự vật lý của mảng, bản đồ sẽ nối 105 về 102 rồi mới tới 103. Muốn lấy điểm cũ nhất còn giữ, lùi từ `next` đúng `count` ô theo vòng. [Tài liệu về circular buffer của Linux](https://www.kernel.org/doc/html/latest/core-api/circular-buffers.html)

## Một cửa sổ GPS bằng C# trên .NET 10

Ví dụ dưới đây dùng chính sách ghi đè điểm cũ nhất. Mỗi instance thuộc về một tài xế trong một phiên gửi GPS. Đầu vào là các điểm đã được kiểm tra số thứ tự. Phương thức `Snapshot` trả một bản sao để bên vẽ bản đồ dùng tiếp sau khi nhả khoá.

```csharp
public readonly record struct GpsPoint(long Sequence, double Lat, double Lon);

public sealed class RecentGpsPoints
{
    private readonly GpsPoint[] points;
    private readonly object gate = new();
    private int next;
    private int count;

    public RecentGpsPoints(int capacity)
    {
        ArgumentOutOfRangeException.ThrowIfNegativeOrZero(capacity);
        points = new GpsPoint[capacity];
    }

    public void Add(GpsPoint point)
    {
        lock (gate)
        {
            points[next] = point;
            next = (next + 1) % points.Length;
            if (count < points.Length) count++;
        }
    }

    public GpsPoint[] Snapshot()
    {
        lock (gate)
        {
            var result = new GpsPoint[count];
            int oldest = count == points.Length ? next : 0;
            for (int i = 0; i < count; i++)
                result[i] = points[(oldest + i) % points.Length];
            return result;
        }
    }
}
```

Ưu điểm của cách này là lượt ghi có chi phí O(1) trong vùng khoá, dùng lại mảng cũ và không phải dịch các phần tử. Đổi lại, mỗi lần đọc toàn bộ cửa sổ phải sao chép O(n) điểm và giữ khoá suốt lúc sao chép. Nếu Dispatch chỉ hỏi “D9 đang ở đâu?”, một bản ghi vị trí mới nhất sẽ đơn giản và rẻ hơn. Khoá trong ví dụ bảo vệ dữ liệu trong một instance. Khi chạy nhiều replica, cần định tuyến dữ liệu cùng tài xế về cùng worker hoặc đọc cửa sổ từ một kho dùng chung.

## Chọn dung lượng và quyết định bỏ gì khi đầy

Giả sử điện thoại gửi một điểm mỗi ba giây. Giữ 20 điểm cho ta khoảng 57 giây từ điểm đầu đến điểm cuối nếu nhịp gửi đều. Khi thiết bị đổi tần suất hoặc gửi bù sau mất mạng, khoảng thời gian đó cũng đổi. Nếu giao diện yêu cầu đúng một phút, cần lưu thời điểm đo trên mỗi điểm và lọc theo thời gian. Với 12.000 tài xế, dung lượng 20 tương ứng 240.000 ô dữ liệu, chưa kể đối tượng quản lý và bản sao khi đọc. Cần thu hồi buffer của phiên đã kết thúc để giới hạn tổng bộ nhớ.

| Ngữ cảnh tại Tới Nơi | Chính sách phù hợp | Hệ quả cần chấp nhận |
|---|---|---|
| Vẽ đoạn đường gần nhất của từng tài xế | Ghi đè điểm cũ nhất trong buffer của chính tài xế đó | Mất điểm cũ khi đầy và mất cả cửa sổ nếu worker khởi động lại |
| Chờ worker gửi một lô telemetry đi xử lý | Dùng hàng đợi có giới hạn. Khi đầy, cho bên gửi chờ nếu nguồn chịu được áp lực ngược, hoặc bỏ mẫu theo quy tắc rõ ràng | Chờ làm tăng độ trễ. Bỏ mẫu làm giảm độ chi tiết. Ring buffer tự nó chưa cung cấp cơ chế chờ hay đánh thức worker |
| Lịch sử chuyến dùng để đối soát, sự kiện nhận chuyến hoặc thu tiền | Ghi bền vững trước khi xác nhận đã nhận. Bộ đệm trong RAM chỉ phục vụ tăng tốc | Ghi bền vững tốn thêm I/O, nhưng hệ thống có thể phục hồi dữ liệu đã xác nhận |

Một điểm gửi bù có số thứ tự 98 có thể đến sau 105. Nếu cứ gọi `Add`, điểm 98 sẽ đẩy một điểm mới hơn ra ngoài và tạo đoạn đường đi ngược. Với màn hình trực tiếp, hãy xử lý tuần tự theo tài xế, bỏ điểm trùng hoặc cũ trong cùng phiên trước khi ghi. Với kho lịch sử, điểm đến muộn vẫn có thể cần được lưu theo thời điểm đo. [Cách Tới Nơi tách luồng GPS trong giờ cao điểm](/articles/architecture-gio-cao-diem-gps)

## Kiểm tra trước khi dùng cho bản đồ

Dùng dung lượng 4 và lần lượt ghi các điểm từ 101 đến 110. Snapshot ban đầu phải rỗng, sau điểm 104 phải có đủ bốn điểm, sau điểm 110 phải đúng thứ tự 107–110. Giữ lại snapshot đầu tiên rồi ghi thêm để kiểm tra bản sao cũ vẫn ổn định. Cuối cùng, cho D9 gửi dồn nhiều điểm trong khi D12 chỉ gửi một điểm. Điểm của D12 phải còn nguyên. Phép thử cuối phát hiện ngay lỗi dùng chung một vòng cho cả đội xe, dù lớp bộ đệm riêng lẻ đã vượt qua mọi test case.
