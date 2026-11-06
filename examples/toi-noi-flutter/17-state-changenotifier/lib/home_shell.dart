import 'package:flutter/material.dart';

import 'history_screen.dart';
import 'quote_screen.dart';

/// Khung hai tab. Tab đang mở là state tạm thời (ephemeral) của riêng khung
/// này nên vẫn dùng `setState`: không widget nào khác cần biết.
class HomeShell extends StatefulWidget {
  const HomeShell({super.key});

  @override
  State<HomeShell> createState() => _HomeShellState();
}

class _HomeShellState extends State<HomeShell> {
  int _index = 0;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text(_index == 0 ? 'Báo giá' : 'Lịch sử')),
      body: IndexedStack(
        index: _index,
        children: const [QuoteScreen(), HistoryScreen()],
      ),
      bottomNavigationBar: NavigationBar(
        selectedIndex: _index,
        onDestinationSelected: (i) => setState(() => _index = i),
        destinations: const [
          NavigationDestination(
            icon: Icon(Icons.request_quote_outlined),
            selectedIcon: Icon(Icons.request_quote),
            label: 'Báo giá',
          ),
          NavigationDestination(icon: Icon(Icons.history), label: 'Lịch sử'),
        ],
      ),
    );
  }
}
