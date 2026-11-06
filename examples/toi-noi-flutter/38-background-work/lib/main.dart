import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:geolocator/geolocator.dart';
import 'package:path_provider/path_provider.dart';
import 'package:workmanager/workmanager.dart';

import 'background_dispatcher.dart';
import 'background_tasks.dart';
import 'buffer_store.dart';
import 'driver_app.dart';
import 'driver_location_controller.dart';
import 'driver_location_policy.dart';
import 'location_buffer.dart';
import 'location_tracker.dart';
import 'location_uploader.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await Workmanager().initialize(callbackDispatcher);
  final scheduler = WorkmanagerScheduler();
  await scheduler.schedulePurge(DriverBackgroundPlans.purgeExpired);

  final dir = await getApplicationSupportDirectory();
  final controller = DriverLocationController(
    policy: const DriverLocationPolicy(),
    tracker: GeolocatorLocationTracker(),
    uploader: LocationUploader(
      sink: const LoggingLocationSink(),
      buffer: LocationBuffer(),
      store: FileBufferStore(File('${dir.path}/pending_samples.json')),
      // Mất mạng: hẹn task nền gửi bù khi có mạng (ExistingWorkPolicy.keep
      // nên gọi nhiều lần vẫn chỉ có một task chờ).
      onFailed: (_) => unawaited(
        scheduler.scheduleFlushWhenOnline(DriverBackgroundPlans.flushPending),
      ),
    ),
  );

  runApp(
    DriverApp(
      controller: controller,
      onRequestPermission: () async {
        await Geolocator.requestPermission();
      },
    ),
  );
}
