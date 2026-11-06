import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'trip_history_source.dart';
import 'trip_history_tile.dart';

/// Màn hình lịch sử chuyến: app bar co giãn, tóm tắt, danh sách phân trang,
/// kéo xuống để làm mới, cuộn gần cuối thì tải thêm.
///
/// Màn hình cần nhớ danh sách đã tải, trạng thái đang tải và lỗi, nên là
/// `StatefulWidget`. Bài này chỉ dùng `setState` ở mức tối thiểu để báo "dữ
/// liệu đổi, build lại"; bài 10 giải thích `State` và vòng đời đầy đủ.
class TripHistoryScreen extends StatefulWidget {
  const TripHistoryScreen({
    super.key,
    required this.source,
    this.pageSize = 20,
    this.loadMoreThreshold = 300,
  }) : assert(pageSize > 0);

  final TripHistorySource source;

  /// Số chuyến xin mỗi lần gọi nguồn.
  final int pageSize;

  /// Còn ít hơn bấy nhiêu pixel nữa là hết danh sách thì tải trang kế.
  final double loadMoreThreshold;

  @override
  State<TripHistoryScreen> createState() => _TripHistoryScreenState();
}

class _TripHistoryScreenState extends State<TripHistoryScreen> {
  final _controller = ScrollController();
  final _trips = <Trip>[];

  // Đang chờ nguồn trả lời. Mang giá trị true ngay từ đầu vì initState bắt đầu
  // tải trang đầu trước khi build lần đầu.
  bool _loading = true;
  bool _hasMore = true;
  Object? _error;

  // Tăng mỗi lần làm mới. Câu trả lời của lượt cũ (generation nhỏ hơn) bị bỏ.
  int _generation = 0;

  @override
  void initState() {
    super.initState();
    _controller.addListener(_onScroll);
    // _loading đã là true từ khai báo, nên không cần setState trước build đầu.
    _fetch(reset: true);
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  void _onScroll() {
    if (!_controller.hasClients) return;
    if (_controller.position.extentAfter < widget.loadMoreThreshold) {
      _loadMore();
    }
  }

  void _loadMore() {
    // Cả ba điều kiện đều cần: đang tải thì không gọi chồng, hết trang thì
    // không gọi nữa, đang lỗi thì chờ người dùng bấm "Thử lại".
    if (_loading || !_hasMore || _error != null) return;
    _load(reset: false);
  }

  /// Bật cờ đang tải rồi gọi [_fetch]. Hoàn thành khi nguồn trả lời.
  Future<void> _load({required bool reset}) {
    setState(() {
      _loading = true;
      _error = null;
    });
    return _fetch(reset: reset);
  }

  /// Hỏi nguồn trang đầu ([reset] true) hoặc trang kế rồi cập nhật state.
  Future<void> _fetch({required bool reset}) async {
    final generation = reset ? ++_generation : _generation;
    final offset = reset ? 0 : _trips.length;

    try {
      final page = await widget.source.fetchPage(
        offset: offset,
        limit: widget.pageSize,
      );
      if (!mounted || generation != _generation) return;
      setState(() {
        if (reset) _trips.clear();
        _trips.addAll(page.trips);
        _hasMore = page.hasMore;
        _loading = false;
      });
    } on Object catch (error) {
      if (!mounted || generation != _generation) return;
      final keepList = reset && _trips.isNotEmpty;
      setState(() {
        _loading = false;
        if (!keepList) _error = error;
      });
      if (keepList) {
        ScaffoldMessenger.of(context)
            .showSnackBar(const SnackBar(content: Text('Không làm mới được')));
      }
      return;
    }
    // Trang vừa tải có thể chưa lấp đầy màn hình: không có cuộn thì cũng không
    // có sự kiện cuộn nào gọi _onScroll, nên ta tự hỏi lại sau khi build xong.
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) _onScroll();
    });
  }

  void _retry() => _load(reset: _trips.isEmpty);

  void _pin(Trip trip) {
    setState(() {
      _trips.removeWhere((t) => t.isSameTrip(trip));
      _trips.insert(0, trip);
    });
  }

  int? _indexOfKey(Key key) {
    if (key is! ValueKey<String>) return null;
    final index = _trips.indexWhere((trip) => trip.id == key.value);
    return index < 0 ? null : index;
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: RefreshIndicator(
        onRefresh: () => _load(reset: true),
        child: CustomScrollView(
          controller: _controller,
          // Danh sách ngắn hơn màn hình vẫn phải kéo được để làm mới.
          physics: const AlwaysScrollableScrollPhysics(),
          slivers: [
            const SliverAppBar(
              pinned: true,
              expandedHeight: 120,
              flexibleSpace: FlexibleSpaceBar(title: Text('Lịch sử chuyến')),
            ),
            if (_trips.isNotEmpty)
              SliverToBoxAdapter(child: _Summary(trips: _trips)),
            if (_trips.isEmpty)
              SliverFillRemaining(
                hasScrollBody: false,
                child: _EmptyState(
                  loading: _loading,
                  error: _error,
                  onRetry: _retry,
                ),
              )
            else
              SliverList.separated(
                itemCount: _trips.length,
                findItemIndexCallback: _indexOfKey,
                itemBuilder: (context, index) {
                  final trip = _trips[index];
                  return TripHistoryTile(
                    key: ValueKey(trip.id),
                    trip: trip,
                    onPin: index == 0 ? null : () => _pin(trip),
                  );
                },
                separatorBuilder: (context, index) => const Divider(height: 1),
              ),
            if (_trips.isNotEmpty)
              SliverToBoxAdapter(
                child: _Footer(
                  loading: _loading,
                  hasMore: _hasMore,
                  error: _error,
                  onRetry: _retry,
                ),
              ),
          ],
        ),
      ),
    );
  }
}

/// Tóm tắt các chuyến đã tải: số chuyến hoàn thành và tổng cước của chúng.
class _Summary extends StatelessWidget {
  const _Summary({required this.trips});

  final List<Trip> trips;

  @override
  Widget build(BuildContext context) {
    final completed = trips
        .where((trip) => trip.status == TripStatus.completed)
        .toList();
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Text(
        'Đã tải ${trips.length} chuyến · ${completed.length} hoàn thành, '
        'tổng cước ${completed.totalFare}',
      ),
    );
  }
}

/// Nội dung khi chưa có chuyến nào: đang tải, lỗi hoặc rỗng thật.
class _EmptyState extends StatelessWidget {
  const _EmptyState({
    required this.loading,
    required this.error,
    required this.onRetry,
  });

  final bool loading;
  final Object? error;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    if (loading) {
      return const Center(child: CircularProgressIndicator());
    }
    if (error != null) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Text('Không tải được lịch sử chuyến'),
            TextButton(onPressed: onRetry, child: const Text('Thử lại')),
          ],
        ),
      );
    }
    return const Center(child: Text('Chưa có chuyến nào'));
  }
}

/// Dòng cuối danh sách: vòng quay, nút thử lại hoặc lời báo đã hết.
class _Footer extends StatelessWidget {
  const _Footer({
    required this.loading,
    required this.hasMore,
    required this.error,
    required this.onRetry,
  });

  final bool loading;
  final bool hasMore;
  final Object? error;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    final Widget child;
    if (error != null) {
      child = TextButton(
        onPressed: onRetry,
        child: const Text('Không tải thêm được. Thử lại'),
      );
    } else if (loading) {
      child = const CircularProgressIndicator();
    } else if (!hasMore) {
      child = const Text('Đã hết lịch sử');
    } else {
      child = const SizedBox.shrink();
    }
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 24),
      child: Center(child: child),
    );
  }
}
