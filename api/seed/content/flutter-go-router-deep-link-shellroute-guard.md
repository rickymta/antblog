---
slug: flutter-go-router-deep-link-shellroute-guard
title: "go_router: deep link, ShellRoute, guard đăng nhập"
summary: "Bài 23: điều hướng app khách Tới Nơi bằng go_router 18.1: StatefulShellRoute cho ba tab giữ state, tham số /trip/:id, redirect async theo đăng nhập Riverpod kèm refreshListenable và from, deep link toinoi://trip/{id}, universal/app link, cấu hình native và 90 test router."
category: programming
level: intermediate
tags: [flutter, dart, mobile, toi-noi, go-router, deep-link, riverpod, ios, android]
language: vi
---

Khách nhận tin nhắn "Tài xế đang tới, xem chuyến T-1002" kèm một đường link. Bấm vào, app Tới Nơi phải mở thẳng màn hình chuyến đó. Nếu khách chưa đăng nhập, app đưa sang màn hình đăng nhập rồi quay lại đúng chuyến vừa bấm. Nếu mã chuyến không tồn tại, khách thấy màn hình lỗi có lối thoát. App vẫn có ba tab Đặt xe, Lịch sử, Tài khoản, và quãng đường khách đã kéo ở tab đầu không được mất khi liếc sang tab khác.

Bài 12 dùng `Navigator.push` và named route, đủ cho luồng từ báo giá sang xác nhận nhưng không trả lời được câu "link này dẫn tới màn hình nào". Ta cần một **Router**: bảng khai báo ánh xạ đường dẫn sang màn hình, đọc được từ cả code lẫn hệ điều hành. Đây là bài 23 của lộ trình **Flutter qua Tới Nơi**, dùng `go_router` 18.1.3. Ta nên đọc [bài 22 về lưu trữ cục bộ](/articles/flutter-local-persistence-drift-secure-storage-lich-su-offline), nơi phiên đăng nhập được lưu, và [bài 18 về Riverpod](/articles/flutter-riverpod-provider-notifier-asyncnotifier-dat-chuyen), vì trạng thái đăng nhập nằm trong một `AsyncNotifier`. Ví dụ dùng `Trip`, `Money`, `calculateFare` của `toi_noi_core` (hồ sơ [Tới Nơi](/articles/du-an-mau-toi-noi)); bảng giá là số minh hoạ.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>App đã khai báo route <code>/trip/:id</code>. Ai đó bấm link <code>toinoi://trip/T-1002</code> và Android mở app. Ta chưa làm gì thêm. Màn hình chuyến T-1002 có hiện ra không?</p>
<details><summary>Xem đáp án</summary><p><strong>Không.</strong> go_router khớp route theo <code>uri.path</code> và bỏ qua scheme lẫn host. Với <code>toinoi://trip/T-1002</code>, <code>trip</code> là <em>host</em> còn <code>path</code> chỉ là <code>/T-1002</code>, không khớp route nào, nên router báo <code>no routes for location</code>. Test <code>router không có bước resolve</code> chứng minh điều này; mục 4 cho cách sửa. Link <code>https://toinoi.example/trip/T-1002</code> thì khớp ngay vì <code>path</code> đúng là <code>/trip/T-1002</code>.</p></details>
</div>

## 1. Khai báo route: tham số và ba tab giữ state

Một **route** là một dòng khai báo: đường dẫn và widget dựng màn hình. Phần `:id` trong `/trip/:id` là **path parameter**, lấy bằng `state.pathParameters['id']`. Ba tab nằm trong `StatefulShellRoute.indexedStack`: mỗi tab là một `StatefulShellBranch` có `Navigator` riêng, và `IndexedStack` giữ cả ba cùng lúc. Màn hình chuyến, đăng nhập và "không tìm thấy" đứng cùng cấp với shell nên chiếm toàn màn hình (Hình 1).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 380" role="img" aria-label="Hình 1. Cây route của app khách. GoRouter có Navigator gốc. Trong đó StatefulShellRoute.indexedStack chứa ba nhánh: Đặt xe tại đường dẫn gốc, Lịch sử tại /history, Tài khoản tại /account, mỗi nhánh có Navigator riêng. NavigationBar trong ShellScaffold gọi goBranch để đổi nhánh. Ba route cùng cấp với shell và chiếm toàn màn hình là /trip/:id, /login và /not-found.">
<defs><marker id="fl23-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="16" y="16" width="848" height="348" rx="12" class="f-none s-muted" stroke-width="1.5" stroke-dasharray="6 5"/><text x="32" y="38" class="t-small">GoRouter: Navigator gốc</text>
<rect x="40" y="56" width="570" height="290" rx="12" class="f-pk-soft s-pk"/><text x="56" y="82" class="t-mono t-pk">StatefulShellRoute.indexedStack</text>
<rect x="56" y="100" width="170" height="120" rx="10" class="f-surface s-line"/><text x="72" y="126" class="t-b">Tab Đặt xe</text><text x="72" y="150" class="t-mono">GoRoute /</text><text x="72" y="172" class="t-small">BookingScreen</text><text x="72" y="194" class="t-small">Navigator riêng</text>
<rect x="240" y="100" width="170" height="120" rx="10" class="f-surface s-line"/><text x="256" y="126" class="t-b">Tab Lịch sử</text><text x="256" y="150" class="t-mono">GoRoute /history</text><text x="256" y="172" class="t-small">HistoryScreen</text><text x="256" y="194" class="t-small">Navigator riêng</text>
<rect x="424" y="100" width="170" height="120" rx="10" class="f-surface s-line"/><text x="440" y="126" class="t-b">Tab Tài khoản</text><text x="440" y="150" class="t-mono">GoRoute /account</text><text x="440" y="172" class="t-small">AccountScreen</text><text x="440" y="194" class="t-small">Navigator riêng</text>
<rect x="56" y="272" width="538" height="56" rx="10" class="f-surface s-line"/><text x="72" y="296" class="t-b">NavigationBar</text><text x="72" y="316" class="t-small">trong ShellScaffold, bấm tab gọi shell.goBranch(index)</text>
<path d="M141 270 L141 222" class="f-none s-pk" stroke-width="2" marker-end="url(#fl23-1-a)"/>
<path d="M325 270 L325 222" class="f-none s-pk" stroke-width="2" marker-end="url(#fl23-1-a)"/>
<path d="M509 270 L509 222" class="f-none s-pk" stroke-width="2" marker-end="url(#fl23-1-a)"/>
<text x="640" y="82" class="t-small">Cùng cấp với shell</text>
<rect x="640" y="96" width="208" height="72" rx="10" class="f-phys-soft s-phys"/><text x="656" y="122" class="t-mono">GoRoute /trip/:id</text><text x="656" y="146" class="t-small">TripScreen(id)</text>
<rect x="640" y="184" width="208" height="72" rx="10" class="f-phys-soft s-phys"/><text x="656" y="210" class="t-mono">GoRoute /login</text><text x="656" y="234" class="t-small">LoginScreen</text>
<rect x="640" y="272" width="208" height="72" rx="10" class="f-phys-soft s-phys"/><text x="656" y="298" class="t-mono">GoRoute /not-found</text><text x="656" y="322" class="t-small">NotFoundScreen</text>
</svg>
</div>
<figcaption>Hình 1. Cây route. Mũi tên chỉ chiều điều khiển: thanh tab chọn nhánh nào đang hiện. Ba route bên phải nằm ngoài shell nên khi mở chúng, thanh tab biến mất.</figcaption>
</figure>

### router.dart · 23-go-router/lib/router.dart (trích)

```dart
List<RouteBase> buildRoutes() => [
  StatefulShellRoute.indexedStack(
    builder: (context, state, shell) => ShellScaffold(shell: shell),
    branches: [
      StatefulShellBranch(
        routes: [
          GoRoute(
            path: AppPaths.booking,
            builder: (context, state) => const BookingScreen(),
          ),
        ],
      ),
      // ... hai nhánh còn lại: AppPaths.history và AppPaths.account
    ],
  ),
  GoRoute(
    path: '/trip/:id',
    builder: (context, state) => TripScreen(id: state.pathParameters['id']!),
  ),
  GoRoute(
    path: AppPaths.login,
    builder: (context, state) => const LoginScreen(),
  ),
  // ... /not-found
];
```

`shell` là `StatefulNavigationShell`, widget chứa các `Navigator` của nhánh. `ShellScaffold` đặt nó vào `body` và thêm `NavigationBar`:

### shell_scaffold.dart · 23-go-router/lib/shell_scaffold.dart (trích)

```dart
return Scaffold(
  body: shell,
  bottomNavigationBar: NavigationBar(
    selectedIndex: shell.currentIndex,
    // Bấm lại tab đang mở thì về màn hình đầu của tab đó.
    onDestinationSelected: (index) =>
        shell.goBranch(index, initialLocation: index == shell.currentIndex),
    destinations: const [
      // Đặt xe, Lịch sử, Tài khoản: xem file trong lab
    ],
  ),
);
```

`goBranch` quay về vị trí cuối của nhánh, hoặc vị trí đầu nếu nhánh chưa mở. Nhánh chưa mở cũng chưa được dựng vì `StatefulShellBranch.preload` mặc định là `false`; test `tab chưa mở thì chưa dựng` kiểm điều đó.

## 2. go và push: ai giữ state của tab

`router.go('/trip/T-1001')` **thay** danh sách route đang hiện bằng danh sách khớp đường dẫn mới. Danh sách mới không có shell, nên shell bị gỡ và mọi `State` bên trong, gồm quãng đường khách đã kéo, bị huỷ. `context.push('/trip/T-1001')` **thêm** một trang lên `Navigator` gốc, shell nằm nguyên bên dưới (Hình 2).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 380" role="img" aria-label="Hình 2. So sánh go và push khi tab Đặt xe đang có quãng đường 12 km. Với router.go tới /trip/T-1001, ngăn xếp chỉ còn TripScreen, shell bị gỡ và giá trị 12 km mất; quay lại tab thì quãng đường về 5 km mặc định. Với context.push, ngăn xếp gồm shell giữ nguyên 12 km ở dưới và TripScreen ở trên; Back trả về đúng chỗ cũ.">
<defs><marker id="fl23-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl23-2-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<text x="20" y="30" class="t-b">router.go('/trip/T-1001')</text>
<rect x="20" y="46" width="390" height="70" rx="10" class="f-pk-soft s-pk"/><text x="36" y="72" class="t-b">Trước: shell + tab Đặt xe</text><text x="36" y="96" class="t-small">quãng đường khách đã chọn: 12 km</text>
<path d="M215 118 L215 154" class="f-none s-pk" stroke-width="2" marker-end="url(#fl23-2-a)"/><text x="228" y="142" class="t-small">go: thay cả danh sách route</text>
<rect x="20" y="158" width="390" height="70" rx="10" class="f-phys-soft s-phys"/><text x="36" y="184" class="t-b">Sau: chỉ còn TripScreen</text><text x="36" y="208" class="t-small">ngăn xếp có một trang</text>
<rect x="20" y="246" width="390" height="100" rx="10" class="f-warn-soft s-warn" stroke-dasharray="6 5"/><text x="36" y="272" class="t-b t-warn">Shell đã bị gỡ khỏi cây</text><text x="36" y="296" class="t-small">State của BookingScreen bị huỷ.</text><text x="36" y="318" class="t-small">Về lại tab: quãng đường là 5 km mặc định.</text>
<text x="470" y="30" class="t-b">context.push('/trip/T-1001')</text>
<rect x="470" y="46" width="390" height="70" rx="10" class="f-pk-soft s-pk"/><text x="486" y="72" class="t-b">Trước: shell + tab Đặt xe</text><text x="486" y="96" class="t-small">quãng đường khách đã chọn: 12 km</text>
<path d="M665 118 L665 154" class="f-none s-pk" stroke-width="2" marker-end="url(#fl23-2-a)"/><text x="678" y="142" class="t-small">push: thêm một trang</text>
<rect x="470" y="158" width="390" height="56" rx="10" class="f-phys-soft s-phys"/><text x="486" y="182" class="t-b">TripScreen</text><text x="486" y="202" class="t-small">trang trên cùng của Navigator gốc</text>
<rect x="470" y="226" width="390" height="70" rx="10" class="f-pk-soft s-pk"/><text x="486" y="252" class="t-b">Shell + tab Đặt xe, giữ nguyên</text><text x="486" y="276" class="t-small">quãng đường vẫn là 12 km</text>
<text x="470" y="326" class="t-small">Back: pop TripScreen, về đúng chỗ cũ.</text>
</svg>
</div>
<figcaption>Hình 2. Cùng một đích, hai cách đi. Với go, trang cũ không còn trong danh sách nên bị huỷ. Với push, trang cũ vẫn nằm dưới và giữ state.</figcaption>
</figure>

Test kéo thanh trượt tới 12 km: `push` rồi Back thì còn 12, `go` tới chuyến rồi `go('/')` thì về 5.

### trip_screen.dart · 23-go-router/lib/trip_screen.dart (trích)

```dart
void _back(BuildContext context) {
  if (context.canPop()) {
    context.pop();
  } else {
    context.go(AppPaths.history);
  }
}
```

Màn hình chuyến mở được từ danh sách lẫn từ link nên nút Back phải xử lý cả hai. Mở bằng `push` thì còn trang bên dưới để `pop`. Mở từ link thì ngăn xếp chỉ có một trang, `canPop()` là `false`, và Back đưa khách tới `/history` thay vì thoát app. Một chi tiết dễ nhầm khi viết test: sau `push`, `routeInformationProvider.value` và `currentConfiguration.uri` vẫn là vị trí của phần khai báo (`/`), còn `router.state.uri` là `/trip/T-1001`. Muốn biết màn hình trên cùng, đọc `router.state`.

## 3. Guard đăng nhập: redirect, refreshListenable và from

`redirect` chạy **trước khi** router dựng màn hình: nhận vị trí muốn vào và trả `null` (cho vào) hoặc một đường dẫn khác. Ta tách logic ra hàm thuần `guardLocation` để test không cần widget:

### guard.dart · 23-go-router/lib/guard.dart (trích)

```dart
const _publicPaths = {AppPaths.booking, AppPaths.login, AppPaths.notFound};

bool requiresLogin(String path) => !_publicPaths.contains(path);

Future<String?> guardLocation(
  Uri uri,
  Future<Session?> Function() readSession,
) async {
  // 1. Link vào app: đổi thành đường dẫn nội bộ rồi để vòng redirect chạy
  //    lại với đường dẫn mới (go_router lặp tới khi redirect trả null).
  final link = DeepLinks.resolve(uri);
  if (link != null) return link;

  // 2. Trang công khai (trừ /login): vào luôn, không chờ kho phiên.
  final path = uri.path;
  if (path != AppPaths.login && !requiresLogin(path)) return null;

  // 3. Còn lại: đọc phiên rồi quyết định.
  final signedIn = await readSession() != null;

  // Đã đăng nhập mà còn ở /login: đi tiếp tới nơi định tới.
  if (path == AppPaths.login) {
    return signedIn ? safeFrom(uri.queryParameters['from']) : null;
  }

  // Chưa đăng nhập mà vào trang riêng: sang /login, nhớ nơi định tới.
  if (!signedIn && requiresLogin(path)) {
    return AppPaths.loginFor(_locationOf(uri));
  }
  return null;
}
```

Danh sách công khai chỉ có ba đường dẫn; mọi thứ khác cần đăng nhập, kể cả đường dẫn chưa khai báo. Quên một route mà mặc định là "chặn" thì chỉ gây phiền, mặc định là "cho vào" thì lộ dữ liệu. Hàm trả `Future` vì lúc khởi động phiên còn đang đọc từ kho (bài 22). Test `đang đọc kho phiên` cho thấy trong lúc chờ, router chưa dựng màn hình nào, nên bước 2 cho trang chủ vào luôn.

`from` cần cẩn thận vì ai cũng gõ được tham số query, và sau đăng nhập app đi tới đó. `safeFrom` chỉ nhận đường dẫn nội bộ; `https://evil.example/...`, `//evil.example`, URI có scheme hay chính `/login` đều thành `/`.

`redirect` chỉ chạy khi có điều hướng, nên đăng nhập xong mà không có điều hướng nào thì khách vẫn ở `/login`. `GoRouter.refreshListenable` giải việc đó: mỗi lần `Listenable` phát tín hiệu, router đánh giá lại `redirect` với vị trí **khai báo** hiện tại (phần `go`). Trạng thái đăng nhập nằm trong Riverpod nên ta bắc một cầu nhỏ, kèm một bước kiểm riêng khi mất phiên:

### router.dart · 23-go-router/lib/router.dart (trích)

```dart
final class AuthRefresh extends ChangeNotifier {
  void ping() => notifyListeners();
}

Future<Session?> _currentSession(Ref ref) {
  final auth = ref.read(authProvider);
  if (auth.hasValue) return Future.value(auth.value);
  return ref.read(authProvider.future);
}

final routerProvider = Provider<GoRouter>((ref) {
  final refresh = AuthRefresh();
  late final GoRouter router;

  // `refreshListenable` chạy lại redirect với vị trí của phần khai báo (go),
  // không thấy trang đẩy bằng push. Mất phiên thì ta tự kiểm trang trên cùng.
  Future<void> recheckTopPage() async {
    if (router.routerDelegate.currentConfiguration.isEmpty) return;
    final target = await guardLocation(
      router.state.uri,
      () => _currentSession(ref),
    );
    if (target != null && ref.mounted) router.go(target);
  }

  ref.listen(authProvider, (_, next) {
    refresh.ping();
    if (next.hasValue && next.value == null) recheckTopPage();
  });

  router = GoRouter(
    initialLocation: AppPaths.booking,
    refreshListenable: refresh,
    redirect: (context, state) =>
        guardLocation(state.uri, () => _currentSession(ref)),
    // Không khớp route nào (hoặc redirect lỗi): tới màn hình "không tìm thấy".
    onException: (context, state, router) =>
        router.go(AppPaths.notFoundFor(state.uri)),
    routes: buildRoutes(),
  );

  ref.onDispose(() {
    router.dispose();
    refresh.dispose();
  });
  return router;
});
```

Ba chi tiết đều có test. Một, provider dùng `ref.listen` và `ref.read`, không dùng `ref.watch`. Hai, `_currentSession` đọc thẳng state khi đã có giá trị và chỉ chờ `.future` khi kho còn đang đọc. Ba, `LoginScreen` không biết `from` và không tự điều hướng: nó chỉ gọi `signIn`, rồi `refreshListenable` làm router đánh giá lại `/login?from=%2Ftrip%2FT-1002` và `safeFrom` đưa khách tới chuyến. Đăng xuất đi cùng đường: nút chỉ đổi state, `redirect` đưa khách về `/login?from=%2Faccount`.

Có một giới hạn, nối với chi tiết `router.state` ở mục 2: `redirect` chỉ thấy vị trí khai báo, không thấy trang đẩy bằng `push`. Đang ở `/`, `push('/trip/T-1001')` rồi `signOut()`: redirect chỉ thấy `/` (công khai) nên `TripScreen` vẫn hiện dù phiên đã mất. Test `chỉ refreshListenable thì trang push vẫn còn sau đăng xuất` tái hiện. Vì vậy `recheckTopPage` chạy `guardLocation` với `router.state.uri`, tức trang trên cùng, mỗi khi mất phiên, và `go` tới `/login?from=...` nếu bị chặn; ngăn xếp bị thay nên không còn đường Back về dữ liệu cũ. Phương án khác là mở mọi trang riêng bằng `go`, nhưng ta mất state tab.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 470" role="img" aria-label="Hình 3. Các bước của guardLocation. Từ vị trí cần vào, bước 1 hỏi link có scheme không: nếu có thì trả về đường dẫn nội bộ và vòng redirect chạy lại. Bước 2 hỏi trang có công khai không: nếu có thì trả null để cho vào mà không chờ kho phiên. Bước 3 đọc phiên rồi quyết định: ở /login mà đã đăng nhập thì đi tới safeFrom của from; trang riêng mà chưa đăng nhập thì sang /login kèm from; còn lại trả null. Khi authProvider đổi, AuthRefresh phát tín hiệu và router chạy lại guard với vị trí khai báo; khi mất phiên còn kiểm cả trang đẩy bằng push.">
<defs><marker id="fl23-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl23-3-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="20" y="16" width="340" height="52" rx="10" class="f-phys-soft s-phys"/><text x="36" y="38" class="t-b">Vị trí cần vào</text><text x="36" y="58" class="t-mono">state.uri</text>
<path d="M190 70 L190 98" class="f-none s-pk" stroke-width="2" marker-end="url(#fl23-3-a)"/>
<rect x="20" y="100" width="340" height="60" rx="10" class="f-pk-soft s-pk"/><text x="36" y="124" class="t-b">1. Link có scheme?</text><text x="36" y="146" class="t-mono">DeepLinks.resolve(uri)</text>
<path d="M362 130 L458 130" class="f-none s-pk" stroke-width="2" marker-end="url(#fl23-3-a)"/><text x="392" y="122" class="t-small">có</text>
<rect x="460" y="100" width="400" height="60" rx="10" class="f-surface s-line"/><text x="476" y="124" class="t-b">Trả đường dẫn nội bộ</text><text x="476" y="146" class="t-small">ví dụ /trip/T-1002, vòng redirect chạy lại</text>
<path d="M190 162 L190 190" class="f-none s-pk" stroke-width="2" marker-end="url(#fl23-3-a)"/><text x="204" y="182" class="t-small">không</text>
<rect x="20" y="192" width="340" height="60" rx="10" class="f-pk-soft s-pk"/><text x="36" y="216" class="t-b">2. Trang công khai?</text><text x="36" y="238" class="t-small">/ và /not-found (không tính /login)</text>
<path d="M362 222 L458 222" class="f-none s-pk" stroke-width="2" marker-end="url(#fl23-3-a)"/><text x="392" y="214" class="t-small">có</text>
<rect x="460" y="192" width="400" height="60" rx="10" class="f-surface s-line"/><text x="476" y="216" class="t-b">Trả null: cho vào</text><text x="476" y="238" class="t-small">không chờ kho phiên</text>
<path d="M190 254 L190 282" class="f-none s-pk" stroke-width="2" marker-end="url(#fl23-3-a)"/><text x="204" y="274" class="t-small">không</text>
<rect x="20" y="284" width="340" height="100" rx="10" class="f-pk-soft s-pk"/><text x="36" y="308" class="t-b">3. Đọc phiên, rồi quyết định</text><text x="36" y="330" class="t-small">chờ nếu kho phiên còn đang đọc</text><text x="36" y="352" class="t-mono">signedIn = phiên != null</text>
<path d="M362 334 L458 334" class="f-none s-pk" stroke-width="2" marker-end="url(#fl23-3-a)"/>
<rect x="460" y="284" width="400" height="100" rx="10" class="f-surface s-line"/><text x="476" y="308" class="t-small">ở /login và đã đăng nhập: safeFrom(from)</text><text x="476" y="334" class="t-small">trang riêng, chưa đăng nhập: /login?from=</text><text x="476" y="360" class="t-small">còn lại: null</text>
<path d="M190 418 L190 386" class="f-none s-warn" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#fl23-3-w)"/>
<rect x="20" y="420" width="840" height="40" rx="10" class="f-warn-soft s-warn" stroke-dasharray="6 5"/><text x="36" y="445" class="t-small">authProvider đổi: ping() chạy lại guard với vị trí khai báo; mất phiên thì kiểm cả trang push</text>
</svg>
</div>
<figcaption>Hình 3. Guard chạy theo thứ tự từ trên xuống và dừng ở bước đầu tiên có kết quả. Đường đứt là tín hiệu làm guard chạy lại khi trạng thái đăng nhập đổi, không phải một điều hướng mới.</figcaption>
</figure>

## 4. Deep link: từ hệ điều hành tới route

Khi khách bấm link, hệ điều hành đưa URI cho **engine**. Theo mã nguồn engine 3.47.4, Android lấy `Intent.getData().toString()` còn iOS gửi `url.absoluteString`, cả hai qua thông điệp `pushRouteInformation` của kênh `flutter/navigation`. Xử lý này bật sẵn: khoá `flutter_deeplinking_enabled` (Android) hay `FlutterDeepLinkingEnabled` (iOS) vắng thì coi như bật. Phần Dart gọi `didPushRouteInformation` cho các observer, và `GoRouteInformationProvider` là một trong số đó (Hình 4).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 498" role="img" aria-label="Hình 4. Đường đi của một deep link khi app đang chạy. Hệ điều hành đưa URI cho engine. Engine gửi pushRouteInformation với URI đầy đủ qua kênh flutter/navigation. WidgetsBinding gọi didPushRouteInformation. GoRouteInformationProvider giữ RouteInformation hiện tại. Parser và redirect gọi DeepLinks.resolve đổi toinoi://trip/T-1002 thành /trip/T-1002 rồi guard kiểm phiên. Cuối cùng route /trip/:id khớp và dựng TripScreen. Nếu thiếu bước resolve, path chỉ là /T-1002 và không khớp route nào. Khi khởi động nguội trên Android, engine đặt defaultRouteName bằng URI và GoRouter dùng nó làm vị trí đầu; trên iOS app mở ở đường dẫn gốc rồi nhận link sau khung hình đầu tiên.">
<defs><marker id="fl23-4-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="16" y="16" width="370" height="44" rx="10" class="f-phys-soft s-phys"/><text x="32" y="43" class="t-b">Hệ điều hành</text><text x="420" y="43" class="t-small">Android: Intent.getData()</text><text x="420" y="52" class="t-small">iOS: scene/openURL, universal link: webpageURL</text>
<path d="M200 62 L200 78" class="f-none s-pk" stroke-width="2" marker-end="url(#fl23-4-a)"/>
<rect x="16" y="80" width="370" height="44" rx="10" class="f-phys-soft s-phys"/><text x="32" y="107" class="t-b">Engine, kênh flutter/navigation</text><text x="420" y="107" class="t-small">pushRouteInformation, location là URI đầy đủ</text>
<path d="M200 126 L200 142" class="f-none s-pk" stroke-width="2" marker-end="url(#fl23-4-a)"/>
<rect x="16" y="144" width="370" height="44" rx="10" class="f-pk-soft s-pk"/><text x="32" y="171" class="t-mono">WidgetsBinding.didPushRouteInformation</text><text x="420" y="171" class="t-small">gọi từng WidgetsBindingObserver</text>
<path d="M200 190 L200 206" class="f-none s-pk" stroke-width="2" marker-end="url(#fl23-4-a)"/>
<rect x="16" y="208" width="370" height="44" rx="10" class="f-pk-soft s-pk"/><text x="32" y="235" class="t-mono">GoRouteInformationProvider</text><text x="420" y="235" class="t-small">giữ RouteInformation(uri) làm giá trị hiện tại</text>
<path d="M200 254 L200 270" class="f-none s-pk" stroke-width="2" marker-end="url(#fl23-4-a)"/>
<rect x="16" y="272" width="370" height="44" rx="10" class="f-pk-soft s-pk"/><text x="32" y="299" class="t-b">Parser + redirect (guardLocation)</text><text x="420" y="299" class="t-small">resolve đổi sang /trip/T-1002, rồi guard kiểm phiên</text>
<path d="M200 318 L200 334" class="f-none s-pk" stroke-width="2" marker-end="url(#fl23-4-a)"/>
<rect x="16" y="336" width="370" height="44" rx="10" class="f-pk-soft s-pk"/><text x="32" y="363" class="t-b">Khớp route /trip/:id, dựng TripScreen</text><text x="420" y="363" class="t-small">pathParameters['id'] = T-1002</text>
<text x="420" y="387" class="t-small t-warn">Thiếu resolve: path chỉ là /T-1002, không khớp</text>
<rect x="16" y="410" width="848" height="72" rx="10" class="f-pk-soft s-pk" stroke-dasharray="6 5"/><text x="32" y="434" class="t-b">Khởi động nguội</text><text x="32" y="454" class="t-small">Android: engine đặt PlatformDispatcher.defaultRouteName bằng URI, GoRouter lấy nó làm vị trí đầu.</text><text x="32" y="472" class="t-small">iOS: app mở ở đường dẫn gốc, link tới bằng pushRouteInformation sau khung hình đầu tiên.</text>
</svg>
</div>
<figcaption>Hình 4. Luồng đi từ link tới màn hình khi app đang chạy. Mũi tên chỉ chiều dữ liệu. Khi khởi động nguội trên Android, link vào qua <code>defaultRouteName</code> và đi tiếp từ khâu parser.</figcaption>
</figure>

Khi app **chưa chạy** thì hai nền tảng khác nhau. Trên Android, engine đặt URI làm `defaultRouteName`; nếu khác `/` thì `GoRouter` dùng nó thay `initialLocation`. Trên iOS, tài liệu Flutter ghi app mở ở `/` rồi nhận link ngay sau đó; mã engine chỉ chờ khung hình đầu tối đa 3 giây, nên màn hình đầu phải lên hình nhanh (thêm một lý do để trang chủ không chờ kho phiên). Mọi đường vào đều đi qua parser và `redirect`. Ta tận dụng điều đó: go_router khớp theo `uri.path`, nhưng `state.uri` trong `redirect` cấp router vẫn giữ nguyên scheme và host, nên ta đổi link về đường dẫn nội bộ ngay tại đó.

### deep_links.dart · 23-go-router/lib/deep_links.dart (trích)

```dart
static String? resolve(Uri uri) {
  if (!uri.hasScheme) return null;
  final query = uri.hasQuery ? '?${uri.query}' : '';

  if (uri.scheme == scheme) {
    if (!_customHosts.contains(uri.host)) {
      return AppPaths.notFoundFor(uri);
    }
    return '/${uri.host}${uri.path}$query';
  }

  final isWeb = uri.scheme == 'https' || uri.scheme == 'http';
  if (isWeb && uri.host == webHost) {
    return '${uri.path.isEmpty ? '/' : uri.path}$query';
  }
  // Scheme lạ hoặc tên miền khác: không phải link của ta.
  return AppPaths.notFoundFor(uri);
}
```

Custom scheme: `host` là tên màn hình nên ta ghép lại thành `/trip/T-1002`. Universal link: ta chỉ bỏ origin, nên `from` của guard là đường dẫn chứ không phải cả URL. Tên miền và scheme lạ đi tới `/not-found`. Test dựng hai đường vào: `launchWithLink` (kiểu Android, đặt `defaultRouteNameTestValue`) và `receiveLink` (thông điệp engine gửi lúc app đang chạy, cũng là đường của iOS).

## 5. Cấu hình native: hệ điều hành chỉ mở app khi ta khai báo

Code Dart chỉ chạy sau khi hệ điều hành đã chọn app của ta, và việc chọn nằm ở file native. Lab commit đúng các file đó trong `native-config/` dưới dạng bản mẫu `.example` (runner `android/`, `ios/` không commit), với tên miền hư cấu `toinoi.example`, Team ID và vân tay chứng chỉ giả:

| File mẫu | Đặt vào | Làm gì |
|---|---|---|
| `AndroidManifest.xml.example` | `android/app/src/main/` | `intent-filter` cho `toinoi://trip` và `https://toinoi.example/trip/` (`autoVerify`) |
| `Info.plist.example` | `ios/Runner/` | `CFBundleURLTypes` đăng ký scheme `toinoi` |
| `Runner.entitlements.example` | `ios/Runner/` | `applinks:toinoi.example` (Associated Domains) |
| `apple-app-site-association.example` | server, `/.well-known/`, bỏ `.example`, không `.json` | iOS xác nhận tên miền thuộc app `<TeamID>.<bundle id>` |
| `assetlinks.json.example` | server, `/.well-known/`, bỏ `.example` | Android xác nhận tên miền thuộc package có vân tay này |

### AndroidManifest.xml.example · 23-go-router/native-config/android/AndroidManifest.xml.example (trích)

```xml
<intent-filter>
    <action android:name="android.intent.action.VIEW"/>
    <category android:name="android.intent.category.DEFAULT"/>
    <category android:name="android.intent.category.BROWSABLE"/>
    <data android:scheme="toinoi" android:host="trip"/>
</intent-filter>

<intent-filter android:autoVerify="true">
    <action android:name="android.intent.action.VIEW"/>
    <category android:name="android.intent.category.DEFAULT"/>
    <category android:name="android.intent.category.BROWSABLE"/>
    <data
        android:scheme="https"
        android:host="toinoi.example"
        android:pathPrefix="/trip/"/>
</intent-filter>
```

Với app nhắm Android 12 trở lên, activity có `intent-filter` mà thiếu `android:exported` thì không cài được (tài liệu Android); file `flutter create` sinh đã có sẵn. Với `launchMode="singleTop"`, khi activity đang ở trên cùng, link đi vào `onNewIntent` và mã engine chuyển nó thành `pushRouteInformation`. Đừng trộn `<data>` vào bộ lọc `MAIN`/`LAUNCHER`. Khi một bộ lọc có `autoVerify`, hệ thống xét các bộ lọc `VIEW` + `DEFAULT` + `BROWSABLE` có scheme `http` hoặc `https`, rồi tải `https://<host>/.well-known/assetlinks.json` cho từng host.

Custom scheme ai cũng đăng ký được, nên chỉ dùng cho việc không nhạy cảm và luôn xác thực lại ở server. Universal link và App Link gắn với tên miền ta kiểm soát và đã được hệ điều hành xác minh, nên là đường chính cho link gửi qua tin nhắn. Kiểm lại tại thời điểm phát hành: Flutter ghi CDN của Apple có thể mất tới 24 giờ để lấy file AASA, Android 15 trở lên xác minh lại định kỳ nên đổi `assetlinks.json` có thể mất tới bảy ngày. Flavor `staging` có application ID khác nên cần dòng riêng trong cả hai file.

Test `native_config_test.dart` đọc từng file bằng `dart:io` và gói `xml`: scheme và host của manifest trùng hằng của `DeepLinks`, `autoVerify` và `exported` có mặt, `applinks:` không kèm `https://`, vân tay đủ 32 byte, hai file server nói về cùng một app và cùng nhánh `/trip/`. Bảy link đi qua bản rút gọn của bộ lọc Android: ba được nhận, bốn bị loại (`/pricing`, tên miền lạ, `http`, host `payment`). Đây là kiểm tra nhất quán, không chứng minh hệ điều hành chấp nhận.

## 6. Link hỏng và mã không tồn tại

Có hai loại "không có gì để hiện". Loại một là đường dẫn không khớp route nào (`/khong-co-trang-nay`, `toinoi://trip` thiếu mã, `https://toinoi.example/pricing`). Router tạo `GoException` và gọi `onException`; callback này **thay** `errorBuilder` và `errorPageBuilder`, truyền từ hai cái trở lên sẽ vi phạm `assert` trong constructor. Ta đưa khách tới `/not-found?uri=...` để màn hình hiện lại link đã nhận kèm nút "Về trang chủ".

Loại hai là route khớp nhưng dữ liệu không có: `/trip/T-9999`. Với router đây là điều hướng thành công; chính `TripScreen` xử lý `TripNotFound`:

### trip_screen.dart · 23-go-router/lib/trip_screen.dart (trích)

```dart
AsyncError(:final error) when error is TripNotFound => Column(
  crossAxisAlignment: CrossAxisAlignment.start,
  children: [
    Text('Không tìm thấy chuyến ${error.id}.'),
    const SizedBox(height: ToiNoiSpacing.md),
    FilledButton(
      key: const Key('back-to-history'),
      onPressed: () => context.go(AppPaths.history),
      child: const Text('Về lịch sử'),
    ),
  ],
),
```

`TripNotFound` là `Exception`, và Riverpod 3 mặc định thử lại provider lỗi `Exception` tới 10 lần, nên khách sẽ nhìn vòng quay gần 40 giây. `tripByIdProvider` tắt retry bằng `retry: (_, _) => null`, và test `TripNotFound không bị Riverpod thử lại 10 lần` đếm đúng một lần gọi repository. Khách chưa đăng nhập vào đường dẫn lạ thì qua `/login` trước rồi mới tới `/not-found`, vì danh sách công khai chỉ có ba đường dẫn.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp với go_router</span>

<p><strong>1. Link custom scheme không mở được màn hình.</strong> Dấu hiệu: màn hình lỗi mặc định với <code>no routes for location: toinoi://trip/T-1002</code>. Nguyên nhân: router khớp theo path, path ở đây chỉ là <code>/T-1002</code>. Cách sửa: đổi link về đường dẫn nội bộ trong <code>redirect</code> cấp router. Test <code>router không có bước resolve</code> tái hiện, test kế bên chứng minh cách sửa.</p>

<p><strong>2. Đăng nhập xong vẫn kẹt ở /login.</strong> Dấu hiệu: state đã đổi mà màn hình đứng yên. Nguyên nhân: thiếu <code>refreshListenable</code>. Cách sửa: truyền một <code>Listenable</code> phát tín hiệu khi trạng thái đổi. Test <code>không có: đăng nhập xong vẫn kẹt</code> phải gọi <code>router.refresh()</code> thủ công mới thoát.</p>

<p><strong>3. Có <code>refreshListenable</code> mà vẫn kẹt.</strong> Dấu hiệu: <code>ping()</code> được gọi nhưng <code>redirect</code> vẫn thấy chưa đăng nhập. Nguyên nhân: <code>redirect</code> chờ <code>authProvider.future</code> đúng lúc state vừa đổi, future có thể còn trả phiên cũ. Cách sửa: đã có giá trị thì đọc <code>ref.read(authProvider)</code>, chỉ chờ <code>.future</code> khi còn đang tải. Test <code>future trả giá trị cũ trong listener, state thì đã mới</code> chứng minh; lỗi này xuất hiện thật khi viết lab.</p>

<p><strong>4. Tab mất state sau đăng nhập.</strong> Dấu hiệu: <code>GoRouter</code> bị tạo lại, quãng đường về mặc định. Nguyên nhân: provider của router dùng <code>ref.watch(authProvider)</code> nên mỗi lần đổi trạng thái nó dựng router mới. Cách sửa: <code>ref.listen</code> cộng <code>refreshListenable</code>. Test <code>đổi trạng thái đăng nhập tạo ra GoRouter khác</code> chứng minh lỗi, test kế bên chứng minh bản của lab.</p>

<p><strong>5. <code>from</code> không được kiểm.</strong> Dấu hiệu: <code>/login?from=//evil.example</code> đưa người dùng đi nơi lạ sau đăng nhập. Nguyên nhân: tin <code>from</code> là đường dẫn nội bộ. Cách sửa: <code>safeFrom</code> (mục 3). Test <code>from giả mạo trỏ ra ngoài</code> bảo vệ điều này.</p>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/23-go-router
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` với **90 test**. Đã kiểm với Flutter 3.47.4, Dart 3.13.3, `go_router` 18.1.3, `flutter_riverpod` 3.5.0 (bài 18 kiểm ở 3.4.3; đây là bản mới nhất khi `flutter pub add`) và `xml` 7.1.0 cho test đọc file cấu hình. Mức kiểm chứng: **Kiểm chứng tự động**.

| Test | Chứng minh điều gì |
|---|---|
| `guard_test.dart` (12) | `safeFrom` từ chối link ngoài, `//host`, scheme, `/login`; `guardLocation` chặn, cho vào, chuyển `from` |
| `deep_links_test.dart` (8) | `host` và `path` của `toinoi://trip/T-1002`; ghép lại; tên miền và scheme lạ vào `/not-found` |
| `router_guard_test.dart` (15) | Chặn rồi quay lại `from`, đăng xuất (cả khi trang riêng đang hiện từ push), kho phiên chậm hoặc hỏng |
| `shell_test.dart` (7) | Đổi tab giữ 12 km; `push` giữ state, `go` làm mất |
| `trip_route_test.dart` (11) | Cước 12.005, 25.500, 453.000 VND đúng công thức; id lạ; Back; `onException` |
| `deep_link_flow_test.dart` (15) | Khởi động nguội và link khi app chạy, hai dạng link, `from` là đường dẫn |
| `pitfalls_test.dart` (8) | Bốn lỗi đầu của mục trên, mỗi lỗi có bản sai lẫn bản đúng |
| `native_config_test.dart` (14) | Giá trị trong `native-config/`, nhất quán chéo với `DeepLinks` |

Ta thử phá để chắc test bắt được lỗi: bỏ `DeepLinks.resolve` làm 12 test đỏ, bỏ `refreshListenable` 8 test, luôn đợi `authProvider.future` 9 test, bỏ `safeFrom` 2 test, bỏ `recheckTopPage` 1 test, đổi `android:host` trong manifest mẫu 2 test.

**Chưa kiểm** (cần thiết bị, tài khoản hoặc tên miền thật): hệ điều hành có thật sự mở app khi bấm link; intent-filter, entitlement, Associated Domains được chấp nhận; xác minh App Link và Universal Link với server thật (`toinoi.example` hư cấu, vân tay và Team ID giả); nút Back hệ thống sau khi mở bằng link; chọn app khi nhiều app cùng nhận một scheme. Việc engine gửi URI đầy đủ chỉ được xác nhận bằng đọc mã nguồn engine 3.47.4 và test mô phỏng thông điệp. README của lab có cách tự thử (`adb shell am start`, `xcrun simctl openurl`). Chưa dùng `onEnter`, restoration state hay `go_router_builder`.

## Bài tập

1. Thêm màn hình khuyến mãi `/promo/:code` xem được khi chưa đăng nhập, mở được bằng `toinoi://promo/SUMMER` và `https://toinoi.example/promo/SUMMER`. Cần sửa những chỗ nào?
2. App đang ở tab Lịch sử, khách đã `push` chuyến T-1001. Link `toinoi://trip/T-1002` tới. Ngăn xếp sau đó trông thế nào, `canPop()` trả gì?
3. Link `https://toinoi.example/trip/T-1002` tới khi khách chưa đăng nhập. Giá trị `from` được lưu là gì, và vì sao không phải cả URL?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Thêm <code>GoRoute</code> cho <code>/promo/:code</code>; sửa <code>requiresLogin</code> để đường dẫn bắt đầu bằng <code>/promo/</code> là công khai (danh sách hiện so khớp chính xác); thêm host <code>promo</code> vào <code>_customHosts</code>. Phía native thêm <code>&lt;data android:scheme="toinoi" android:host="promo"/&gt;</code>, một bộ lọc https với <code>pathPrefix="/promo/"</code> và <code>{"/": "/promo/*"}</code> trong AASA; cập nhật <code>native_config_test.dart</code> vì test kiểm danh sách chính xác.</p>
<p><strong>2.</strong> Link được xử lý bằng <code>go</code> nên cả ngăn xếp bị thay: chỉ còn <code>TripScreen</code> của T-1002, shell và chuyến T-1001 biến mất, <code>canPop()</code> là <code>false</code>. Test <code>custom scheme: mở chuyến, thay cả ngăn xếp hiện tại</code> chứng minh.</p>
<p><strong>3.</strong> <code>/trip/T-1002</code>. Vòng redirect đầu, <code>DeepLinks.resolve</code> bỏ <code>https://toinoi.example</code>; vòng sau guard dựng <code>from</code> chỉ từ đường dẫn và query. Nếu <code>from</code> là cả URL thì <code>safeFrom</code> từ chối (có scheme) và khách đăng nhập xong về trang chủ. Test <code>chưa đăng nhập: sang /login với from là đường dẫn, không phải link</code> kiểm đúng <code>/login?from=%2Ftrip%2FT-1002</code>.</p>
</details>

Bài trước: [Lưu trữ cục bộ: drift, shared_preferences và secure storage](/articles/flutter-local-persistence-drift-secure-storage-lich-su-offline). Bài tiếp: [Authentication: OTP, JWT, refresh token rotation và interceptor](/articles/flutter-authentication-otp-jwt-refresh-token-secure-storage).

## Tài liệu gốc

- [Deep linking](https://docs.flutter.dev/ui/navigation/deep-linking), [Set up app links for Android](https://docs.flutter.dev/cookbook/navigation/set-up-app-links) và [Set up universal links for iOS](https://docs.flutter.dev/cookbook/navigation/set-up-universal-links) (docs.flutter.dev).
- [go_router](https://pub.dev/packages/go_router) và [tài liệu API](https://pub.dev/documentation/go_router/latest/go_router/go_router-library.html) (pub.dev), gồm `StatefulShellRoute`, `redirect`, `refreshListenable`, `onException`.
- [Verify Android App Links](https://developer.android.com/training/app-links/verify-android-applinks) và [Behavior changes: Android 12](https://developer.android.com/about/versions/12/behavior-changes-12) (developer.android.com).
- [Supporting associated domains](https://developer.apple.com/documentation/xcode/supporting-associated-domains) (developer.apple.com).
- [RouteInformation](https://api.flutter.dev/flutter/widgets/RouteInformation-class.html) và [WidgetsBindingObserver.didPushRouteInformation](https://api.flutter.dev/flutter/widgets/WidgetsBindingObserver/didPushRouteInformation.html) (api.flutter.dev).
- [flutter_riverpod](https://pub.dev/packages/flutter_riverpod) và [Automatic Retry](https://riverpod.dev/docs/concepts2/retry) (pub.dev, riverpod.dev).
