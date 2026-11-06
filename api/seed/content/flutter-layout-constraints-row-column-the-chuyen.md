---
slug: flutter-layout-constraints-row-column-the-chuyen
title: "Layout và constraints: Row, Column, Expanded và thẻ chuyến"
summary: "Bài 08: hiểu quy tắc constraints đi xuống, kích thước đi lên, cha đặt vị trí; chia chỗ bằng Row, Column, Expanded, Flexible; dựng TripCard không overflow ở màn hình 320 px, tái hiện overflow bằng test và đổi bố cục theo breakpoint với LayoutBuilder."
category: flutter
level: beginner
tags: [flutter, dart, mobile, toi-noi, layout, widget-test]
language: vi
---

Đội QA của Tới Nơi thử app khách trên một chiếc điện thoại màn hình hẹp (rộng 320 pixel logic). Khách đặt giao bưu kiện tới "Căn hộ B-1807, Chung cư Hoa Sen Xanh, Số 235/12/4 đường Nguyễn Văn Linh Nối Dài...". Thẻ chuyến vẽ chữ chạy tuột ra khỏi mép phải, kèm một vạch vàng đen. Trên máy lập trình viên, địa chỉ mẫu chỉ là "Số 12 đường Lá Cọ", nên chưa ai thấy lỗi.

Lỗi không nằm ở địa chỉ quá dài mà ở chỗ ta chưa hiểu widget được cấp bao nhiêu chỗ và ai quyết định điều đó. Bài này dạy một quy tắc của layout Flutter rồi dùng nó để giải thích `Row`, `Column`, `Expanded`, `Flexible`, `Stack` và `LayoutBuilder` khi dựng thẻ chuyến `TripCard`. Đây là bài 08 của lộ trình **Flutter qua Tới Nơi**. Bạn cần biết dựng cây widget và `StatelessWidget` ở [bài 07](/articles/flutter-widget-cay-widget-hot-reload-man-hinh-bao-gia), và cần `Trip`, `Money` của package `toi_noi_core` từ [bài 06](/articles/dart-package-pubspec-lint-test-toi-noi-core). Hồ sơ sản phẩm: [Tới Nơi](/articles/du-an-mau-toi-noi). Mọi con số cước và ngưỡng trong bài là số minh hoạ.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một ô vuông viết <code>SizedBox(width: 100, height: 100)</code> nằm trong <code>SizedBox(width: 300, height: 300)</code> đặt trong một <code>Center</code>. Ô vuông trên màn hình rộng 100 hay 300 pixel? Và nếu chen một <code>Align</code> giữa hai widget này thì sao?</p>
<details><summary>Xem đáp án</summary><p>Rộng 300 pixel. <code>SizedBox</code> đưa cho con một ràng buộc chặt 300x300, và con chỉ được chọn kích thước trong ràng buộc đó, nên lời đòi 100 bị bỏ qua. Chen <code>Align</code> vào thì ô vuông rộng đúng 100 pixel, vì <code>Align</code> nới lỏng ràng buộc trước khi chuyển xuống cho con. Lab chạy đúng hai trường hợp này bằng test.</p></details>
</div>

## Constraints đi xuống, kích thước đi lên, cha đặt vị trí

Tài liệu Flutter tóm hệ thống layout thành một câu: **Constraints go down. Sizes go up. Parent sets position.** Mỗi widget làm ba việc theo thứ tự:

1. Nhận **constraints** từ cha. Đó chỉ là bốn số thực: bề rộng nhỏ nhất và lớn nhất, chiều cao nhỏ nhất và lớn nhất.
2. Đưa constraints (có thể khác nhau cho từng con) xuống các con, rồi hỏi mỗi con "bạn muốn to bao nhiêu". Con trả về một **size** nằm trong constraints đó.
3. Tự đặt từng con vào một toạ độ, rồi báo size của chính mình lên cho cha.

Hình 1 vẽ ba bước trên một đoạn của thẻ chuyến.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 400" role="img" aria-label="Hình 1. Ba widget xếp dọc: Padding, Row, Expanded bọc Text. Từ trên xuống, mỗi cặp liền kề có mũi tên đi xuống ghi constraints và mũi tên đi lên ghi kích thước. Padding đưa xuống bề rộng tối đa 288, Row đưa xuống cho Text bề rộng chặt 264, Row đặt Expanded tại x bằng 24."><defs><marker id="fl08-1-d" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl08-1-u" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs>
<rect x="320" y="20" width="240" height="76" rx="10" class="f-phys-soft s-phys"/><text x="336" y="48" class="t-b">Cha: Padding(all: 16)</text><text x="336" y="70" class="t-small">nhận từ trên: w tối đa 320</text><text x="336" y="86" class="t-small">đặt Row tại (16, 16)</text>
<rect x="320" y="166" width="240" height="76" rx="10" class="f-pk-soft s-pk"/><text x="336" y="194" class="t-b">Con: Row</text><text x="336" y="216" class="t-small">đo Icon và SizedBox trước</text><text x="336" y="232" class="t-small">đặt Expanded tại x = 24</text>
<rect x="320" y="312" width="240" height="76" rx="10" class="f-warn-soft s-warn"/><text x="336" y="340" class="t-b">Cháu: Expanded &gt; Text</text><text x="336" y="362" class="t-small">nhận w = 264 (chặt)</text><text x="336" y="378" class="t-small">tự chọn chiều cao theo số dòng</text>
<path d="M400 98 L400 164" class="f-none s-pk" stroke-width="2" marker-end="url(#fl08-1-d)"/>
<text x="20" y="124" class="t-b t-pk">constraints đi xuống</text><text x="20" y="144" class="t-small">w tối đa 320 - 2 x 16 = 288</text>
<path d="M480 164 L480 98" class="f-none s-phys" stroke-width="2" marker-end="url(#fl08-1-u)"/>
<text x="580" y="124" class="t-b t-phys">kích thước đi lên</text><text x="580" y="144" class="t-small">Row trả 288 x 40 (ví dụ)</text>
<path d="M400 244 L400 310" class="f-none s-pk" stroke-width="2" marker-end="url(#fl08-1-d)"/>
<text x="20" y="270" class="t-b t-pk">constraints đi xuống</text><text x="20" y="290" class="t-small">288 - 16 (Icon) - 8 (SizedBox) = 264</text>
<path d="M480 310 L480 244" class="f-none s-phys" stroke-width="2" marker-end="url(#fl08-1-u)"/>
<text x="580" y="270" class="t-b t-phys">kích thước đi lên</text><text x="580" y="290" class="t-small">Text trả 264 x 40 (hai dòng)</text>
</svg>
</div>
<figcaption>Hình 1. Constraints đi xuống (mũi tên xanh dương), kích thước đi lên (mũi tên xanh ngọc). Vị trí do cha quyết định. Số đo là ví dụ giản lược: Row thật trong thẻ hẹp hơn 288 vì còn lề của thẻ.</figcaption>
</figure>

Hệ quả: widget **không** tự ý chọn kích thước, chỉ chọn trong khoảng cha cho phép. Flutter đo layout trong một lượt nên chạy nhanh; đổi lại, con không biết vị trí tuyệt đối của mình và không quyết định nó.

Ràng buộc **chặt** (tight) có min bằng max, con chỉ còn một lựa chọn; ràng buộc **lỏng** (loose) có min bằng 0, con chọn từ 0 tới max. Đoạn dưới đo cả hai dạng:

### constraint_demos.dart · 08-layout-constraints/lib/constraint_demos.dart (trích)

```dart
/// Cha ép chặt 300x300 (`SizedBox` truyền constraints chặt cho con).
/// Con đòi 100x100 nhưng không được nghe, trừ khi có `Align` nới lỏng.
class TightParentDemo extends StatelessWidget {
  const TightParentDemo({super.key, required this.loosen});

  /// `true`: chen `Align` giữa cha và con để con được tự chọn kích thước.
  final bool loosen;

  static const boxKey = Key('demo-box');

  @override
  Widget build(BuildContext context) {
    const box = ColoredBox(
      key: boxKey,
      color: Colors.teal,
      child: SizedBox(width: 100, height: 100),
    );
    return SizedBox(
      width: 300,
      height: 300,
      child: loosen ? const Align(child: box) : box,
    );
  }
}
```

Khi `loosen` là `false`, `SizedBox` ngoài **biến** constraints lỏng của cha thành ràng buộc chặt 300x300 cho con. `SizedBox(width: 100, height: 100)` bên trong đòi 100, nhưng 100 nằm ngoài khoảng chặt `[300, 300]` nên bị kéo về 300. Khi `loosen` là `true`, `Align` đổi 300x300 chặt thành `0..300` rồi mới chuyển xuống, nên ô vuông được 100x100 và `Align` đặt nó vào giữa. Test đo `getSize` và `getCenter`: 300x300, 100x100, và tâm ô trùng tâm của cha.

Ở gốc cây, màn hình ép widget gốc đúng bằng kích thước của nó. Lab tận dụng điều đó: `tester.view.physicalSize` đổi cửa sổ giả.

## Row và Column: đo con cố định trước, chia phần còn lại theo flex

`Row` xếp con theo chiều ngang, `Column` theo chiều dọc; cả hai là `Flex` với trục chính khác nhau. Mã nguồn của `RenderFlex` mô tả thuật toán qua sáu bước, và ba bước đầu là điều ta cần:

1. Mỗi con **không có flex** được đo với trục chính **không giới hạn** và trục phụ theo constraints của `Row`.
2. Phần trục chính còn lại được chia cho các con có flex, theo tỉ lệ flex.
3. Mỗi con có flex được layout một lần với phần đã chia: `Expanded` nhận ràng buộc **chặt** (buộc phải lấp đầy), `Flexible` mặc định nhận ràng buộc **lỏng** (từ 0 tới phần được chia).

Hình 2 chạy thuật toán trên một hàng 288 pixel gồm biểu tượng 40 pixel, một con co giãn và nhãn cước 90 pixel.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 380" role="img" aria-label="Hình 2. Bốn thanh ngang thể hiện một Row rộng 288 pixel, tỉ lệ một pixel thật bằng hai pixel hình. Bước 1 đo Icon 40 và nhãn cước 90, còn lại 158. Bước 2 chia 158 cho con có flex. Bước 3 Expanded nhận chặt 158 nên lấp đầy. Bước 3b Flexible với con rộng 100 nên giữ 100 và để trống 58.">
<text x="150" y="16" class="t-small">Row rộng 288 px (một pixel thật bằng hai pixel trong hình)</text>
<text x="150" y="44" class="t-b">1. Con không flex được đo trước, trục chính không giới hạn</text>
<rect x="150" y="54" width="80" height="44" rx="6" class="f-phys-soft s-phys"/><text x="164" y="81" class="t-small">Icon 40</text>
<rect x="230" y="54" width="316" height="44" rx="6" class="f-none s-muted" stroke-dasharray="6 5"/><text x="262" y="81" class="t-small">còn lại: 288 - 40 - 90 = 158</text>
<rect x="546" y="54" width="180" height="44" rx="6" class="f-phys-soft s-phys"/><text x="560" y="81" class="t-small">Fare 90</text>
<text x="150" y="130" class="t-b">2. Phần còn lại chia cho con có flex, theo tỉ lệ flex</text>
<rect x="150" y="140" width="80" height="44" rx="6" class="f-phys-soft s-phys"/><text x="164" y="167" class="t-small">Icon 40</text>
<rect x="230" y="140" width="316" height="44" rx="6" class="f-none s-pk" stroke-dasharray="6 5"/><text x="262" y="167" class="t-small">flex 1 trên tổng flex 1 = 158</text>
<rect x="546" y="140" width="180" height="44" rx="6" class="f-phys-soft s-phys"/><text x="560" y="167" class="t-small">Fare 90</text>
<text x="150" y="216" class="t-b">3. Expanded: ràng buộc chặt 158, con buộc lấp đầy</text>
<rect x="150" y="226" width="80" height="44" rx="6" class="f-phys-soft s-phys"/><text x="164" y="253" class="t-small">Icon 40</text>
<rect x="230" y="226" width="316" height="44" rx="6" class="f-pk-soft s-pk"/><text x="262" y="253" class="t-small">Expanded nhận đúng 158</text>
<rect x="546" y="226" width="180" height="44" rx="6" class="f-phys-soft s-phys"/><text x="560" y="253" class="t-small">Fare 90</text>
<text x="150" y="302" class="t-b">3b. Flexible (thay cho Expanded): ràng buộc lỏng 0..158, con đòi 100 thì giữ 100</text>
<rect x="150" y="312" width="80" height="44" rx="6" class="f-phys-soft s-phys"/><text x="164" y="339" class="t-small">Icon 40</text>
<rect x="230" y="312" width="200" height="44" rx="6" class="f-pk-soft s-pk"/><text x="244" y="339" class="t-small">con 100</text>
<rect x="430" y="312" width="116" height="44" rx="6" class="f-none s-muted" stroke-dasharray="6 5"/><text x="446" y="339" class="t-small">trống 58</text>
<rect x="546" y="312" width="180" height="44" rx="6" class="f-phys-soft s-phys"/><text x="560" y="339" class="t-small">Fare 90</text>
</svg>
</div>
<figcaption>Hình 2. Thuật toán chia chỗ của Row (số đo là ví dụ giản lược). Ô liền là con đã có kích thước, ô đứt là phần chưa ai nhận. Expanded lấp kín phần còn lại, Flexible chỉ cho phép con nhỏ hơn nó.</figcaption>
</figure>

Hai widget dưới đây chạy cùng thuật toán với số khác Hình 2: hàng 360 pixel có ô cố định 60 cùng hai `Expanded` flex 1 và 2, và hàng 300 pixel đặt `Flexible` cạnh `Expanded`.

### constraint_demos.dart · 08-layout-constraints/lib/constraint_demos.dart (trích)

```dart
/// Hàng 360 px: một ô cố định 60 px, hai ô `Expanded` flex 1 và flex 2.
class FlexRatioRow extends StatelessWidget {
  const FlexRatioRow({super.key});

  static const fixedKey = Key('flex-fixed');
  static const oneKey = Key('flex-1');
  static const twoKey = Key('flex-2');

  @override
  Widget build(BuildContext context) {
    return const SizedBox(
      width: 360,
      height: 48,
      child: Row(
        children: [
          SizedBox(key: fixedKey, width: 60, height: 48),
          Expanded(flex: 1, child: SizedBox(key: oneKey, height: 48)),
          Expanded(flex: 2, child: SizedBox(key: twoKey, height: 48)),
        ],
      ),
    );
  }
}

/// Hàng 300 px, hai con cùng flex 1 và cùng đòi rộng 50 px: `Flexible` (loose)
/// để con giữ 50 px, `Expanded` (tight) ép con phình ra đúng phần được cấp.
class FlexFitRow extends StatelessWidget {
  const FlexFitRow({super.key});

  static const flexibleKey = Key('fit-flexible');
  static const expandedKey = Key('fit-expanded');

  @override
  Widget build(BuildContext context) {
    return const SizedBox(
      width: 300,
      height: 48,
      child: Row(
        children: [
          Flexible(child: SizedBox(key: flexibleKey, width: 50, height: 48)),
          Expanded(child: SizedBox(key: expandedKey, width: 50, height: 48)),
        ],
      ),
    );
  }
}
```

Hàng đầu còn lại 360 - 60 = 300, chia 1:2 thành 100 và 200; test đo đúng `60/100/200`. Ở hàng hai mỗi con được chia 150, nhưng `Flexible` giữ 50 còn `Expanded` phình ra 150. Chọn `Expanded` khi con **phải** lấp đầy phần của nó, `Flexible` khi con được nhỏ hơn.

Vài công cụ đi kèm. `SizedBox(width: 12)` là khoảng cách cố định. `mainAxisSize` mặc định là `max`, tức chiếm hết trục chính được cấp; đặt `MainAxisSize.min` để `Row` hoặc `Column` co vừa nội dung. `crossAxisAlignment` canh con theo trục phụ. `Align` đặt con theo một điểm và nới lỏng constraints như ta vừa thấy. `Wrap` giống `Row` nhưng thiếu chỗ thì xuống dòng thay vì tràn.

## Dựng TripCard

Thẻ nhận một `Trip` và hai địa chỉ. Domain của bài 06 chưa có địa chỉ, nên ta truyền chúng qua tham số riêng thay vì sửa package. Dữ liệu mẫu dựng như app sẽ làm: `Trip.request` rồi `apply` từng sự kiện, cước do `calculateFare` tính.

### sample_trips.dart · 08-layout-constraints/lib/sample_trips.dart (trích)

```dart
/// Dựng chuyến đang ở [status] đúng như app sẽ làm: `Trip.request` rồi `apply`
/// từng sự kiện. Cước tính bằng `calculateFare`, không gõ tay.
Trip tripAt(
  TripStatus status, {
  String id = 'T-1042',
  ServiceKind serviceKind = ServiceKind.ride,
  double distanceKm = 5,
}) {
  var trip = Trip.request(
    id: id,
    serviceKind: serviceKind,
    distanceKm: distanceKm,
    requestedAt: DateTime.utc(2026, 11, 6, 2, 30),
  );
  for (final event in _eventsTo(status)) {
    trip = trip.apply(event);
  }
  return trip;
}
```

Chuyến 5 km hiện `25.500 VND`, 2,001 km hiện `12.005 VND`, 100 km hiện `453.000 VND`; test kiểm cả ba chuỗi. Khung thẻ là một `LayoutBuilder` chọn một trong hai thân, mục cuối quay lại:

### trip_card.dart · 08-layout-constraints/lib/trip_card.dart (trích)

```dart
  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final layout = layoutFor(constraints.maxWidth);
        return Card(
          margin: EdgeInsets.zero,
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: switch (layout) {
              TripCardLayout.compact => _CompactBody(this),
              TripCardLayout.wide => _WideBody(this),
            },
          ),
        );
      },
    );
  }
```

Thân `compact` cho điện thoại gồm hai hàng:

### trip_card.dart · 08-layout-constraints/lib/trip_card.dart (trích)

```dart
/// Điện thoại: hai hàng. Hàng trên là biểu tượng + hành trình; hàng dưới là
/// trạng thái + cước trong `Wrap`, đủ chỗ thì nằm cùng hàng, thiếu chỗ (chữ
/// lớn) thì cước xuống dòng thay vì tràn.
///
/// `stretch` cho mỗi con bề rộng chặt bằng cột. Với `start`, `Wrap` nhận
/// ràng buộc lỏng, co theo nội dung và `spaceBetween` không còn tác dụng.
class _CompactBody extends StatelessWidget {
  const _CompactBody(this.card);

  final TripCard card;

  @override
  Widget build(BuildContext context) {
    return Column(
      key: TripCard.compactKey,
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _ServiceBadge(trip: card.trip),
            const SizedBox(width: 12),
            Expanded(
              child: _Journey(
                trip: card.trip,
                pickup: card.pickup,
                dropoff: card.dropoff,
              ),
            ),
          ],
        ),
        const SizedBox(height: 12),
        Wrap(
          alignment: WrapAlignment.spaceBetween,
          crossAxisAlignment: WrapCrossAlignment.center,
          spacing: 12,
          runSpacing: 8,
          children: [
            _StatusLabel(status: card.trip.status),
            _FareBlock(trip: card.trip),
          ],
        ),
      ],
    );
  }
}
```

Hàng đầu là `Row` gồm biểu tượng cỡ cố định, khoảng cách 12 pixel và `_Journey` nằm trong `Expanded`, nhận phần chiều ngang còn lại dưới dạng ràng buộc **chặt, hữu hạn**. Nhờ đó mọi `Text` bên trong biết chỗ mà xuống dòng. Hàng thứ hai dùng `Wrap`: đủ chỗ thì trạng thái và cước nằm cùng hàng, thiếu chỗ (chữ hệ thống lớn) thì cước xuống dòng. `Column` có `MainAxisSize.min` để thẻ chỉ cao bằng nội dung, và `CrossAxisAlignment.stretch` để mỗi con nhận bề rộng chặt bằng cột: với `start`, `Wrap` nhận ràng buộc lỏng, co theo nội dung và `spaceBetween` không còn tác dụng (test đo ở 560 pixel).

Mỗi dòng địa chỉ cũng là một `Row` có `Expanded` bọc `Text`, kèm giới hạn hai dòng và dấu ba chấm:

### trip_card.dart · 08-layout-constraints/lib/trip_card.dart (trích)

```dart
class _AddressLine extends StatelessWidget {
  const _AddressLine({required this.icon, required this.text});

  final IconData icon;
  final String text;

  @override
  Widget build(BuildContext context) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.only(top: 2),
          child: Icon(icon, size: 16),
        ),
        const SizedBox(width: 8),
        Expanded(
          child: Text(text, maxLines: 2, overflow: TextOverflow.ellipsis),
        ),
      ],
    );
  }
}
```

`Expanded` cho `Text` bề rộng hữu hạn; `maxLines: 2` và `TextOverflow.ellipsis` quyết định việc cần làm khi vẫn dài: cắt ở dòng hai và thêm "...".

Biểu tượng dịch vụ dùng `Stack` để đặt chấm trạng thái đè lên góc dưới phải:

### trip_card.dart · 08-layout-constraints/lib/trip_card.dart (trích)

```dart
  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    return SizedBox(
      width: 44,
      height: 44,
      child: Stack(
        children: [
          Align(
            alignment: Alignment.center,
            child: CircleAvatar(
              radius: 20,
              backgroundColor: scheme.primaryContainer,
              child: Icon(
                iconOf(trip.serviceKind),
                color: scheme.onPrimaryContainer,
              ),
            ),
          ),
          Positioned(
            right: 0,
            bottom: 0,
            child: Container(
              key: const Key('status-dot'),
              width: 12,
              height: 12,
              decoration: BoxDecoration(
                color: statusColor(trip.status, scheme),
                shape: BoxShape.circle,
                border: Border.all(color: scheme.surface, width: 2),
              ),
            ),
          ),
        ],
      ),
    );
  }
```

`Stack` xếp chồng các con theo thứ tự khai báo. Con có `Positioned` được đặt theo khoảng cách tới mép `Stack` (ở đây `right: 0, bottom: 0`); con không có `Positioned` (là `Align` bọc `CircleAvatar`) được đặt theo `alignment`. `SizedBox(width: 44, height: 44)` cho `Stack` ràng buộc chặt 44x44, nên chấm tròn nằm ở góc của ô đó; test kiểm góc dưới phải của chấm trùng góc dưới phải của ô.

## Overflow: đọc lỗi và tái hiện bằng test

### broken_examples.dart · 08-layout-constraints/lib/broken_examples.dart (trích)

```dart
/// SAI CÓ CHỦ ĐÍCH: thẻ chuyến viết như người mới. `Text` không nằm trong
/// `Expanded` nên `Row` đo nó với bề rộng không giới hạn; địa chỉ dài làm hàng
/// rộng hơn màn hình và Flutter báo `RenderFlex overflowed`.
/// Bản đúng là `TripCard` trong `trip_card.dart`.
class BrokenTripCard extends StatelessWidget {
  const BrokenTripCard({super.key, required this.trip, required this.pickup});

  final Trip trip;
  final String pickup;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Row(
          children: [
            const Icon(Icons.directions_car),
            const SizedBox(width: 12),
            Text(pickup),
            const SizedBox(width: 12),
            Text(trip.fare.toString()),
          ],
        ),
      ),
    );
  }
}
```

Đây là ca sai có chủ đích. `Text(pickup)` không nằm trong `Expanded`, nên theo bước 1 của thuật toán, `Row` đo nó với bề rộng **không giới hạn**. Địa chỉ dài cho ra `Text` rộng hơn cả màn hình; tổng bề rộng các con vượt bề rộng `Row`, và phần thừa gọi là **overflow**. Hình 3 đặt hai cách viết cạnh nhau với cùng dữ liệu.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 3. Hai hàng cùng nội dung trong khung rộng 288 pixel. Hàng sai: Icon, Text rộng 400 và nhãn cước, tổng 530 nên tràn 242 pixel ra ngoài khung. Hàng đúng: Text nằm trong Expanded nhận 158 pixel và xuống hai dòng, tổng đúng 288."><defs><marker id="fl08-3-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<text x="20" y="22" class="t-b t-warn">Sai: Text không nằm trong Expanded</text>
<rect x="20" y="34" width="432" height="60" rx="6" class="f-none s-muted" stroke-dasharray="6 5"/>
<rect x="20" y="42" width="36" height="44" rx="6" class="f-phys-soft s-phys"/><text x="24" y="69" class="t-small">Icon</text>
<rect x="68" y="42" width="600" height="44" rx="6" class="f-warn-soft s-warn"/><text x="82" y="69" class="t-small">Text(địa chỉ dài): rộng 400</text>
<rect x="680" y="42" width="135" height="44" rx="6" class="f-warn-soft s-warn"/><text x="692" y="69" class="t-small">Fare 90</text>
<path d="M452 108 L815 108" class="f-none s-warn" stroke-width="2" marker-end="url(#fl08-3-w)"/><text x="470" y="128" class="t-warn t-b">tràn 242 px (24 + 8 + 400 + 8 + 90 = 530 > 288)</text>
<text x="20" y="158" class="t-small">Row đo con không flex với bề rộng không giới hạn, nên Text lấy trọn 400 px.</text>
<text x="20" y="200" class="t-b t-pk">Đúng: Text bọc trong Expanded</text>
<rect x="20" y="212" width="432" height="60" rx="6" class="f-none s-muted" stroke-dasharray="6 5"/>
<rect x="20" y="220" width="36" height="44" rx="6" class="f-phys-soft s-phys"/><text x="24" y="247" class="t-small">Icon</text>
<rect x="68" y="220" width="237" height="44" rx="6" class="f-pk-soft s-pk"/><text x="80" y="247" class="t-small">Expanded 158: hai dòng</text>
<rect x="317" y="220" width="135" height="44" rx="6" class="f-phys-soft s-phys"/><text x="329" y="247" class="t-small">Fare 90</text>
<text x="472" y="247" class="t-pk t-b">24 + 8 + 158 + 8 + 90 = 288</text>
<text x="20" y="292" class="t-small">Tỉ lệ một pixel thật bằng 1,5 pixel hình. Khung nét đứt là bề rộng 288 px của Row.</text>
</svg>
</div>
<figcaption>Hình 3. Cùng icon, địa chỉ và cước (số đo là ví dụ minh hoạ). Bỏ Expanded thì Text rộng 400 và hàng tràn 242 pixel; có Expanded thì Text chỉ được 158 pixel và xuống dòng.</figcaption>
</figure>

Ở debug, Flutter vẽ vạch vàng đen ở mép tràn và in `A RenderFlex overflowed by ... pixels on the right`. Theo mã nguồn `DebugOverflowIndicatorMixin`, phần vẽ nằm trong `assert` nên chỉ có ở bản debug.

Test đặt cửa sổ 320 pixel (`devicePixelRatio = 1` để pixel vật lý bằng pixel logic) rồi lấy lỗi bằng `takeException`:

### support.dart · 08-layout-constraints/test/support.dart (trích)

```dart
void setScreen(WidgetTester tester, double width, [double height = 900]) {
  tester.view.devicePixelRatio = 1;
  tester.view.physicalSize = Size(width, height);
  addTearDown(tester.view.reset);
}
```

### overflow_test.dart · 08-layout-constraints/test/overflow_test.dart (trích)

```dart
    testWidgets('BrokenTripCard ở 320 px: RenderFlex overflowed', (
      tester,
    ) async {
      setScreen(tester, 320);
      await tester.pumpWidget(
        app(
          BrokenTripCard(
            trip: tripAt(TripStatus.arriving),
            pickup: longAddress,
          ),
        ),
      );

      // Flutter báo overflow qua FlutterError.onError; flutter_test giữ lại
      // để takeException lấy ra. Lấy một lần: gọi lần hai sẽ trả null.
      final error = tester.takeException();
      expect(error, isA<FlutterError>());
      expect(error.toString(), contains('A RenderFlex overflowed by'));
      expect(error.toString(), contains('on the right'));
      expect(tester.takeException(), isNull);
    });
```

`pumpWidget` dựng và vẽ một khung hình, lúc đó lỗi overflow được báo; `flutter_test` giữ lại để ta lấy bằng `takeException`, gọi lần hai thì trả `null`. Ca sai nhờ vậy tái hiện **ổn định**. Ca đúng là nửa kia của bằng chứng: `TripCard` trong cùng cửa sổ 320 pixel, cả hai địa chỉ đều dài.

### trip_card_test.dart · 08-layout-constraints/test/trip_card_test.dart (trích)

```dart
    testWidgets(
      'không overflow, thẻ rộng 288 px và mọi chữ nằm trong màn hình',
      (tester) async {
        setScreen(tester, 320);
        await tester.pumpWidget(
          app(
            TripCard(
              trip: tripAt(TripStatus.arriving, distanceKm: 12.4),
              pickup: longAddress,
              dropoff: longAddress,
            ),
          ),
        );

        // Overflow được báo qua FlutterError; không có gì để lấy nghĩa là sạch.
        expect(tester.takeException(), isNull);
        expect(tester.getSize(find.byType(TripCard)).width, 288);
        expect(find.byKey(TripCard.compactKey), findsOneWidget);

        for (final element in find.byType(Text).evaluate()) {
          final rect = tester.getRect(find.byWidget(element.widget));
          expect(rect.left, greaterThanOrEqualTo(0));
          expect(rect.right, lessThanOrEqualTo(320));
        }
      },
    );
```

`takeException` trả `null` nghĩa là không overflow. Test còn đo thẻ rộng 288 và rìa phải mọi `Text` nằm trong 320. Để chắc test có răng, ta thử phá mã: bỏ `Expanded` ở `_AddressLine` thì phần lớn test (18 trên 31 lúc thử) của lab chuyển đỏ. Lưu ý: dưới `flutter test`, mọi chữ dùng font test mặc định (FlutterTest), mỗi ký tự là một ô vuông cạnh bằng cỡ chữ, nên chữ trong test thường rộng hơn chữ thật. Phép thử hẹp vì vậy khắt khe, nhưng không thay được việc nhìn thẻ trên máy thật.

Hai lỗi kinh điển khác (`ListView` trong `Column`, `Expanded` trong khung cuộn ngang) cùng gốc: cha cho con **không giới hạn** ở trục đó. Phần "Lỗi thường gặp" nêu cách sửa.

## LayoutBuilder: bố cục theo chỗ được cấp, không theo màn hình

`LayoutBuilder` đưa cho `builder` đúng `BoxConstraints` mà cha truyền cho nó, và `builder` chạy lại khi cha truyền constraints khác (theo tài liệu API). Hình 4 cho thấy cùng `TripCard` chọn hai thân khác nhau.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 320" role="img" aria-label="Hình 4. Hai thẻ chuyến. Bên trái, constraints.maxWidth bằng 288, nhỏ hơn 600, thẻ dùng bố cục compact hai hàng: biểu tượng và hai địa chỉ ở hàng trên, nhãn trạng thái và cước ở hàng dưới. Bên phải, maxWidth bằng 600 trở lên, thẻ dùng bố cục wide một hàng: biểu tượng, địa chỉ, nhãn trạng thái, cước."><text x="20" y="22" class="t-b t-pk">maxWidth = 288 &lt; 600: compact</text>
<rect x="20" y="36" width="320" height="190" rx="10" class="f-pk-soft s-pk"/>
<circle cx="56" cy="74" r="16" class="f-phys-soft s-phys"/>
<text x="84" y="70" class="t-b">Đặt xe · T-1042</text><text x="84" y="90" class="t-small">Căn hộ B-1807, Chung cư Hoa Sen...</text><text x="84" y="108" class="t-small">Số 12 đường Lá Cọ, Quận 1</text>
<rect x="36" y="160" width="120" height="30" rx="8" class="f-surface s-line"/><text x="48" y="180" class="t-small">Tài xế đang tới</text>
<text x="226" y="180" class="t-b">25.500 VND</text><text x="226" y="198" class="t-small">5 km</text>
<text x="20" y="252" class="t-small">hàng 1: Row [icon, Expanded(hành trình)]</text><text x="20" y="270" class="t-small">hàng 2: Wrap [trạng thái, cước]</text>
<text x="400" y="22" class="t-b t-phys">maxWidth = 600 trở lên: wide</text>
<rect x="400" y="36" width="460" height="100" rx="10" class="f-phys-soft s-phys"/>
<circle cx="436" cy="86" r="16" class="f-surface s-phys"/>
<text x="464" y="76" class="t-b">Đặt xe · T-1042</text><text x="464" y="96" class="t-small">Căn hộ B-1807, Chung cư...</text><text x="464" y="114" class="t-small">Số 12 đường Lá Cọ, Quận 1</text>
<rect x="660" y="72" width="110" height="30" rx="8" class="f-surface s-line"/><text x="668" y="92" class="t-small">Tài xế đang tới</text>
<text x="780" y="86" class="t-b">25.500 VND</text><text x="780" y="104" class="t-small">5 km</text>
<text x="400" y="162" class="t-small">một hàng: Row [icon, Expanded(hành trình), trạng thái, cước]</text>
<text x="20" y="302" class="t-small">Cùng một TripCard: LayoutBuilder chọn nhánh (hai cây con khác nhau) theo constraints của cha. Hình không theo tỉ lệ.</text>
</svg>
</div>
<figcaption>Hình 4. Bố cục chọn theo bề rộng thẻ được cấp. Ngưỡng 600 là số minh hoạ của lab, không phải chuẩn của nền tảng.</figcaption>
</figure>

Điểm then chốt là `LayoutBuilder` đọc **constraints của cha**, không đọc kích thước màn hình. Thẻ nằm trong ô rộng 320 pixel của máy tính bảng vẫn chọn `compact`; hai thẻ cạnh nhau trong `Row` với `Expanded` trên màn hình 1000 pixel mỗi thẻ được 484 pixel nên cũng `compact`. `MediaQuery` cho biết cả cửa sổ, hợp để hỏi "máy này lớn cỡ nào", nhưng sai chỗ khi câu hỏi thật là "widget này được cấp bao nhiêu chỗ". Cả hai tình huống đều có test.

Test breakpoint đặt cửa sổ 631 và 632 pixel (thẻ rộng 599 và 600 sau hai lề 16): 599 là `compact`, 600 là `wide`. Test xoay màn hình giữ nguyên cây widget, chỉ đổi cửa sổ rồi `pump` lại:

### trip_card_test.dart · 08-layout-constraints/test/trip_card_test.dart (trích)

```dart
    testWidgets('xoay màn hình 390 -> 844 làm bố cục đổi trong cùng một cây', (
      tester,
    ) async {
      final card = TripCard(
        trip: tripAt(TripStatus.accepted),
        pickup: shortAddress,
        dropoff: longAddress,
      );
      setScreen(tester, 390, 844);
      await tester.pumpWidget(app(card));
      expect(find.byKey(TripCard.compactKey), findsOneWidget);

      tester.view.physicalSize = const Size(844, 390);
      await tester.pump();
      expect(find.byKey(TripCard.wideKey), findsOneWidget);
      expect(tester.takeException(), isNull);
    });
```

`physicalSize` tính bằng pixel vật lý, còn layout dùng pixel logic, bằng pixel vật lý chia `devicePixelRatio`. Một test đặt `devicePixelRatio = 3` và `physicalSize = Size(960, 2700)` rồi xác nhận thẻ vẫn rộng 288 và `compact`, hệt màn hình 320 pixel logic. Khi mô phỏng một máy cụ thể, nhớ chia cho tỉ lệ này.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi layout hay gặp</span>
<ul>
<li><strong><code>Text</code> dài trong <code>Row</code> không có <code>Expanded</code> hoặc <code>Flexible</code>.</strong> Dấu hiệu: vạch vàng đen ở debug và <code>A RenderFlex overflowed by ... pixels on the right</code>. Nguyên nhân: <code>Row</code> đo con không flex với bề rộng không giới hạn. Cách sửa: bọc <code>Text</code> trong <code>Expanded</code> và đặt <code>maxLines</code>, <code>overflow</code>. Lỗi chỉ lộ khi nội dung đủ dài.</li>
<li><strong><code>ListView</code> đặt trực tiếp trong <code>Column</code>.</strong> Dấu hiệu: <code>Vertical viewport was given unbounded height</code>. Nguyên nhân: <code>Column</code> cho con chiều cao không giới hạn, <code>ListView</code> cần hữu hạn. Cách sửa: bọc <code>ListView</code> trong <code>Expanded</code> như <code>ListInColumnFixed</code>.</li>
<li><strong><code>Expanded</code> trong <code>Row</code> nằm trong khung cuộn ngang.</strong> Dấu hiệu: <code>RenderFlex children have non-zero flex but incoming width constraints are unbounded</code>. Nguyên nhân: khung cuộn ngang cho <code>Row</code> bề rộng không giới hạn, không còn gì để chia. Cách sửa: bỏ <code>Expanded</code>, cho con bề rộng cụ thể.</li>
<li><strong>Đặt <code>width</code> mà widget không đổi.</strong> Dấu hiệu: ô đòi 100 vẫn rộng bằng cha. Nguyên nhân: cha truyền ràng buộc chặt. Cách sửa: chen <code>Align</code> hoặc <code>Center</code>.</li>
<li><strong>Chọn bố cục bằng <code>MediaQuery</code> khi câu hỏi là "được cấp bao nhiêu chỗ".</strong> Dấu hiệu: đúng trên cả màn hình nhưng sai trong cột hẹp. Cách sửa: dùng <code>LayoutBuilder</code> ở chính widget cần quyết định và test cả ô hẹp trong màn hình rộng.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/08-layout-constraints
flutter pub get
flutter analyze
flutter test
```

Đã kiểm với Flutter 3.47.4 và Dart 3.13.3: `flutter analyze` 0 issue, `flutter test` 32 test xanh. Mức: **Kiểm chứng tự động**, không cần thiết bị.

| Test | Chứng minh điều gì |
|---|---|
| `trip_card_test.dart`, nhóm dữ liệu | `25.500 VND`, `12.005 VND`, `453.000 VND` lấy từ `Trip.fare`; nhãn 8 trạng thái dựng bằng `Trip.request` + `apply`; ba dịch vụ ba biểu tượng; `formatKm` |
| `trip_card_test.dart`, nhóm 320 px | Không overflow với địa chỉ dài; thẻ rộng 288; chữ nằm trong màn hình; địa chỉ bị cắt hai dòng (`didExceedMaxLines`); chữ hệ thống 1,5 lần vẫn không tràn |
| `trip_card_test.dart`, nhóm `LayoutBuilder` | 599 là `compact`, 600 là `wide`; xoay màn hình đổi bố cục; bố cục theo cha chứ không theo màn hình; `devicePixelRatio` 3; hai thẻ cạnh nhau là `compact`; ở 560 px cước canh sát mép phải thẻ (khoá `stretch`) |
| `overflow_test.dart` | `BrokenTripCard` ném `RenderFlex overflowed` (`takeException`); bản `TripCard` cùng dữ liệu thì không; hai lỗi unbounded có thông điệp đúng |
| `constraints_test.dart` | Ràng buộc chặt ép 100 thành 300; `Align` nới lỏng; flex `60/100/200`; `Flexible` giữ 50, `Expanded` 150 |
| `gallery_test.dart` | Màn hình mẫu dựng đủ 4 thẻ không lỗi ở 320, 390, 768 pixel |

**Chưa làm**: chạy trên iOS Simulator hay Android emulator (máy kiểm chưa có Android SDK và Xcode đầy đủ); cỡ chữ gấp đôi, accessibility và responsive đầy đủ (bài 16); golden test (bài 29); `intl` cho tiền và số km (bài 14); địa chỉ trong `Trip`. 

## Bài tập

1. Một `Row` rộng 400 pixel gồm `Expanded(flex: 1)`, `Expanded(flex: 3)` và `SizedBox(width: 80)`. Mỗi `Expanded` rộng bao nhiêu?
2. Trong `_AddressLine`, đổi `Expanded` thành `Flexible`. Với địa chỉ dài, thẻ có overflow không? `Flexible` khác gì `Expanded` ở đây?
3. Ta muốn thẻ chỉ chuyển sang `wide` khi rộng từ 720 pixel. Sửa gì trong lab, và những test nào buộc phải đổi theo?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Phần còn lại là 400 - 80 = 320, chia 1:3 thành 80 và 240 pixel.</p>
<p><strong>2.</strong> Không overflow: <code>Flexible</code> vẫn cho <code>Text</code> ràng buộc tối đa bằng phần được chia, nên chữ vẫn xuống dòng và bị cắt bằng <code>maxLines</code>. Khác biệt là ràng buộc lỏng thay vì chặt: con được nhỏ hơn phần chia. Với <code>Text</code> hai cách gần như giống nhau; khác rõ khi con có kích thước riêng như trong <code>FlexFitRow</code>.</p>
<p><strong>3.</strong> Đổi <code>wideBreakpoint</code> thành 720. Các test ghim số cũ sẽ đỏ: <code>expect(TripCard.wideBreakpoint, 600)</code>, cặp <code>layoutFor(599.9)</code> và <code>layoutFor(600)</code>, và hai test cửa sổ 631 và 632 pixel (phải dời thành 751 và 752, tức thẻ rộng 719 và 720). Các test còn lại không ghim số này.</p>
</details>

Bài trước: [Widget, cây widget và hot reload](/articles/flutter-widget-cay-widget-hot-reload-man-hinh-bao-gia). Bài tiếp: [ListView, Sliver và cuộn: danh sách lịch sử chuyến](/articles/flutter-listview-sliver-scroll-lich-su-chuyen).

## Tài liệu gốc

- [Understanding constraints](https://docs.flutter.dev/ui/layout/constraints) và [Layouts in Flutter](https://docs.flutter.dev/ui/layout) (docs.flutter.dev).
- [Layout widgets](https://docs.flutter.dev/ui/widgets/layout) và [Common Flutter errors](https://docs.flutter.dev/testing/common-errors) (docs.flutter.dev).
- [Adaptive and responsive design](https://docs.flutter.dev/ui/adaptive-responsive) (docs.flutter.dev).
- [BoxConstraints](https://api.flutter.dev/flutter/rendering/BoxConstraints-class.html), [Flex](https://api.flutter.dev/flutter/widgets/Flex-class.html), [Expanded](https://api.flutter.dev/flutter/widgets/Expanded-class.html) và [Flexible](https://api.flutter.dev/flutter/widgets/Flexible-class.html) (api.flutter.dev).
- [LayoutBuilder](https://api.flutter.dev/flutter/widgets/LayoutBuilder-class.html), [Wrap](https://api.flutter.dev/flutter/widgets/Wrap-class.html), [Stack](https://api.flutter.dev/flutter/widgets/Stack-class.html), [Align](https://api.flutter.dev/flutter/widgets/Align-class.html) và [MediaQuery](https://api.flutter.dev/flutter/widgets/MediaQuery-class.html) (api.flutter.dev).
- [TestFlutterView](https://api.flutter.dev/flutter/flutter_test/TestFlutterView-class.html) và [WidgetTester.takeException](https://api.flutter.dev/flutter/flutter_test/WidgetTester/takeException.html) (api.flutter.dev).
