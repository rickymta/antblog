---
slug: azure-saga-outbox-service-bus-postgresql
title: "Saga và Outbox trên Service Bus và PostgreSQL"
summary: "Bài 53: điều phối luồng đặt chuyến Booking, Payment, Dispatch của Tới Nơi bằng orchestrator Durable Functions qua Service Bus, với outbox/inbox trong PostgreSQL, compensation khi Payment lỗi và bảng kịch bản kiểm thử."
category: devops-azure
level: advanced
tags: [azure, cloud, devops, toi-noi, saga, outbox, service-bus, postgresql]
language: vi
---

Đây là **bài 53** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 6 · Kiến trúc nâng cao và vận hành (bài 4/9 của phần).

Bài này nối ba thứ đã học riêng lẻ: Outbox/Inbox ở [bài 26 loạt C#](/articles/dotnet10-outbox-inbox-idempotent-consumer), Service Bus session và dead-letter ở [bài 34](/articles/azure-service-bus-queue-topic-session-dead-letter), Durable Functions ở [bài 25](/articles/azure-functions-trigger-binding-durable). Kết quả là một luồng đặt chuyến xuyên ba service mà không cần transaction phân tán. Lệnh và code trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Payment bị lỗi giữa chừng thì orchestrator có thể «rollback» việc Booking đã làm bằng cách hoàn tác transaction không?</p>
<details><summary>Xem đáp án</summary><p>Không. Transaction của Booking đã commit từ lâu và nằm trong database khác. Cách duy nhất là gửi thêm một lệnh ngược (compensation), tức một thay đổi mới ghi đè ý nghĩa của thay đổi cũ.</p></details>
</div>

## Không có transaction phân tán, nên đặt chuyến là chuỗi bước đảo ngược được

Một chuyến đi của Tới Nơi chạm ba service, mỗi service có PostgreSQL riêng. Booking tạo chuyến ở trạng thái `PENDING`. Payment giữ tiền của khách. Dispatch gán tài xế. Không có lệnh `COMMIT` nào bao được cả ba database, và kiến trúc tổng thể đã cấm service chọc vào database của service khác. Vì vậy mỗi bước là một transaction local, và mỗi bước đã làm phải có một bước ngược để gọi khi các bước sau thất bại.

Đó là **Saga**: chuỗi transaction local nối bằng message, kèm compensation. Saga không cho bạn tính cô lập. Giữa hai bước, thế giới nhìn thấy trạng thái dở dang: tiền đã giữ nhưng chưa có tài xế. Thiết kế phải chấp nhận điều đó và làm nó an toàn.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hình 1. Ba bước Booking, Payment, Dispatch do orchestrator Durable Functions điều phối; bước thất bại kích hoạt compensation theo thứ tự ngược."><defs><marker id="az53-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="250" height="80" rx="10" class="f-pk-soft s-pk"/><text x="34" y="52" class="t-b">1. Booking</text><text x="34" y="78" class="t-small">Trip PENDING + outbox</text><path d="M270 60 L315 60" class="f-none s-phys" stroke-width="2" marker-end="url(#az53-0-a)"/><rect x="315" y="20" width="250" height="80" rx="10" class="f-phys-soft s-phys"/><text x="329" y="52" class="t-b">2. Payment</text><text x="329" y="78" class="t-small">Giữ tiền của khách</text><path d="M565 60 L610 60" class="f-none s-phys" stroke-width="2" marker-end="url(#az53-0-a)"/><rect x="610" y="20" width="250" height="80" rx="10" class="f-phys-soft s-phys"/><text x="624" y="52" class="t-b">3. Dispatch</text><text x="624" y="78" class="t-small">Gán tài xế</text><path d="M145 100 L145 170" class="f-none s-phys" stroke-width="2" marker-end="url(#az53-0-a)"/><path d="M440 100 L440 170" class="f-none s-phys" stroke-width="2" marker-end="url(#az53-0-a)"/><path d="M735 100 L735 170" class="f-none s-warn" stroke-width="2" marker-end="url(#az53-0-a)"/><rect x="20" y="170" width="545" height="90" rx="10" class="f-pk-soft s-pk"/><text x="34" y="203" class="t-b">Orchestrator (Durable Functions)</text><text x="34" y="229" class="t-small">Gửi lệnh, chờ kết quả, timeout</text><rect x="610" y="170" width="250" height="90" rx="10" class="f-warn-soft s-warn"/><text x="624" y="203" class="t-b">Compensation</text><text x="624" y="229" class="t-small">Hoàn tiền, huỷ chuyến</text></svg>
</div>
<figcaption>Hình 1. Ba bước Booking, Payment, Dispatch do orchestrator Durable Functions điều phối; bước thất bại kích hoạt compensation theo thứ tự ngược.</figcaption>
</figure>

## Orchestration thắng khi có timeout và nhiều nhánh hỏng

Có hai cách nối các bước. **Choreography**: mỗi service nghe event của service trước và tự quyết bước tiếp. **Orchestration**: một thành phần giữ sơ đồ luồng và ra lệnh cho từng service.

| Tiêu chí | Choreography | Orchestration |
| --- | --- | --- |
| Ai biết trạng thái cả chuyến | Không ai; phải ghép từ log | Orchestrator, xem được theo instance |
| Timeout «Payment không trả lời» | Mỗi service phải tự đặt timer | Một timer trong orchestrator |
| Compensation | Rải ra nhiều service, dễ thiếu một nhánh | Một chỗ, đi ngược theo ngăn xếp |
| Thêm bước thứ tư | Sửa nhiều subscriber | Sửa một orchestrator |
| Rủi ro | Vòng lặp event khó thấy | Orchestrator thành nơi chứa luật nghiệp vụ |

Tài liệu tổng thể gợi ý chỉ dùng orchestrator cho quy trình quan trọng như booking, không cho mọi tương tác nhỏ. Với ba bước, timeout và hai nhánh hỏng, orchestration đáng giá. Rủi ro ở dòng cuối bảng vẫn thật: orchestrator chỉ nên biết thứ tự và điều kiện, còn luật nghiệp vụ nằm trong service. Phương án thay thế là một service riêng giữ state machine trong bảng `saga_state` của PostgreSQL, đổi lại tự viết timer, khoá và phục hồi sau crash. Orchestrator dưới đây chạy trên Durable Functions (bài 25), isolated worker; lệnh đi qua ba queue có session, kết quả quay về qua một subscription.

```csharp
public sealed record Cmd(string TripId, string Queue, string Name);

public static class BookTripSaga
{
    [Function(nameof(BookTripSaga))]
    public static async Task<string> Run([OrchestrationTrigger] TaskOrchestrationContext ctx)
    {
        var tripId = ctx.GetInput<string>()!;
        var undo = new Stack<Cmd>();
        undo.Push(new(tripId, "booking-commands", "CancelTrip"));
        undo.Push(new(tripId, "payment-commands", "RefundPayment"));
        if (!await Step(ctx, new(tripId, "payment-commands", "AuthorizePayment")))
            return await Compensate(ctx, undo, "PaymentFailed");
        undo.Push(new(tripId, "dispatch-commands", "ReleaseDriver"));
        if (!await Step(ctx, new(tripId, "dispatch-commands", "AssignDriver")))
            return await Compensate(ctx, undo, "DispatchFailed");
        await ctx.CallActivityAsync(nameof(Activities.Send), new Cmd(tripId, "booking-commands", "ConfirmTrip"));
        return "Confirmed";
    }
    static async Task<bool> Step(TaskOrchestrationContext ctx, Cmd cmd)
    {
        await ctx.CallActivityAsync(nameof(Activities.Send), cmd);
        using var cts = new CancellationTokenSource();
        var reply = ctx.WaitForExternalEvent<string>(cmd.Name + "Result");
        var timeout = ctx.CreateTimer(ctx.CurrentUtcDateTime.AddMinutes(2), cts.Token);
        if (await Task.WhenAny(reply, timeout) != reply) return false;
        cts.Cancel();
        return reply.Result == "ok";
    }
    static async Task<string> Compensate(TaskOrchestrationContext ctx, Stack<Cmd> undo, string why)
    {
        while (undo.Count > 0) await ctx.CallActivityAsync(nameof(Activities.Send), undo.Pop());
        return why;
    }
}
```

Khối này cho thấy hình dạng của pattern, dùng API `TaskOrchestrationContext` của gói `Microsoft.Azure.Functions.Worker.Extensions.DurableTask`. Nó không chứa activity `Send` (gửi một `ServiceBusMessage` tới queue đã chọn), không chứa hàm chuyển reply thành `RaiseEventAsync`, và chưa được biên dịch cùng project thật.

Ba chi tiết trong đoạn code đáng đọc kỹ. Thứ nhất, lệnh ngược được đẩy vào ngăn xếp **trước** khi gửi lệnh thuận. Nếu orchestrator hết giờ chờ Payment, bạn không biết Payment đã giữ tiền hay chưa; gửi `RefundPayment` vô điều kiện là an toàn nếu Payment xử lý nó idempotent. Thứ hai, mọi lệnh mang `SessionId = tripId` và đi cùng một queue của service đó, nên Payment luôn nhận `AuthorizePayment` trước `RefundPayment`. Thứ ba, activity `Send` đặt `MessageId = {tripId}:{tên lệnh}` cố định, không sinh Guid mới. Activity có thể bị chạy lại, và ID ổn định cho phép duplicate detection loại bản gửi trùng. Duplicate detection xét `MessageId` trong cả entity, không phụ thuộc session; riêng entity partitioned thì phạm vi là từng partition, mà partition lại chọn theo `SessionId`, nên cùng chuyến vẫn được che.

Timer hết hạn không có nghĩa lệnh bị huỷ. Nó chỉ nghĩa orchestrator ngừng chờ. Mọi service nhận lệnh phải coi đó là at-least-once và có thể đến muộn.

## Outbox ghi cùng transaction, relay giữ MessageId, inbox chặn hiệu ứng lặp

Các service phản hồi orchestrator bằng event, và mỗi event phải rời khỏi database của service đó mà không để lọt khoảng trống dual-write: commit xong mà publish hỏng. Cách xử lý ở bài 26 vẫn nguyên: service ghi hiệu ứng nghiệp vụ và một dòng `outbox` trong cùng transaction. Relay đọc dòng chưa gửi, đẩy lên Service Bus rồi mới đánh dấu đã gửi. Bài này thêm một ràng buộc: `MessageId` của message **bằng chính id của dòng outbox**.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 200" role="img" aria-label="Hình 2. Outbox trong PostgreSQL, relay gửi với MessageId bằng outbox id, Service Bus loại trùng trong cửa sổ, inbox của consumer chặn hiệu ứng lặp."><defs><marker id="az53-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="55" width="190" height="90" rx="10" class="f-pk-soft s-pk"/><text x="34" y="88" class="t-b">Outbox</text><text x="34" y="114" class="t-small">Cùng commit với dữ liệu</text><path d="M210 100 L240 100" class="f-none s-phys" stroke-width="2" marker-end="url(#az53-1-a)"/><rect x="240" y="55" width="190" height="90" rx="10" class="f-phys-soft s-phys"/><text x="254" y="88" class="t-b">Relay</text><text x="254" y="114" class="t-small">MessageId = outbox id</text><path d="M430 100 L460 100" class="f-none s-phys" stroke-width="2" marker-end="url(#az53-1-a)"/><rect x="460" y="55" width="190" height="90" rx="10" class="f-phys-soft s-phys"/><text x="474" y="88" class="t-b">Service Bus</text><text x="474" y="114" class="t-small">Loại trùng trong cửa sổ</text><path d="M650 100 L680 100" class="f-none s-phys" stroke-width="2" marker-end="url(#az53-1-a)"/><rect x="680" y="55" width="180" height="90" rx="10" class="f-pk-soft s-pk"/><text x="694" y="88" class="t-b">Inbox</text><text x="694" y="114" class="t-small">Unique theo message id</text></svg>
</div>
<figcaption>Hình 2. Outbox trong PostgreSQL, relay gửi với MessageId bằng outbox id, Service Bus loại trùng trong cửa sổ, inbox của consumer chặn hiệu ứng lặp.</figcaption>
</figure>

Relay dưới đây đọc một lô bằng `FOR UPDATE SKIP LOCKED`, nên chạy nhiều bản relay không đọc trùng cùng một dòng. Việc giữ transaction suốt lúc gọi mạng chỉ hợp với lô nhỏ; lô lớn cần thiết kế lease riêng.

```csharp
public sealed class OutboxRelay(NpgsqlDataSource db, ServiceBusSender sender)
{
    public async Task<int> RunOnceAsync(CancellationToken ct)
    {
        await using var conn = await db.OpenConnectionAsync(ct);
        await using var tx = await conn.BeginTransactionAsync(ct);
        var rows = new List<(Guid Id, Guid TripId, string Type, string Json)>();
        await using (var select = new NpgsqlCommand(
            "select id, trip_id, event_type, payload from outbox where published_at is null " +
            "order by created_at limit 50 for update skip locked", conn, tx))
        await using (var r = await select.ExecuteReaderAsync(ct))
            while (await r.ReadAsync(ct))
                rows.Add((r.GetGuid(0), r.GetGuid(1), r.GetString(2), r.GetString(3)));
        foreach (var row in rows)
        {
            var msg = new ServiceBusMessage(row.Json)
            {
                MessageId = row.Id.ToString(), SessionId = row.TripId.ToString(), Subject = row.Type,
            };
            msg.ApplicationProperties["eventType"] = row.Type;
            await sender.SendMessageAsync(msg, ct);
            await using var mark = new NpgsqlCommand(
                "update outbox set published_at = now() where id = @id", conn, tx);
            mark.Parameters.AddWithValue("id", row.Id);
            await mark.ExecuteNonQueryAsync(ct);
        }
        await tx.CommitAsync(ct);
        return rows.Count;
    }
}
```

Relay crash sau `SendMessageAsync` nhưng trước `CommitAsync` thì dòng vẫn chưa gửi, và lần chạy sau gửi lại cùng `MessageId`. Duplicate detection của topic loại bản thứ hai nếu nó rơi trong cửa sổ bạn đặt. Ngoài cửa sổ, hoặc ở hạ nguồn không có duplicate detection, chỉ **inbox** còn bảo vệ. Consumer ghi `(consumer, message_id)` với ràng buộc unique cùng hiệu ứng nghiệp vụ trong một transaction; message trùng làm unique vi phạm và hiệu ứng không lặp (cách kiểm đúng loại lỗi 23505 ở bài 26). Hai lớp này bổ sung nhau, không lớp nào thay được lớp kia.

## Idempotency key và trạng thái pending là một phần của sản phẩm

Saga mất vài giây tới vài phút, nên `POST /trips` không thể chờ tới khi có tài xế. Booking trả `202 Accepted` kèm trạng thái `PENDING`, và client phải xử lý điều đó như một trạng thái thật chứ không phải ngoại lệ.

Mạng di động hay đứt, app tự gửi lại. Vì thế client gửi header `Idempotency-Key` (một Guid sinh khi người dùng bấm «Đặt xe»), và Booking lưu bản ghi unique theo `(customer_id, key)` cùng hash nội dung request trong cùng transaction tạo chuyến:

- cùng key, cùng nội dung: trả lại chuyến đã tạo, không tạo chuyến thứ hai;
- cùng key, nội dung khác: trả lỗi xung đột, vì đó là lỗi của client chứ không phải retry;
- key mới: chuyến mới.

Về UX, app hiển thị «Đang tìm tài xế» khi `PENDING`, và tuyệt đối không hiện «Đặt xe thành công» khi chưa `CONFIRMED`. Nếu Payment lỗi, trạng thái chuyển `CANCELLED` kèm lý do, và app phải nói điều đó bằng lời của người dùng («Thanh toán không thành công, chuyến đã huỷ»). Đây là eventual consistency nhìn thấy được: Booking đọc từ chính database của nó nên `GET /trips/{id}` thấy `PENDING` ngay sau `POST`, rồi thay đổi khi Saga tiến triển. Trạng thái đẩy tới app qua kênh realtime đã nói ở [bài 52](/articles/azure-kien-truc-tham-chieu-microservices-toi-noi).

## Kiểm thử bằng cách phá từng chỗ của chuỗi

Một Saga chưa thử khi bị phá thì mới là bản vẽ. Bảng dưới liệt kê kịch bản, cách gây ra và điều cơ chế dự đoán. Tất cả là kỳ vọng theo thiết kế, không phải kết quả đã đo.

| Kịch bản | Cách gây ra | Kỳ vọng theo cơ chế | Quan sát ở đâu |
| --- | --- | --- | --- |
| Tắt Payment giữa chừng | Deactivate revision của Payment trước khi gửi `POST /trips` | Lệnh nằm trong queue; sau 2 phút orchestrator hết giờ chờ, gửi `RefundPayment` và `CancelTrip`; chuyến `CANCELLED`. Bật lại Payment: nó xử lý `AuthorizePayment` rồi `RefundPayment` theo thứ tự session, tiền về trạng thái đã hoàn | Bảng `trips` và `payments`; trạng thái instance Durable |
| Payment trả lỗi | Cho Payment trả `failed` cho `AuthorizePayment` | Compensation chạy ngay, không chờ timer | Orchestrator, `cancel_reason` của chuyến |
| Replay message | Gửi lại message từ dead-letter hoặc đặt lại `published_at` của một dòng outbox | Inbox thấy `message_id` đã có và bỏ qua; không hiệu ứng thứ hai | Bảng inbox, số dòng `payments` của chuyến |
| Message trùng | Dừng relay giữa `SendMessageAsync` và commit, rồi chạy lại | Cùng `MessageId` bị duplicate detection loại nếu trong cửa sổ; nếu lọt, inbox chặn | Metric message của queue, bảng inbox |
| Poison command | Gửi lệnh không parse được | Qua số lần giao tối đa thì vào dead-letter; orchestrator hết giờ và chạy compensation | Dead-letter queue, alert |
| Host orchestrator khởi động lại | Restart Function app giữa lúc chờ | Replay lại từ lịch sử, không bắt đầu lại; lệnh đã gửi không gửi lại với ID mới | Lịch sử instance Durable |
| Client gửi hai lần cùng key | Hai `POST /trips` cùng `Idempotency-Key` | Một chuyến, hai phản hồi giống nhau | Bảng `trips`, bảng idempotency |

Kịch bản «Poison command» hay bị bỏ quên nhất, dù nó khiến chuyến kẹt trong session: một message hỏng chặn các message sau của cùng chuyến cho tới khi nó vào dead-letter, đúng như đã nói ở bài 34. Alert trên dead-letter không phải tuỳ chọn; một message ở đó là một khách đang chờ.

## Lab: dựng ba queue, cố ý làm Payment lỗi và đọc kết quả trong PostgreSQL

Lab dùng lại namespace `sb-toinoi-dev-sea` (tier Standard, đủ cho session và duplicate detection) và topic `trip-events` của bài 34; nếu đã xoá sau bài 34, dựng lại bằng khối lệnh ở đó, vì duplicate detection chỉ bật được lúc tạo entity. Khối đầu tạo ba queue lệnh có session và duplicate detection, rồi một subscription nhận kết quả để một function chuyển thành `RaiseEventAsync` vào đúng instance (instance ID = `tripId`). Tên cờ CLI có thể khác giữa các phiên bản; kiểm bằng `--help` nếu lệnh báo lỗi.

```bash
RG=rg-toinoi-msg-dev-sea; NS=sb-toinoi-dev-sea
for Q in booking-commands payment-commands dispatch-commands; do
  az servicebus queue create -g $RG --namespace-name $NS -n $Q \
    --enable-session true --max-delivery-count 5 \
    --enable-duplicate-detection true \
    --duplicate-detection-history-time-window PT10M
done
az servicebus topic subscription create -g $RG --namespace-name $NS \
  --topic-name trip-events -n saga-replies --max-delivery-count 10
az servicebus topic subscription rule delete -g $RG --namespace-name $NS \
  --topic-name trip-events --subscription-name saga-replies -n '$Default'
az servicebus topic subscription rule create -g $RG --namespace-name $NS \
  --topic-name trip-events --subscription-name saga-replies -n results-only \
  --filter-type SqlFilter --filter-sql-expression "eventType LIKE '%Result'"
```

Khối này chỉ tạo entity. Nó chưa chứng minh quyền của Function app trên Service Bus đã được cấp (gán vai trò dữ liệu cho managed identity của app, như bài 34 làm cho danh tính của bạn), và cũng chưa có service nào chạy.

Sau khi triển khai Booking, Payment, Dispatch và Function app chứa orchestrator, lab đi ba bước: deactivate revision của Payment để lệnh `AuthorizePayment` nằm chờ trong queue; gửi `POST /trips` với một `Idempotency-Key`; đợi quá hai phút rồi đọc trạng thái trong PostgreSQL. Nếu app chạy single revision mode và lệnh deactivate bị từ chối, chuyển sang multiple revision mode trước. Tên bảng và cột là minh hoạ.

```bash
az containerapp revision deactivate -g rg-toinoi-payment-dev-sea \
  -n ca-toinoi-payment-dev --revision <revision-name>

psql "$BOOKING_PG" -c "select id, status, cancel_reason from trips where id = '<trip-id>'"
psql "$PAYMENT_PG" -c "select trip_id, status from payments where trip_id = '<trip-id>'"
psql "$BOOKING_PG" -c "select count(*) from outbox where published_at is null"
```

Kỳ vọng theo cơ chế: chuyến chuyển sang `CANCELLED` sau khoảng hai phút, không còn dòng outbox treo. Khi `az containerapp revision activate` bật Payment trở lại, `payments` của chuyến kết thúc ở trạng thái đã hoàn, vì `AuthorizePayment` rồi `RefundPayment` được xử lý đúng thứ tự trong session. Nếu `payments` còn ở trạng thái đã giữ tiền, compensation chưa đủ. Không có bước nào ở đây đã được chạy để lấy số liệu.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Compensation không phải undo: khách có thể đã nhận thông báo «đã giữ tiền» trước khi nhận «chuyến bị huỷ». Mỗi lệnh ngược phải idempotent và thực sự chạy được ở mọi trạng thái, kể cả khi lệnh thuận chưa từng thành công. Orchestrator còn có chi phí: lịch sử mỗi instance nằm trong storage và tính tiền theo giao dịch, và sửa code orchestrator khi còn instance đang chạy có thể làm replay lệch. Xoá resource sau lab: <code>az group delete -n rg-toinoi-msg-dev-sea --yes</code>.</p>
</div>

**Bài tập:** Payment trả lời «ok» cho `AuthorizePayment` đúng lúc timer hai phút vừa hết. Orchestrator đã chọn nhánh hết giờ và gửi `RefundPayment`. Chuyện gì xảy ra với tiền của khách, và hai cơ chế nào trong bài bảo đảm nó cuối cùng đúng?

<details><summary>Xem lời giải</summary><p>Payment đã giữ tiền, rồi nhận <code>RefundPayment</code> và hoàn lại, nên khách không mất tiền. Hai cơ chế: session theo <code>tripId</code> giữ thứ tự để refund đến sau authorize, và lệnh ngược idempotent chạy đúng dù reply đến muộn hay không đến. Reply «ok» muộn gửi về orchestrator bị bỏ qua vì instance đã kết thúc; chuyến vẫn kết thúc ở <code>CANCELLED</code>.</p></details>

## Học tiếp từ đây

Bạn vừa có một Saga ba bước với compensation theo ngăn xếp, outbox và inbox giữ phần của từng service, idempotency key cho `POST /trips` và bảng kịch bản để phá từng chỗ của chuỗi. Phần chưa được chứng minh là hành vi thật khi tải cao, nhiều relay và nhiều instance cùng chạy. Bài sau, [bài 54](/articles/azure-toi-uu-chi-phi-reservation-savings-plan-finops), chuyển sang chi phí: reservation, savings plan, spot và cách theo dõi cho cả những thành phần như Service Bus, orchestrator và Firewall vừa xuất hiện.

## Tài liệu gốc

- [Durable Functions overview](https://learn.microsoft.com/en-us/azure/azure-functions/durable/durable-functions-overview)
- [Azure Service Bus message sessions](https://learn.microsoft.com/en-us/azure/service-bus-messaging/message-sessions)
- [Service Bus duplicate detection](https://learn.microsoft.com/en-us/azure/service-bus-messaging/duplicate-detection)
- [Saga distributed transactions pattern](https://learn.microsoft.com/en-us/azure/architecture/patterns/saga)
- [Azure Database for PostgreSQL documentation](https://learn.microsoft.com/en-us/azure/postgresql/)
