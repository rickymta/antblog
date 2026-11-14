---
slug: location-go-goroutine-channel-nhap-mon-cho-ingest
title: "Go đủ dùng cho ingest: goroutine, channel, context và race detector"
summary: "Bài 11: Go tối thiểu để đọc location-ingest, đối chiếu tokio và Rust: goroutine, channel có đệm, select, context, errgroup; tái hiện một data race làm mất hơn nửa số đếm rồi để cờ -race phát hiện."
category: location-routing
level: intermediate
tags: [toi-noi, location, go, goroutine, channel, context, errgroup, race-detector]
language: vi
---

Lúc cao điểm, 25.000 tài xế của Tới Nơi mở kết nối giữ suốt ca và mỗi 4 giây gửi một điểm GPS, tổng cộng khoảng 6.250 điểm mỗi giây. Phía nhận cần giữ ngần ấy kết nối, kiểm tra từng điểm, đẩy vào hàng đợi rồi trả ack, và không được kẹt khi Kafka chậm. Đây là việc của `location-ingest`, và ở loạt này dịch vụ đó được viết bằng Go.

Blog chưa có loạt Go riêng, nên bài này dạy Go vừa đủ để đọc và sửa được `location-ingest` và `tracking-hub` ở các bài sau. Giả định bạn đã biết chút Rust: chỗ nào Go khác Rust thì bài nói thẳng. Phần cuối bài tái hiện một lỗi mà mọi người viết Go đều gặp, data race, để thấy nó trông ra sao khi không có công cụ phát hiện.

Đây là bài 11 của lộ trình **Nền tảng định vị** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Lý do service này viết bằng Go chứ không phải Rust nằm ở [bài ADR](/articles/location-adr-rust-go-python-csharp-ranh-gioi-service); bài này không nhắc lại lập luận đó, chỉ dùng kết luận của nó.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Tám goroutine chạy song song, mỗi goroutine thực hiện <code>count++</code> một trăm nghìn lần trên cùng một biến <code>int</code>. Kết quả mong đợi là 800.000. Chạy thật sẽ ra gì, và chương trình có báo lỗi nào không?</p>
<details><summary>Xem đáp án</summary><p><b>Không đoán được, và không có lỗi nào.</b> Trong lần chạy của lab, kết quả là 270.093, mất 529.907 lần cộng, chương trình vẫn thoát bình thường. Chạy lại sẽ ra số khác. Chỉ khi thêm cờ <code>-race</code> thì Go mới in <code>WARNING: DATA RACE</code> và thoát với mã 66.</p></details>
</div>

## Go xuất hiện ở đâu trong service định vị

Hai chương trình Go của loạt: `location-ingest` nhận lô GPS từ app tài xế qua WebSocket, và `tracking-hub` đẩy vị trí tài xế tới khách. Cả hai có chung một hình dạng: **rất nhiều kết nối nhàn rỗi, mỗi kết nối làm một việc nhỏ lặp đi lặp lại, và giữa chúng là hàng đợi có giới hạn**. Mất một điểm GPS không làm lệch một đồng tiền nào (kiến trúc §10 xếp vị trí vào telemetry tạm thời), nên cái giá phải đánh đổi của Go (có bộ gom rác, không kiểm tra data race lúc biên dịch) là chấp nhận được ở đây. Thuật toán nặng trên đường nóng, như bám đường và định tuyến, vẫn ở Rust.

Mô hình của Go cho loại bài toán này là **mỗi kết nối một goroutine** và viết code theo kiểu tuần tự, chặn trực tiếp: đọc một khung, xử lý, ghi ack, lặp lại. Runtime của Go lo việc đưa hàng chục nghìn goroutine lên vài chục luồng hệ điều hành. Với tokio bạn cũng làm được điều tương tự, nhưng phải đánh dấu `async` ở mọi hàm trên đường gọi và cẩn thận không chặn runtime; Go không phân biệt hàm `async` với hàm thường.

| Việc | Go | Rust (tokio) |
| --- | --- | --- |
| Chạy một việc song song | `go f()` | `tokio::spawn(f())` |
| Hàng đợi giữa hai việc | `make(chan T, 64)` | `mpsc::channel::<T>(64)` |
| Chờ nhiều sự kiện | `select` | `tokio::select!` |
| Huỷ cả cây việc | `context.Context` | `CancellationToken` (tokio-util), hoặc drop future |
| Chờ một nhóm xong | `sync.WaitGroup` | `JoinSet` |
| Nhóm có lỗi và huỷ lan | `errgroup` | `JoinSet` cộng tự huỷ |
| Dọn dẹp cuối hàm | `defer` | `Drop` (gắn theo biến, không theo hàm) |
| Lỗi | giá trị `error` trả về | `Result<T, E>` |
| An toàn dữ liệu chung | quy ước, kiểm lúc chạy bằng `-race` | `Send`/`Sync`, kiểm lúc biên dịch |

Hàng cuối là điểm khác biệt quan trọng nhất và là lý do bài này có một mục riêng về race detector. Safe Rust từ chối biên dịch chương trình truy cập dữ liệu chung mà không đồng bộ hoá; Go biên dịch mọi thứ và chỉ có công cụ kiểm tra lúc chạy. Phía C#, cùng ý tưởng nằm ở [bài cancellation](/articles/csharp-async-await-cancellation) (`CancellationToken` đóng vai `context`) và [bài concurrency](/articles/csharp-concurrency-thread-safety) (check-then-act, `lock`).

## Goroutine và channel qua một pipeline nhỏ

Lab của bài là một pipeline ba stage, chạy trên dữ liệu do simulator sinh (200 tài xế × 50 điểm, seed 11):

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Pipeline ba stage nối bằng channel có đệm 64: một goroutine sinh điểm, bốn worker kiểm tra, một goroutine gom theo thành phố; errgroup huỷ context dùng chung khi một stage lỗi hoặc hết hạn, stage 1 và 2 nghe context đó, stage 3 dừng khi channel đóng."><defs><marker id="location-go-goroutine-channel-nhap-mon-cho-ingest-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="170" height="100" rx="8" class="f-phys-soft s-phys"/><text x="105" y="52" class="t-b" text-anchor="middle">Stage 1 · sinh điểm</text><text x="105" y="70" text-anchor="middle">1 goroutine</text><text x="105" y="88" class="t-small" text-anchor="middle">200 tài xế × 50 điểm</text><rect x="222" y="55" width="96" height="50" rx="8" class="f-surface s-muted"/><text x="270" y="75" class="t-mono" text-anchor="middle">ch1</text><text x="270" y="91" class="t-small" text-anchor="middle">đệm 64</text><rect x="350" y="30" width="190" height="100" rx="8" class="f-phys-soft s-phys"/><text x="445" y="52" class="t-b" text-anchor="middle">Stage 2 · kiểm tra</text><text x="445" y="70" text-anchor="middle">4 goroutine worker</text><text x="445" y="88" class="t-small" text-anchor="middle">đọc ch1, ghi ch2</text><rect x="572" y="55" width="96" height="50" rx="8" class="f-surface s-muted"/><text x="620" y="75" class="t-mono" text-anchor="middle">ch2</text><text x="620" y="91" class="t-small" text-anchor="middle">đệm 64</text><rect x="700" y="30" width="160" height="100" rx="8" class="f-phys-soft s-phys"/><text x="780" y="52" class="t-b" text-anchor="middle">Stage 3 · gom</text><text x="780" y="70" text-anchor="middle">1 goroutine</text><text x="780" y="88" class="t-small" text-anchor="middle">sở hữu map, không khoá</text><path d="M190 80 L220 80" class="f-none s-phys" stroke-width="2" marker-end="url(#location-go-goroutine-channel-nhap-mon-cho-ingest-1-a)"/><path d="M318 80 L348 80" class="f-none s-phys" stroke-width="2" marker-end="url(#location-go-goroutine-channel-nhap-mon-cho-ingest-1-a)"/><path d="M540 80 L570 80" class="f-none s-phys" stroke-width="2" marker-end="url(#location-go-goroutine-channel-nhap-mon-cho-ingest-1-a)"/><path d="M668 80 L698 80" class="f-none s-phys" stroke-width="2" marker-end="url(#location-go-goroutine-channel-nhap-mon-cho-ingest-1-a)"/><rect x="20" y="190" width="840" height="70" rx="8" class="f-warn-soft s-warn"/><text x="440" y="216" class="t-b" text-anchor="middle">errgroup.WithContext tạo ctx dùng chung; stage 1 và 2 nghe nó trong select</text><text x="440" y="234" class="t-small" text-anchor="middle">Một stage trả lỗi hoặc hết hạn thì ctx.Done() đóng; stage 1 và 2 thoát, ch1 rồi ch2 đóng, nên stage 3 hết vòng range</text><path d="M105 190 L105 134" class="f-none s-warn dash" stroke-width="2" marker-end="url(#location-go-goroutine-channel-nhap-mon-cho-ingest-1-a)"/><path d="M445 190 L445 134" class="f-none s-warn dash" stroke-width="2" marker-end="url(#location-go-goroutine-channel-nhap-mon-cho-ingest-1-a)"/><text x="440" y="286" class="t-mono" text-anchor="middle">select { case ch &lt;- ev: ...  case &lt;-ctx.Done(): return ctx.Err() }</text></svg>
</div>

<figcaption>Hình 1. Ba stage nối bằng hai channel có đệm 64. Stage 1 và 2 nghe chung một <code>ctx</code>; stage 3 chỉ dừng khi <code>ch2</code> đóng. Nhờ vậy một lỗi ở đâu đó vẫn dừng được cả cây.</figcaption>
</figure>

Stage 1 sinh điểm. Có hai chỗ đáng đọc: `defer close(ch1)` và `select`.

Tệp `examples/toi-noi-location/go/cmd/loc11_nhap_mon/pipeline.go` (dòng 39–65):

```go
	g, ctx := errgroup.WithContext(ctx)
	ch1 := make(chan *pb.DriverLocationUpdated, 64)
	ch2 := make(chan *pb.DriverLocationUpdated, 64)
	var res result
	res.perCity = map[string]int{}
	t0 := time.Now()

	// Stage 1: 200 tài xế × 50 điểm, có seed.
	g.Go(func() error {
		defer close(ch1)
		fleet := sim.NewFleet(200, 11)
		for step := 0; step < 50; step++ {
			for _, d := range fleet {
				ev := &pb.DriverLocationUpdated{DriverId: d.ID, CityCode: d.City, Point: d.Step(4, int64(step)*4000+1)}
				select {
				case ch1 <- ev:
					atomic.AddInt64(&res.generated, 1)
					if timeoutMs > 0 {
						time.Sleep(20 * time.Microsecond) // làm chậm một chút để hết hạn xảy ra giữa chừng
					}
				case <-ctx.Done():
					return ctx.Err()
				}
			}
		}
		return nil
	})
```

Vài điểm cần nhớ khi đọc đoạn trên nếu bạn đến từ Rust:

- **`go func() { ... }()`** (ở đây qua `g.Go`) khởi chạy một goroutine; không có `JoinHandle` trả về, nên việc chờ phải dùng `WaitGroup` hoặc `errgroup`.
- **Channel có đệm 64** là hàng đợi giới hạn: khi đầy, `ch1 <- ev` chặn goroutine gửi. Đây chính là backpressure ở mức nhỏ nhất, giống `mpsc::channel(64)` gọi `send().await`. Bài 13 sẽ nói vì sao ở cổng vào của ingest ta **không** muốn chặn kiểu này.
- **Người gửi đóng channel**, không phải người nhận. `defer close(ch1)` chạy khi hàm kết thúc dù bằng `return nil` hay `return ctx.Err()`. Gửi vào channel đã đóng là panic, nên mỗi channel chỉ nên có một bên đóng.
- **`select`** chờ nhiều khả năng cùng lúc. Ở đây: hoặc gửi được `ev`, hoặc `ctx.Done()` đóng. Thiếu nhánh thứ hai thì khi stage sau đã thoát, stage này kẹt vĩnh viễn ở `ch1 <- ev` (goroutine bị rò, không ai thu hồi).
- **Biến vòng lặp** `step`, `d` được cấp mới cho mỗi vòng từ Go 1.22 (module của lab khai báo `go 1.27.1`), nên closure bên trong an toàn khi chạy sau vòng lặp. Trước 1.22 đây là một cái bẫy kinh điển.

Stage 2 và 3 cho thấy hai cách sở hữu dữ liệu khác nhau:

Tệp `examples/toi-noi-location/go/cmd/loc11_nhap_mon/pipeline.go` (dòng 69–103):

```go
	var wg sync.WaitGroup
	var seen atomic.Int64
	cfg := validate.Default()
	for w := 0; w < 4; w++ {
		wg.Add(1)
		g.Go(func() error {
			defer wg.Done()
			for ev := range ch1 {
				if n := seen.Add(1); failAt >= 0 && int(n) == failAt {
					return errors.New("điểm hỏng ở stage kiểm tra")
				}
				var st validate.State // chỉ kiểm hình thức từng điểm, không so với điểm trước
				if v := cfg.Check(&st, ev.Point, 0, ev.Point.DeviceTimeMs); v.Reason != validate.Accepted {
					continue
				}
				atomic.AddInt64(&res.validated, 1)
				select {
				case ch2 <- ev:
				case <-ctx.Done():
					return ctx.Err()
				}
			}
			return nil
		})
	}
	g.Go(func() error { wg.Wait(); close(ch2); return nil }) // đóng ch2 khi mọi worker xong

	// Stage 3: một goroutine sở hữu map, không cần khoá.
	g.Go(func() error {
		for ev := range ch2 {
			res.perCity[ev.CityCode]++
			res.aggregated++
		}
		return nil
	})
```

Bốn worker cùng đọc `ch1` (`for ev := range ch1` tự thoát khi `ch1` đóng và cạn). Các bộ đếm `seen`, `res.validated` dùng `atomic` vì nhiều goroutine cùng cộng. Còn `res.perCity`, một `map` bình thường, **không có khoá** vì chỉ goroutine ở stage 3 đụng vào nó: dữ liệu có một chủ duy nhất, các goroutine khác gửi thông điệp tới goroutine sở hữu đó qua channel. Đây là cách mà Go khuyên (chia sẻ bộ nhớ bằng cách giao tiếp, đừng giao tiếp bằng cách chia sẻ bộ nhớ), và `location-ingest` dùng đúng kiểu này: trạng thái kiểm tra của mỗi tài xế sống trong biến cục bộ của goroutine giữ kết nối đó (bài 12).

Một chi tiết khác với Rust: gửi con trỏ `ev` qua channel **không chuyển quyền sở hữu**. Sau `ch2 <- ev`, worker vẫn còn cầm con trỏ và trình biên dịch không cấm worker ghi tiếp. Quy ước "gửi xong là không đụng nữa" chỉ nằm trong đầu người viết, và đó là loại lỗi mà mục race detector bên dưới sinh ra để bắt.

Dòng `g.Go(func() error { wg.Wait(); close(ch2); return nil })` ở giữa đáng để ý. `ch2` có bốn người gửi, nên không worker nào được tự đóng nó; một goroutine riêng chờ cả bốn xong rồi mới đóng. Đóng sớm thì worker còn lại gửi vào channel đã đóng và panic.

## context và errgroup: dừng cả cây việc

`context.Context` mang hai thứ qua mọi hàm: hạn chót và tín hiệu huỷ. Lab chạy cùng pipeline này ba lần để xem ba cách kết thúc. `errgroup.WithContext` trả một nhóm goroutine và một `ctx` mới bị huỷ khi **bất kỳ** goroutine nào trong nhóm trả lỗi; `g.Wait()` chờ tất cả và trả lỗi đầu tiên.

Tệp `examples/toi-noi-location/go/cmd/loc11_nhap_mon/pipeline.go` (dòng 32–44):

```go
func runPipeline(timeoutMs, failAt int) result {
	ctx := context.Background()
	if timeoutMs > 0 {
		var cancel context.CancelFunc
		ctx, cancel = context.WithTimeout(ctx, time.Duration(timeoutMs)*time.Millisecond)
		defer cancel()
	}
	g, ctx := errgroup.WithContext(ctx)
	ch1 := make(chan *pb.DriverLocationUpdated, 64)
	ch2 := make(chan *pb.DriverLocationUpdated, 64)
	var res result
	res.perCity = map[string]int{}
	t0 := time.Now()
```

Ba cách chạy, kết quả chạy thật của lab (chi tiết lệnh ở mục chạy ví dụ):

```text
$ go -C examples/toi-noi-location/go run ./cmd/loc11_nhap_mon pipeline
sinh=10000 kiểm=10000 gom=10000 thành phố=map[CT:400 DN:1150 HCM:4750 HN:2750 HP:500 NT:450]
hoàn tất, không lỗi

$ go -C examples/toi-noi-location/go run ./cmd/loc11_nhap_mon timeout
sinh=1242 kiểm=1242 gom=1242 thành phố=map[CT:48 DN:146 HCM:590 HN:339 HP:63 NT:56]
kết thúc sớm vì context hết hạn: context deadline exceeded

$ go -C examples/toi-noi-location/go run ./cmd/loc11_nhap_mon fail
sinh=729 kiểm=704 gom=701 thành phố=map[CT:27 DN:83 HCM:333 HN:188 HP:38 NT:32]
kết thúc vì lỗi, errgroup huỷ các stage còn lại: điểm hỏng ở stage kiểm tra
```

Hai điều rút ra:

1. **Hết hạn và lỗi đi cùng một đường.** Ở `timeout`, `context.WithTimeout` đóng `ctx.Done()` sau 40 ms; ở `fail`, worker trả lỗi ở điểm thứ 700 và `errgroup` huỷ `ctx`. Trong cả hai, stage 1 thoát qua nhánh `<-ctx.Done()` và `defer close(ch1)` đóng channel; worker thoát ở `select` của mình hoặc khi `ch1` cạn; goroutine chờ `wg.Wait()` đóng `ch2`, và stage 3 hết vòng `range` mà không hề nhìn `ctx`. `g.Wait()` trả lỗi đầu tiên để `report` phân biệt bằng `errors.Is(r.err, context.DeadlineExceeded)`. Không stage nào cần biết lý do dừng.
2. **Số điểm đã qua mỗi stage khi bị huỷ là không xác định.** Lần `fail` có `sinh=729 kiểm=704 gom=701`: 28 điểm đã sinh mà không tới cuối, gồm 25 điểm chưa qua bước kiểm (còn trong `ch1` hoặc trong tay worker) và 3 điểm đã kiểm nhưng rơi vào nhánh `ctx.Done()` thay vì `ch2`. Số này đổi mỗi lần chạy, nên test của lab (`main_test.go`) chỉ khẳng định `generated < 10000` và có lỗi, không khẳng định một con số. Bài viết cũng không được coi số này là hằng số.

So với C# thì `ctx` đóng vai `CancellationToken`, nhưng Go quy ước ctx là tham số đầu tiên của mọi hàm có thể chặn, nên không có hàm nào quên nhận tín hiệu huỷ nếu tuân thủ quy ước. Nhận `ctx` thôi chưa đủ: huỷ trong Go mang tính hợp tác, tức hàm phải tự kiểm tra `ctx.Done()` hoặc gọi hàm khác có kiểm tra. `time.Sleep` hay một lệnh `ch <- ev` trần không dừng giữa chừng.

## Data race: lỗi không báo gì

Giờ tới thí nghiệm của câu hỏi đầu bài. Mỗi goroutine làm `count++` trên cùng một biến:

Tệp `examples/toi-noi-location/go/cmd/loc11_nhap_mon/race.go` (dòng 17–31):

```go
func raceBad() {
	var count int
	var wg sync.WaitGroup
	for i := 0; i < workers; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for j := 0; j < perWorker; j++ {
				count++ // DATA RACE
			}
		}()
	}
	wg.Wait()
	fmt.Printf("race-bad: đếm được %d, đúng phải là %d (mất %d)\n", count, want, want-count)
}
```

`count++` không phải một thao tác mà là ba: đọc `count`, cộng một, ghi lại. Khi hai goroutine xen vào nhau giữa các bước, một lần cộng biến mất (Hình 2).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Mất một lần cộng: goroutine A và B cùng đọc count bằng 5 rồi cùng ghi 6, nên count tăng một thay vì hai."><defs><marker id="location-go-goroutine-channel-nhap-mon-cho-ingest-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="70" class="t-b">goroutine A</text><text x="20" y="150" class="t-b">goroutine B</text><text x="20" y="236" class="t-b">count trong RAM</text><rect x="150" y="40" width="150" height="56" rx="8" class="f-phys-soft s-phys"/><text x="225" y="62" text-anchor="middle">t1 · A đọc count</text><text x="225" y="80" class="t-mono" text-anchor="middle">thấy 5</text><rect x="490" y="40" width="150" height="56" rx="8" class="f-phys-soft s-phys"/><text x="565" y="62" text-anchor="middle">t3 · A ghi 5+1</text><text x="565" y="80" class="t-mono" text-anchor="middle">count = 6</text><rect x="320" y="120" width="150" height="56" rx="8" class="f-pk-soft s-pk"/><text x="395" y="142" text-anchor="middle">t2 · B đọc count</text><text x="395" y="160" class="t-mono" text-anchor="middle">cũng thấy 5</text><rect x="660" y="120" width="150" height="56" rx="8" class="f-pk-soft s-pk"/><text x="735" y="142" text-anchor="middle">t4 · B ghi 5+1</text><text x="735" y="160" class="t-mono" text-anchor="middle">count = 6</text><rect x="150" y="205" width="150" height="56" rx="8" class="f-surface s-muted"/><text x="225" y="227" class="t-small" text-anchor="middle">lúc đầu</text><text x="225" y="245" class="t-mono" text-anchor="middle">count = 5</text><rect x="490" y="205" width="150" height="56" rx="8" class="f-surface s-muted"/><text x="565" y="227" class="t-small" text-anchor="middle">sau t3</text><text x="565" y="245" class="t-mono" text-anchor="middle">count = 6</text><rect x="660" y="205" width="150" height="56" rx="8" class="f-warn-soft s-warn"/><text x="735" y="227" class="t-small" text-anchor="middle">sau t4: đáng lẽ là 7</text><text x="735" y="245" class="t-mono" text-anchor="middle">count = 6</text><path d="M225 205 L225 100" class="f-none s-phys" stroke-width="2" marker-end="url(#location-go-goroutine-channel-nhap-mon-cho-ingest-2-a)"/><path d="M290 205 L345 180" class="f-none s-pk" stroke-width="2" marker-end="url(#location-go-goroutine-channel-nhap-mon-cho-ingest-2-a)"/><path d="M565 100 L565 203" class="f-none s-phys" stroke-width="2" marker-end="url(#location-go-goroutine-channel-nhap-mon-cho-ingest-2-a)"/><path d="M735 180 L735 203" class="f-none s-pk" stroke-width="2" marker-end="url(#location-go-goroutine-channel-nhap-mon-cho-ingest-2-a)"/><path d="M150 284 L800 284" class="f-none s-muted" stroke-width="2" marker-end="url(#location-go-goroutine-channel-nhap-mon-cho-ingest-2-a)"/><text x="810" y="288" class="t-small">thời gian</text></svg>
</div>

<figcaption>Hình 2. Lịch xen kẽ gây mất cập nhật. Hai lần <code>count++</code> chỉ làm <code>count</code> tăng một.</figcaption>
</figure>

Chạy không có `-race`, lab in:

```text
race-bad: đếm được 270093, đúng phải là 800000 (mất 529907)
```

Mất hơn một nửa mà không có lỗi nào, và con số thay đổi mỗi lần chạy: đây là lần chạy duy nhất được ghi lại. Một service ghi bộ đếm kiểu này sẽ báo sai mà không ai biết. Khi thêm `-race`, Go chèn mã theo dõi mọi truy cập bộ nhớ và so sánh thứ tự truy cập giữa các goroutine. Báo cáo (rút gọn đường dẫn thành `...`):

```text
WARNING: DATA RACE
Read at 0x00c0001140b8 by goroutine 10:
  main.raceBad.func1()
      .../cmd/loc11_nhap_mon/race.go:25 +0x88

Previous write at 0x00c0001140b8 by goroutine 14:
  main.raceBad.func1()
      .../cmd/loc11_nhap_mon/race.go:25 +0x98
```

Báo cáo chỉ ra đúng dòng 25, hai goroutine liên quan, và nơi chúng được tạo. Một lần chạy `-race` khác cho `Found 2 data race(s)` (một khối đọc với ghi, một khối ghi với ghi) và `exit status 66`. Chương trình thoát mã 66, còn `go run` bọc ngoài trả mã 1; đằng nào cũng khác không, nên CI thất bại khi có race. Hai lưu ý khi dùng:

- `-race` chỉ báo race **xảy ra trong lần chạy đó**. Đoạn mã mà test không chạy tới thì không được kiểm tra, nên một lần chạy sạch không chứng minh là không có race. Vì vậy lab đặt `-race` cho `go test`, và test của hàng đợi, hub, ingest đều chạy với cờ này.
- `-race` chỉ bắt truy cập bộ nhớ không được đồng bộ hoá. Lỗi logic như kiểm tra rồi hành động qua hai lần khoá riêng, deadlock hay goroutine rò đều nằm ngoài tầm của nó.
- Tài liệu của Go ghi rằng cờ này làm tăng đáng kể bộ nhớ và thời gian chạy, nên bản dựng dùng để đo hiệu năng ở các bài sau **không** bật `-race`; cờ này chỉ dùng cho test và cho lab của bài này.

Ba cách sửa, đều chạy dưới `-race` và đều ra đúng 800.000:

Tệp `examples/toi-noi-location/go/cmd/loc11_nhap_mon/race.go` (dòng 36–46 và 57–73):

```go
	// 1. Mutex.
	var mu sync.Mutex
	var c1 int
	parallel(func() {
		for j := 0; j < perWorker; j++ {
			mu.Lock()
			c1++
			mu.Unlock()
		}
	})
	fmt.Printf("mutex:   đếm được %d, muốn %d\n", c1, want)
// …
	// 3. Channel, một goroutine sở hữu bộ đếm.
	ch := make(chan int, 1024)
	done := make(chan int)
	go func() {
		n := 0
		for v := range ch {
			n += v
		}
		done <- n
	}()
	parallel(func() {
		for j := 0; j < perWorker; j++ {
			ch <- 1
		}
	})
	close(ch)
	fmt.Printf("channel: đếm được %d, muốn %d\n", <-done, want)
```

Mutex bảo vệ vùng tới hạn; `atomic.Int64` làm đúng một phép cộng không thể xen; channel với một goroutine sở hữu bộ đếm là cách thứ ba, và cũng là kiểu của stage 3 ở trên. Chọn thế nào cho ingest? Bộ đếm bị nhiều goroutine ghi liên tục, như số lô và số điểm, dùng `atomic`; cấu trúc nhiều trường như hàng đợi dùng mutex (bài 13); trạng thái riêng của từng kết nối thì không chia sẻ gì cả. Safe Rust chặn lỗi này ngay lúc biên dịch: `count++` từ nhiều thread không qua `Mutex` hay `Atomic*` sẽ không biên dịch được. Bù lại, Go cho bạn goroutine rẻ và code chặn trực tiếp, và công cụ `-race` đủ tốt để làm chốt chặn trong CI.

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc của repo (Go 1.27.1, không cần Docker, Kafka hay mạng):

```bash
go -C examples/toi-noi-location/go run ./cmd/loc11_nhap_mon pipeline
go -C examples/toi-noi-location/go run ./cmd/loc11_nhap_mon timeout
go -C examples/toi-noi-location/go run ./cmd/loc11_nhap_mon fail
go -C examples/toi-noi-location/go run ./cmd/loc11_nhap_mon race-bad
go -C examples/toi-noi-location/go run -race ./cmd/loc11_nhap_mon race-fixed
go -C examples/toi-noi-location/go run -race ./cmd/loc11_nhap_mon race-bad 2>&1 | head -32
go -C examples/toi-noi-location/go test -race -count=1 ./cmd/loc11_nhap_mon
```

Lệnh `race-fixed` in ba dòng:

```text
mutex:   đếm được 800000, muốn 800000
atomic:  đếm được 800000, muốn 800000
channel: đếm được 800000, muốn 800000
```

`pipeline` chạy dưới `-race` cũng in `sinh=10000 kiểm=10000 gom=10000` và `hoàn tất, không lỗi`. Số đếm (10.000, 800.000) tái lập được; số `270093`, `1242`, `729` thì không, vì chúng phụ thuộc cách hệ điều hành xếp lịch goroutine. Script `examples/toi-noi-location/verify-p02-go.sh` chạy lại phần này và kiểm tra hai điều: `race-bad` dưới `-race` phải in `WARNING: DATA RACE`, và `race-fixed` phải ra 800.000.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Đây là bộ nhập môn, không phải lab hiệu năng: bài không đo thông lượng hay độ trễ của pipeline. Mỗi số trong bài là kết quả của một lần chạy. Pipeline dùng validator ở dạng tối thiểu (từng điểm riêng lẻ, không so với điểm trước), vì validator đầy đủ cần trạng thái theo tài xế và sẽ ở bài 14. Bài cũng bỏ qua phần của Go mà ingest không cần: generics chỉ xuất hiện lúc đọc hàng đợi ở bài 13, còn interface, module và con trỏ ta sẽ gặp khi đọc code chứ không học riêng.</p>
</div>

## Bài tập

**Bài 1.** Ở stage 2, nếu mỗi worker tự gọi `close(ch2)` sau vòng `for` của nó thì chuyện gì xảy ra, và vì sao lab dùng thêm một goroutine chờ `wg.Wait()`?

<details><summary>Xem lời giải</summary><p>Có bốn worker cùng gửi vào <code>ch2</code>. Worker xong đầu tiên đóng channel; ba worker còn lại tiếp tục <code>ch2 &lt;- ev</code> vào channel đã đóng và chương trình panic (<code>send on closed channel</code>). Quy tắc: channel có nhiều người gửi thì phải có một bên thứ ba biết tất cả đã gửi xong mới đóng. Goroutine chờ <code>wg.Wait()</code> chính là bên thứ ba đó.</p></details>

**Bài 2.** Thay `count++` trong `raceBad` bằng `atomic.AddInt64` trên biến `int64` thì có đủ để `-race` không báo gì và kết quả đúng không? Còn nếu mỗi goroutine tự giữ một biến cục bộ rồi cuối cùng cộng tổng, thì sao?

<details><summary>Xem lời giải</summary><p>Cả hai đều đúng và không có race. <code>atomic.AddInt64</code> biến chuỗi đọc-cộng-ghi thành một thao tác không thể xen, nên kết quả 800.000. Cách thứ hai tránh chia sẻ ngay từ đầu: mỗi goroutine cộng vào biến riêng (không race), rồi gom các tổng riêng sau khi <code>wg.Wait()</code> hoặc qua channel. Cách thứ hai thường nhanh hơn vì không có truy cập chung nào, nhưng bài không đo nên đây là lập luận chứ không phải số liệu. Phải tự chạy rồi mới kết luận.</p></details>

**Bài 3.** Trong `runPipeline`, stage 1 có `case <-ctx.Done(): return ctx.Err()`. Bỏ nhánh này và chỉ giữ lại `ch1 <- ev` thì kịch bản `fail` kết thúc ra sao?

<details><summary>Xem lời giải</summary><p>Khi worker báo lỗi và thoát, các worker còn lại cũng thoát qua <code>ctx.Done()</code> nên không còn ai đọc <code>ch1</code>. Stage 1 gửi tiếp cho tới khi đệm 64 đầy rồi kẹt vĩnh viễn ở <code>ch1 &lt;- ev</code>; <code>g.Wait()</code> không bao giờ trả về vì chờ stage 1. Chương trình treo thay vì dừng. Đây là lý do mọi thao tác gửi hay nhận có thể chặn đều phải đi cùng <code>select</code> với <code>ctx.Done()</code>.</p></details>

Bài trước: [bài 10](/articles/location-tong-hop-geo-core-kiem-thu-hinh-hoc). Bài tiếp: [bài 12](/articles/location-go-ingest-websocket-batch-gps-tai-xe).

## Tài liệu gốc

- [A Tour of Go](https://go.dev/tour/): bài học tương tác về cú pháp, goroutine, channel.
- [Effective Go](https://go.dev/doc/effective_go): mục Concurrency giải thích triết lý chia sẻ bộ nhớ bằng giao tiếp.
- [Go Concurrency Patterns: Pipelines and cancellation](https://go.dev/blog/pipelines).
- [Go Concurrency Patterns: Context](https://go.dev/blog/context).
- [Data Race Detector](https://go.dev/doc/articles/race_detector): cách bật, cách đọc báo cáo, chi phí.
- [Package errgroup](https://pkg.go.dev/golang.org/x/sync/errgroup).
- [The Go Memory Model](https://go.dev/ref/mem).
