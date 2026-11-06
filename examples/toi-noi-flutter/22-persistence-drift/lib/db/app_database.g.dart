// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'app_database.dart';

// ignore_for_file: type=lint
class $TripsTable extends Trips with TableInfo<$TripsTable, TripRow> {
  @override
  final GeneratedDatabase attachedDatabase;
  final String? _alias;
  $TripsTable(this.attachedDatabase, [this._alias]);
  static const VerificationMeta _idMeta = const VerificationMeta('id');
  @override
  late final GeneratedColumn<String> id = GeneratedColumn<String>(
    'id',
    aliasedName,
    false,
    type: DriftSqlType.string,
    requiredDuringInsert: true,
  );
  @override
  late final GeneratedColumnWithTypeConverter<TripStatus, String> status =
      GeneratedColumn<String>(
        'status',
        aliasedName,
        false,
        type: DriftSqlType.string,
        requiredDuringInsert: true,
      ).withConverter<TripStatus>($TripsTable.$converterstatus);
  static const VerificationMeta _distanceMetersMeta = const VerificationMeta(
    'distanceMeters',
  );
  @override
  late final GeneratedColumn<int> distanceMeters = GeneratedColumn<int>(
    'distance_meters',
    aliasedName,
    false,
    type: DriftSqlType.int,
    requiredDuringInsert: true,
  );
  static const VerificationMeta _fareVndMeta = const VerificationMeta(
    'fareVnd',
  );
  @override
  late final GeneratedColumn<int> fareVnd = GeneratedColumn<int>(
    'fare_vnd',
    aliasedName,
    false,
    check: () => ComparableExpr(fareVnd).isBiggerOrEqualValue(0),
    type: DriftSqlType.int,
    requiredDuringInsert: true,
  );
  static const VerificationMeta _requestedAtMsMeta = const VerificationMeta(
    'requestedAtMs',
  );
  @override
  late final GeneratedColumn<int> requestedAtMs = GeneratedColumn<int>(
    'requested_at_ms',
    aliasedName,
    false,
    type: DriftSqlType.int,
    requiredDuringInsert: true,
  );
  @override
  late final GeneratedColumnWithTypeConverter<ServiceKind, String> serviceKind =
      GeneratedColumn<String>(
        'service_kind',
        aliasedName,
        false,
        type: DriftSqlType.string,
        requiredDuringInsert: false,
        defaultValue: Constant(ServiceKind.ride.name),
      ).withConverter<ServiceKind>($TripsTable.$converterserviceKind);
  static const VerificationMeta _driverIdMeta = const VerificationMeta(
    'driverId',
  );
  @override
  late final GeneratedColumn<String> driverId = GeneratedColumn<String>(
    'driver_id',
    aliasedName,
    true,
    type: DriftSqlType.string,
    requiredDuringInsert: false,
  );
  @override
  List<GeneratedColumn> get $columns => [
    id,
    status,
    distanceMeters,
    fareVnd,
    requestedAtMs,
    serviceKind,
    driverId,
  ];
  @override
  String get aliasedName => _alias ?? actualTableName;
  @override
  String get actualTableName => $name;
  static const String $name = 'trips';
  @override
  VerificationContext validateIntegrity(
    Insertable<TripRow> instance, {
    bool isInserting = false,
  }) {
    final context = VerificationContext();
    final data = instance.toColumns(true);
    if (data.containsKey('id')) {
      context.handle(_idMeta, id.isAcceptableOrUnknown(data['id']!, _idMeta));
    } else if (isInserting) {
      context.missing(_idMeta);
    }
    if (data.containsKey('distance_meters')) {
      context.handle(
        _distanceMetersMeta,
        distanceMeters.isAcceptableOrUnknown(
          data['distance_meters']!,
          _distanceMetersMeta,
        ),
      );
    } else if (isInserting) {
      context.missing(_distanceMetersMeta);
    }
    if (data.containsKey('fare_vnd')) {
      context.handle(
        _fareVndMeta,
        fareVnd.isAcceptableOrUnknown(data['fare_vnd']!, _fareVndMeta),
      );
    } else if (isInserting) {
      context.missing(_fareVndMeta);
    }
    if (data.containsKey('requested_at_ms')) {
      context.handle(
        _requestedAtMsMeta,
        requestedAtMs.isAcceptableOrUnknown(
          data['requested_at_ms']!,
          _requestedAtMsMeta,
        ),
      );
    } else if (isInserting) {
      context.missing(_requestedAtMsMeta);
    }
    if (data.containsKey('driver_id')) {
      context.handle(
        _driverIdMeta,
        driverId.isAcceptableOrUnknown(data['driver_id']!, _driverIdMeta),
      );
    }
    return context;
  }

  @override
  Set<GeneratedColumn> get $primaryKey => {id};
  @override
  TripRow map(Map<String, dynamic> data, {String? tablePrefix}) {
    final effectivePrefix = tablePrefix != null ? '$tablePrefix.' : '';
    return TripRow(
      id: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}id'],
      )!,
      status: $TripsTable.$converterstatus.fromSql(
        attachedDatabase.typeMapping.read(
          DriftSqlType.string,
          data['${effectivePrefix}status'],
        )!,
      ),
      distanceMeters: attachedDatabase.typeMapping.read(
        DriftSqlType.int,
        data['${effectivePrefix}distance_meters'],
      )!,
      fareVnd: attachedDatabase.typeMapping.read(
        DriftSqlType.int,
        data['${effectivePrefix}fare_vnd'],
      )!,
      requestedAtMs: attachedDatabase.typeMapping.read(
        DriftSqlType.int,
        data['${effectivePrefix}requested_at_ms'],
      )!,
      serviceKind: $TripsTable.$converterserviceKind.fromSql(
        attachedDatabase.typeMapping.read(
          DriftSqlType.string,
          data['${effectivePrefix}service_kind'],
        )!,
      ),
      driverId: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}driver_id'],
      ),
    );
  }

  @override
  $TripsTable createAlias(String alias) {
    return $TripsTable(attachedDatabase, alias);
  }

  static JsonTypeConverter2<TripStatus, String, String> $converterstatus =
      const EnumNameConverter<TripStatus>(TripStatus.values);
  static JsonTypeConverter2<ServiceKind, String, String> $converterserviceKind =
      const EnumNameConverter<ServiceKind>(ServiceKind.values);
}

class TripRow extends DataClass implements Insertable<TripRow> {
  /// Mã chuyến do server cấp, dùng làm khoá chính.
  final String id;

  /// Tên của enum `TripStatus` (ví dụ `completed`), không lưu chỉ số.
  final TripStatus status;

  /// Quãng đường bằng mét nguyên.
  final int distanceMeters;

  /// Cước bằng đồng nguyên; database từ chối số âm.
  final int fareVnd;

  /// Thời điểm đặt, UTC, tính bằng mili giây từ epoch.
  final int requestedAtMs;

  /// Tên của enum `ServiceKind`. Thêm ở schema 2; dòng cũ nhận `ride`.
  final ServiceKind serviceKind;

  /// Tài xế nhận chuyến; null khi chưa có. Thêm ở schema 2.
  final String? driverId;
  const TripRow({
    required this.id,
    required this.status,
    required this.distanceMeters,
    required this.fareVnd,
    required this.requestedAtMs,
    required this.serviceKind,
    this.driverId,
  });
  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    map['id'] = Variable<String>(id);
    {
      map['status'] = Variable<String>(
        $TripsTable.$converterstatus.toSql(status),
      );
    }
    map['distance_meters'] = Variable<int>(distanceMeters);
    map['fare_vnd'] = Variable<int>(fareVnd);
    map['requested_at_ms'] = Variable<int>(requestedAtMs);
    {
      map['service_kind'] = Variable<String>(
        $TripsTable.$converterserviceKind.toSql(serviceKind),
      );
    }
    if (!nullToAbsent || driverId != null) {
      map['driver_id'] = Variable<String>(driverId);
    }
    return map;
  }

  TripsCompanion toCompanion(bool nullToAbsent) {
    return TripsCompanion(
      id: Value(id),
      status: Value(status),
      distanceMeters: Value(distanceMeters),
      fareVnd: Value(fareVnd),
      requestedAtMs: Value(requestedAtMs),
      serviceKind: Value(serviceKind),
      driverId: driverId == null && nullToAbsent
          ? const Value.absent()
          : Value(driverId),
    );
  }

  factory TripRow.fromJson(
    Map<String, dynamic> json, {
    ValueSerializer? serializer,
  }) {
    serializer ??= driftRuntimeOptions.defaultSerializer;
    return TripRow(
      id: serializer.fromJson<String>(json['id']),
      status: $TripsTable.$converterstatus.fromJson(
        serializer.fromJson<String>(json['status']),
      ),
      distanceMeters: serializer.fromJson<int>(json['distanceMeters']),
      fareVnd: serializer.fromJson<int>(json['fareVnd']),
      requestedAtMs: serializer.fromJson<int>(json['requestedAtMs']),
      serviceKind: $TripsTable.$converterserviceKind.fromJson(
        serializer.fromJson<String>(json['serviceKind']),
      ),
      driverId: serializer.fromJson<String?>(json['driverId']),
    );
  }
  @override
  Map<String, dynamic> toJson({ValueSerializer? serializer}) {
    serializer ??= driftRuntimeOptions.defaultSerializer;
    return <String, dynamic>{
      'id': serializer.toJson<String>(id),
      'status': serializer.toJson<String>(
        $TripsTable.$converterstatus.toJson(status),
      ),
      'distanceMeters': serializer.toJson<int>(distanceMeters),
      'fareVnd': serializer.toJson<int>(fareVnd),
      'requestedAtMs': serializer.toJson<int>(requestedAtMs),
      'serviceKind': serializer.toJson<String>(
        $TripsTable.$converterserviceKind.toJson(serviceKind),
      ),
      'driverId': serializer.toJson<String?>(driverId),
    };
  }

  TripRow copyWith({
    String? id,
    TripStatus? status,
    int? distanceMeters,
    int? fareVnd,
    int? requestedAtMs,
    ServiceKind? serviceKind,
    Value<String?> driverId = const Value.absent(),
  }) => TripRow(
    id: id ?? this.id,
    status: status ?? this.status,
    distanceMeters: distanceMeters ?? this.distanceMeters,
    fareVnd: fareVnd ?? this.fareVnd,
    requestedAtMs: requestedAtMs ?? this.requestedAtMs,
    serviceKind: serviceKind ?? this.serviceKind,
    driverId: driverId.present ? driverId.value : this.driverId,
  );
  TripRow copyWithCompanion(TripsCompanion data) {
    return TripRow(
      id: data.id.present ? data.id.value : this.id,
      status: data.status.present ? data.status.value : this.status,
      distanceMeters: data.distanceMeters.present
          ? data.distanceMeters.value
          : this.distanceMeters,
      fareVnd: data.fareVnd.present ? data.fareVnd.value : this.fareVnd,
      requestedAtMs: data.requestedAtMs.present
          ? data.requestedAtMs.value
          : this.requestedAtMs,
      serviceKind: data.serviceKind.present
          ? data.serviceKind.value
          : this.serviceKind,
      driverId: data.driverId.present ? data.driverId.value : this.driverId,
    );
  }

  @override
  String toString() {
    return (StringBuffer('TripRow(')
          ..write('id: $id, ')
          ..write('status: $status, ')
          ..write('distanceMeters: $distanceMeters, ')
          ..write('fareVnd: $fareVnd, ')
          ..write('requestedAtMs: $requestedAtMs, ')
          ..write('serviceKind: $serviceKind, ')
          ..write('driverId: $driverId')
          ..write(')'))
        .toString();
  }

  @override
  int get hashCode => Object.hash(
    id,
    status,
    distanceMeters,
    fareVnd,
    requestedAtMs,
    serviceKind,
    driverId,
  );
  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is TripRow &&
          other.id == this.id &&
          other.status == this.status &&
          other.distanceMeters == this.distanceMeters &&
          other.fareVnd == this.fareVnd &&
          other.requestedAtMs == this.requestedAtMs &&
          other.serviceKind == this.serviceKind &&
          other.driverId == this.driverId);
}

class TripsCompanion extends UpdateCompanion<TripRow> {
  final Value<String> id;
  final Value<TripStatus> status;
  final Value<int> distanceMeters;
  final Value<int> fareVnd;
  final Value<int> requestedAtMs;
  final Value<ServiceKind> serviceKind;
  final Value<String?> driverId;
  final Value<int> rowid;
  const TripsCompanion({
    this.id = const Value.absent(),
    this.status = const Value.absent(),
    this.distanceMeters = const Value.absent(),
    this.fareVnd = const Value.absent(),
    this.requestedAtMs = const Value.absent(),
    this.serviceKind = const Value.absent(),
    this.driverId = const Value.absent(),
    this.rowid = const Value.absent(),
  });
  TripsCompanion.insert({
    required String id,
    required TripStatus status,
    required int distanceMeters,
    required int fareVnd,
    required int requestedAtMs,
    this.serviceKind = const Value.absent(),
    this.driverId = const Value.absent(),
    this.rowid = const Value.absent(),
  }) : id = Value(id),
       status = Value(status),
       distanceMeters = Value(distanceMeters),
       fareVnd = Value(fareVnd),
       requestedAtMs = Value(requestedAtMs);
  static Insertable<TripRow> custom({
    Expression<String>? id,
    Expression<String>? status,
    Expression<int>? distanceMeters,
    Expression<int>? fareVnd,
    Expression<int>? requestedAtMs,
    Expression<String>? serviceKind,
    Expression<String>? driverId,
    Expression<int>? rowid,
  }) {
    return RawValuesInsertable({
      if (id != null) 'id': id,
      if (status != null) 'status': status,
      if (distanceMeters != null) 'distance_meters': distanceMeters,
      if (fareVnd != null) 'fare_vnd': fareVnd,
      if (requestedAtMs != null) 'requested_at_ms': requestedAtMs,
      if (serviceKind != null) 'service_kind': serviceKind,
      if (driverId != null) 'driver_id': driverId,
      if (rowid != null) 'rowid': rowid,
    });
  }

  TripsCompanion copyWith({
    Value<String>? id,
    Value<TripStatus>? status,
    Value<int>? distanceMeters,
    Value<int>? fareVnd,
    Value<int>? requestedAtMs,
    Value<ServiceKind>? serviceKind,
    Value<String?>? driverId,
    Value<int>? rowid,
  }) {
    return TripsCompanion(
      id: id ?? this.id,
      status: status ?? this.status,
      distanceMeters: distanceMeters ?? this.distanceMeters,
      fareVnd: fareVnd ?? this.fareVnd,
      requestedAtMs: requestedAtMs ?? this.requestedAtMs,
      serviceKind: serviceKind ?? this.serviceKind,
      driverId: driverId ?? this.driverId,
      rowid: rowid ?? this.rowid,
    );
  }

  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    if (id.present) {
      map['id'] = Variable<String>(id.value);
    }
    if (status.present) {
      map['status'] = Variable<String>(
        $TripsTable.$converterstatus.toSql(status.value),
      );
    }
    if (distanceMeters.present) {
      map['distance_meters'] = Variable<int>(distanceMeters.value);
    }
    if (fareVnd.present) {
      map['fare_vnd'] = Variable<int>(fareVnd.value);
    }
    if (requestedAtMs.present) {
      map['requested_at_ms'] = Variable<int>(requestedAtMs.value);
    }
    if (serviceKind.present) {
      map['service_kind'] = Variable<String>(
        $TripsTable.$converterserviceKind.toSql(serviceKind.value),
      );
    }
    if (driverId.present) {
      map['driver_id'] = Variable<String>(driverId.value);
    }
    if (rowid.present) {
      map['rowid'] = Variable<int>(rowid.value);
    }
    return map;
  }

  @override
  String toString() {
    return (StringBuffer('TripsCompanion(')
          ..write('id: $id, ')
          ..write('status: $status, ')
          ..write('distanceMeters: $distanceMeters, ')
          ..write('fareVnd: $fareVnd, ')
          ..write('requestedAtMs: $requestedAtMs, ')
          ..write('serviceKind: $serviceKind, ')
          ..write('driverId: $driverId, ')
          ..write('rowid: $rowid')
          ..write(')'))
        .toString();
  }
}

abstract class _$AppDatabase extends GeneratedDatabase {
  _$AppDatabase(QueryExecutor e) : super(e);
  $AppDatabaseManager get managers => $AppDatabaseManager(this);
  late final $TripsTable trips = $TripsTable(this);
  late final TripDao tripDao = TripDao(this as AppDatabase);
  @override
  Iterable<TableInfo<Table, Object?>> get allTables =>
      allSchemaEntities.whereType<TableInfo<Table, Object?>>();
  @override
  List<DatabaseSchemaEntity> get allSchemaEntities => [trips];
}

typedef $$TripsTableCreateCompanionBuilder = TripsCompanion Function({
  required String id,
  required TripStatus status,
  required int distanceMeters,
  required int fareVnd,
  required int requestedAtMs,
  Value<ServiceKind> serviceKind,
  Value<String?> driverId,
  Value<int> rowid,
});
typedef $$TripsTableUpdateCompanionBuilder = TripsCompanion Function({
  Value<String> id,
  Value<TripStatus> status,
  Value<int> distanceMeters,
  Value<int> fareVnd,
  Value<int> requestedAtMs,
  Value<ServiceKind> serviceKind,
  Value<String?> driverId,
  Value<int> rowid,
});

class $$TripsTableFilterComposer extends Composer<_$AppDatabase, $TripsTable> {
  $$TripsTableFilterComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  ColumnFilters<String> get id => $composableBuilder(
    column: $table.id,
    builder: (column) => ColumnFilters(column),
  );

  ColumnWithTypeConverterFilters<TripStatus, TripStatus, String> get status =>
      $composableBuilder(
        column: $table.status,
        builder: (column) => ColumnWithTypeConverterFilters(column),
      );

  ColumnFilters<int> get distanceMeters => $composableBuilder(
    column: $table.distanceMeters,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<int> get fareVnd => $composableBuilder(
    column: $table.fareVnd,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<int> get requestedAtMs => $composableBuilder(
    column: $table.requestedAtMs,
    builder: (column) => ColumnFilters(column),
  );

  ColumnWithTypeConverterFilters<ServiceKind, ServiceKind, String>
  get serviceKind => $composableBuilder(
    column: $table.serviceKind,
    builder: (column) => ColumnWithTypeConverterFilters(column),
  );

  ColumnFilters<String> get driverId => $composableBuilder(
    column: $table.driverId,
    builder: (column) => ColumnFilters(column),
  );
}

class $$TripsTableOrderingComposer
    extends Composer<_$AppDatabase, $TripsTable> {
  $$TripsTableOrderingComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  ColumnOrderings<String> get id => $composableBuilder(
    column: $table.id,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get status => $composableBuilder(
    column: $table.status,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<int> get distanceMeters => $composableBuilder(
    column: $table.distanceMeters,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<int> get fareVnd => $composableBuilder(
    column: $table.fareVnd,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<int> get requestedAtMs => $composableBuilder(
    column: $table.requestedAtMs,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get serviceKind => $composableBuilder(
    column: $table.serviceKind,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get driverId => $composableBuilder(
    column: $table.driverId,
    builder: (column) => ColumnOrderings(column),
  );
}

class $$TripsTableAnnotationComposer
    extends Composer<_$AppDatabase, $TripsTable> {
  $$TripsTableAnnotationComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  GeneratedColumn<String> get id =>
      $composableBuilder(column: $table.id, builder: (column) => column);

  GeneratedColumnWithTypeConverter<TripStatus, String> get status =>
      $composableBuilder(column: $table.status, builder: (column) => column);

  GeneratedColumn<int> get distanceMeters => $composableBuilder(
    column: $table.distanceMeters,
    builder: (column) => column,
  );

  GeneratedColumn<int> get fareVnd =>
      $composableBuilder(column: $table.fareVnd, builder: (column) => column);

  GeneratedColumn<int> get requestedAtMs => $composableBuilder(
    column: $table.requestedAtMs,
    builder: (column) => column,
  );

  GeneratedColumnWithTypeConverter<ServiceKind, String> get serviceKind =>
      $composableBuilder(
        column: $table.serviceKind,
        builder: (column) => column,
      );

  GeneratedColumn<String> get driverId =>
      $composableBuilder(column: $table.driverId, builder: (column) => column);
}

class $$TripsTableTableManager
    extends
        RootTableManager<
          _$AppDatabase,
          $TripsTable,
          TripRow,
          $$TripsTableFilterComposer,
          $$TripsTableOrderingComposer,
          $$TripsTableAnnotationComposer,
          $$TripsTableCreateCompanionBuilder,
          $$TripsTableUpdateCompanionBuilder,
          (TripRow, BaseReferences<_$AppDatabase, $TripsTable, TripRow>),
          TripRow,
          PrefetchHooks Function()
        > {
  $$TripsTableTableManager(_$AppDatabase db, $TripsTable table)
    : super(
        TableManagerState(
          db: db,
          table: table,
          createFilteringComposer: () =>
              $$TripsTableFilterComposer($db: db, $table: table),
          createOrderingComposer: () =>
              $$TripsTableOrderingComposer($db: db, $table: table),
          createComputedFieldComposer: () =>
              $$TripsTableAnnotationComposer($db: db, $table: table),
          updateCompanionCallback:
              ({
                Value<String> id = const Value.absent(),
                Value<TripStatus> status = const Value.absent(),
                Value<int> distanceMeters = const Value.absent(),
                Value<int> fareVnd = const Value.absent(),
                Value<int> requestedAtMs = const Value.absent(),
                Value<ServiceKind> serviceKind = const Value.absent(),
                Value<String?> driverId = const Value.absent(),
                Value<int> rowid = const Value.absent(),
              }) => TripsCompanion(
                id: id,
                status: status,
                distanceMeters: distanceMeters,
                fareVnd: fareVnd,
                requestedAtMs: requestedAtMs,
                serviceKind: serviceKind,
                driverId: driverId,
                rowid: rowid,
              ),
          createCompanionCallback:
              ({
                required String id,
                required TripStatus status,
                required int distanceMeters,
                required int fareVnd,
                required int requestedAtMs,
                Value<ServiceKind> serviceKind = const Value.absent(),
                Value<String?> driverId = const Value.absent(),
                Value<int> rowid = const Value.absent(),
              }) => TripsCompanion.insert(
                id: id,
                status: status,
                distanceMeters: distanceMeters,
                fareVnd: fareVnd,
                requestedAtMs: requestedAtMs,
                serviceKind: serviceKind,
                driverId: driverId,
                rowid: rowid,
              ),
          withReferenceMapper: (p0) => p0
              .map(
                (e) => (
                  e.readTable<$TripsTable, TripRow>(table),
                  BaseReferences<_$AppDatabase, $TripsTable, TripRow>(
                    db,
                    table,
                    e,
                  ),
                ),
              )
              .toList(),
          prefetchHooksCallback: null,
        ),
      );
}

typedef $$TripsTableProcessedTableManager =
    ProcessedTableManager<
      _$AppDatabase,
      $TripsTable,
      TripRow,
      $$TripsTableFilterComposer,
      $$TripsTableOrderingComposer,
      $$TripsTableAnnotationComposer,
      $$TripsTableCreateCompanionBuilder,
      $$TripsTableUpdateCompanionBuilder,
      (TripRow, BaseReferences<_$AppDatabase, $TripsTable, TripRow>),
      TripRow,
      PrefetchHooks Function()
    >;

class $AppDatabaseManager {
  final _$AppDatabase _db;
  $AppDatabaseManager(this._db);
  $$TripsTableTableManager get trips =>
      $$TripsTableTableManager(_db, _db.trips);
}
