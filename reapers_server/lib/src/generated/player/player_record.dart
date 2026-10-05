/* AUTOMATICALLY GENERATED CODE DO NOT MODIFY */
/*   To generate run: "serverpod generate"    */

// ignore_for_file: implementation_imports
// ignore_for_file: library_private_types_in_public_api
// ignore_for_file: non_constant_identifier_names
// ignore_for_file: public_member_api_docs
// ignore_for_file: type_literal_in_constant_pattern
// ignore_for_file: use_super_parameters
// ignore_for_file: invalid_use_of_internal_member

// ignore_for_file: no_leading_underscores_for_library_prefixes
import 'package:reapers_shared/reapers_shared.dart' as _izkb583i;
import 'package:serverpod/serverpod.dart' as _is;

/// One player's books: everything they own and have done.
///
/// The state is the shared `PlayerState`, stored whole. The server never
/// edits it field by field: every change is one of the shared rules applied
/// to the whole state, exactly as the offline game applies it.
abstract class PlayerRecord
    implements _is.TableRow<int?>, _is.ProtocolSerialization {
  PlayerRecord._({
    this.id,
    required this.authUserId,
    required this.state,
    this.location,
    DateTime? createdAt,
    DateTime? updatedAt,
  }) : createdAt = createdAt ?? DateTime.now(),
       updatedAt = updatedAt ?? DateTime.now();

  factory PlayerRecord({
    int? id,
    required _is.UuidValue authUserId,
    required _izkb583i.PlayerState state,
    _izkb583i.GeoFix? location,
    DateTime? createdAt,
    DateTime? updatedAt,
  }) = _PlayerRecordImpl;

  factory PlayerRecord.fromJson(Map<String, dynamic> jsonSerialization) {
    return PlayerRecord(
      id: jsonSerialization['id'] as int?,
      authUserId: _is.UuidValueJsonExtension.fromJson(
        jsonSerialization['authUserId'],
      ),
      state: _izkb583i.PlayerState.fromJson(jsonSerialization['state']),
      location: jsonSerialization['location'] == null
          ? null
          : _izkb583i.GeoFix.fromJson(jsonSerialization['location']),
      createdAt: jsonSerialization['createdAt'] == null
          ? null
          : _is.DateTimeJsonExtension.fromJson(jsonSerialization['createdAt']),
      updatedAt: jsonSerialization['updatedAt'] == null
          ? null
          : _is.DateTimeJsonExtension.fromJson(jsonSerialization['updatedAt']),
    );
  }

  static final t = PlayerRecordTable();

  static const db = PlayerRecordRepository._();

  @override
  int? id;

  /// The signed-in user these books belong to.
  _is.UuidValue authUserId;

  /// The player as they stand now.
  _izkb583i.PlayerState state;

  /// The latest foreground fix only; never a history. Null when tracking stops.
  _izkb583i.GeoFix? location;

  /// When the books were opened.
  DateTime createdAt;

  /// When they last changed.
  DateTime updatedAt;

  @override
  _is.Table<int?> get table => t;

  /// Returns a shallow copy of this [PlayerRecord]
  /// with some or all fields replaced by the given arguments.
  @_is.useResult
  PlayerRecord copyWith({
    int? id,
    _is.UuidValue? authUserId,
    _izkb583i.PlayerState? state,
    _izkb583i.GeoFix? location,
    DateTime? createdAt,
    DateTime? updatedAt,
  });
  @override
  Map<String, dynamic> toJson() {
    return {
      '__className__': 'PlayerRecord',
      if (id != null) 'id': id,
      'authUserId': authUserId.toJson(),
      'state': state.toJson(),
      if (location != null) 'location': location?.toJson(),
      'createdAt': createdAt.toJson(),
      'updatedAt': updatedAt.toJson(),
    };
  }

  @override
  Map<String, dynamic> toJsonForProtocol() {
    return {
      '__className__': 'PlayerRecord',
      if (id != null) 'id': id,
      'authUserId': authUserId.toJson(),
      'state':
          // ignore: unnecessary_type_check
          state is _is.ProtocolSerialization
          ? (state as _is.ProtocolSerialization).toJsonForProtocol()
          :
            // ignore: dead_code
            state.toJson(),
      if (location != null)
        'location':
            // ignore: unnecessary_type_check
            location is _is.ProtocolSerialization
            ? (location as _is.ProtocolSerialization).toJsonForProtocol()
            :
              // ignore: dead_code
              location?.toJson(),
      'createdAt': createdAt.toJson(),
      'updatedAt': updatedAt.toJson(),
    };
  }

  static PlayerRecordInclude include() {
    return PlayerRecordInclude._();
  }

  static PlayerRecordIncludeList includeList({
    _is.WhereExpressionBuilder<PlayerRecordTable>? where,
    int? limit,
    int? offset,
    _is.OrderByBuilder<PlayerRecordTable>? orderBy,
    _is.OrderByListBuilder<PlayerRecordTable>? orderByList,
    PlayerRecordInclude? include,
  }) {
    return PlayerRecordIncludeList._(
      where: where,
      limit: limit,
      offset: offset,
      orderBy: orderBy?.call(PlayerRecord.t),
      orderByList: orderByList?.call(PlayerRecord.t),
      include: include,
    );
  }

  @override
  String toString() {
    return _is.SerializationManager.encode(this);
  }
}

class _Undefined {}

class _PlayerRecordImpl extends PlayerRecord {
  _PlayerRecordImpl({
    int? id,
    required _is.UuidValue authUserId,
    required _izkb583i.PlayerState state,
    _izkb583i.GeoFix? location,
    DateTime? createdAt,
    DateTime? updatedAt,
  }) : super._(
         id: id,
         authUserId: authUserId,
         state: state,
         location: location,
         createdAt: createdAt,
         updatedAt: updatedAt,
       );

  /// Returns a shallow copy of this [PlayerRecord]
  /// with some or all fields replaced by the given arguments.
  @_is.useResult
  @override
  PlayerRecord copyWith({
    Object? id = _Undefined,
    _is.UuidValue? authUserId,
    _izkb583i.PlayerState? state,
    Object? location = _Undefined,
    DateTime? createdAt,
    DateTime? updatedAt,
  }) {
    return PlayerRecord(
      id: id is int? ? id : this.id,
      authUserId: authUserId ?? this.authUserId,
      state: state ?? this.state.copyWith(),
      location: location is _izkb583i.GeoFix?
          ? location
          : this.location?.copyWith(),
      createdAt: createdAt ?? this.createdAt,
      updatedAt: updatedAt ?? this.updatedAt,
    );
  }
}

class PlayerRecordUpdateTable extends _is.UpdateTable<PlayerRecordTable> {
  PlayerRecordUpdateTable(super.table);

  _is.ColumnValue<_is.UuidValue, _is.UuidValue> authUserId(
    _is.UuidValue value,
  ) => _is.ColumnValue(
    table.authUserId,
    value,
  );

  _is.ColumnValue<_izkb583i.PlayerState, _izkb583i.PlayerState> state(
    _izkb583i.PlayerState value,
  ) => _is.ColumnValue(
    table.state,
    value,
  );

  _is.ColumnValue<_izkb583i.GeoFix, _izkb583i.GeoFix> location(
    _izkb583i.GeoFix? value,
  ) => _is.ColumnValue(
    table.location,
    value,
  );

  _is.ColumnValue<DateTime, DateTime> createdAt(DateTime value) =>
      _is.ColumnValue(
        table.createdAt,
        value,
      );

  _is.ColumnValue<DateTime, DateTime> updatedAt(DateTime value) =>
      _is.ColumnValue(
        table.updatedAt,
        value,
      );
}

class PlayerRecordTable extends _is.Table<int?> {
  PlayerRecordTable({super.tableRelation}) : super(tableName: 'player_record') {
    updateTable = PlayerRecordUpdateTable(this);
    authUserId = _is.ColumnUuid(
      'authUserId',
      this,
    );
    state = _is.ColumnSerializable<_izkb583i.PlayerState>(
      'state',
      this,
    );
    location = _is.ColumnSerializable<_izkb583i.GeoFix>(
      'location',
      this,
    );
    createdAt = _is.ColumnDateTime(
      'createdAt',
      this,
      hasDefault: true,
    );
    updatedAt = _is.ColumnDateTime(
      'updatedAt',
      this,
      hasDefault: true,
    );
  }

  late final PlayerRecordUpdateTable updateTable;

  /// The signed-in user these books belong to.
  late final _is.ColumnUuid authUserId;

  /// The player as they stand now.
  late final _is.ColumnSerializable<_izkb583i.PlayerState> state;

  /// The latest foreground fix only; never a history. Null when tracking stops.
  late final _is.ColumnSerializable<_izkb583i.GeoFix> location;

  /// When the books were opened.
  late final _is.ColumnDateTime createdAt;

  /// When they last changed.
  late final _is.ColumnDateTime updatedAt;

  @override
  List<_is.Column> get columns => [
    id,
    authUserId,
    state,
    location,
    createdAt,
    updatedAt,
  ];
}

class PlayerRecordInclude extends _is.IncludeObject {
  PlayerRecordInclude._();

  @override
  Map<String, _is.Include?> get includes => {};

  @override
  _is.Table<int?> get table => PlayerRecord.t;
}

class PlayerRecordIncludeList extends _is.IncludeList {
  PlayerRecordIncludeList._({
    _is.WhereExpressionBuilder<PlayerRecordTable>? where,
    super.limit,
    super.offset,
    super.orderBy,
    super.orderByList,
    super.include,
  }) {
    super.where = where?.call(PlayerRecord.t);
  }

  @override
  Map<String, _is.Include?> get includes => include?.includes ?? {};

  @override
  _is.Table<int?> get table => PlayerRecord.t;
}

class PlayerRecordRepository {
  const PlayerRecordRepository._();

  /// Returns a list of [PlayerRecord]s matching the given query parameters.
  ///
  /// Use [where] to specify which items to include in the return value.
  /// If none is specified, all items will be returned.
  ///
  /// To specify the order of the items use [orderBy] or [orderByList]
  /// when sorting by multiple columns.
  ///
  /// The maximum number of items can be set by [limit]. If no limit is set,
  /// all items matching the query will be returned.
  ///
  /// [offset] defines how many items to skip, after which [limit] (or all)
  /// items are read from the database.
  ///
  /// ```dart
  /// var persons = await Persons.db.find(
  ///   session,
  ///   where: (t) => t.lastName.equals('Jones'),
  ///   orderBy: (t) => t.firstName,
  ///   limit: 100,
  /// );
  /// ```
  Future<List<PlayerRecord>> find(
    _is.DatabaseSession session, {
    _is.WhereExpressionBuilder<PlayerRecordTable>? where,
    int? limit,
    int? offset,
    _is.OrderByBuilder<PlayerRecordTable>? orderBy,
    _is.OrderByListBuilder<PlayerRecordTable>? orderByList,
    _is.Transaction? transaction,
    _is.LockMode? lockMode,
    _is.LockBehavior? lockBehavior,
  }) async {
    return session.db.find<PlayerRecord>(
      where: where?.call(PlayerRecord.t),
      orderBy: orderBy?.call(PlayerRecord.t),
      orderByList: orderByList?.call(PlayerRecord.t),
      limit: limit,
      offset: offset,
      transaction: transaction,
      lockMode: lockMode,
      lockBehavior: lockBehavior,
    );
  }

  /// Returns the first matching [PlayerRecord] matching the given query parameters.
  ///
  /// Use [where] to specify which items to include in the return value.
  /// If none is specified, all items will be returned.
  ///
  /// To specify the order use [orderBy] or [orderByList]
  /// when sorting by multiple columns.
  ///
  /// [offset] defines how many items to skip, after which the next one will be picked.
  ///
  /// ```dart
  /// var youngestPerson = await Persons.db.findFirstRow(
  ///   session,
  ///   where: (t) => t.lastName.equals('Jones'),
  ///   orderBy: (t) => t.age,
  /// );
  /// ```
  Future<PlayerRecord?> findFirstRow(
    _is.DatabaseSession session, {
    _is.WhereExpressionBuilder<PlayerRecordTable>? where,
    int? offset,
    _is.OrderByBuilder<PlayerRecordTable>? orderBy,
    _is.OrderByListBuilder<PlayerRecordTable>? orderByList,
    _is.Transaction? transaction,
    _is.LockMode? lockMode,
    _is.LockBehavior? lockBehavior,
  }) async {
    return session.db.findFirstRow<PlayerRecord>(
      where: where?.call(PlayerRecord.t),
      orderBy: orderBy?.call(PlayerRecord.t),
      orderByList: orderByList?.call(PlayerRecord.t),
      offset: offset,
      transaction: transaction,
      lockMode: lockMode,
      lockBehavior: lockBehavior,
    );
  }

  /// Finds a single [PlayerRecord] by its [id] or null if no such row exists.
  Future<PlayerRecord?> findById(
    _is.DatabaseSession session,
    int id, {
    _is.Transaction? transaction,
    _is.LockMode? lockMode,
    _is.LockBehavior? lockBehavior,
  }) async {
    return session.db.findById<PlayerRecord>(
      id,
      transaction: transaction,
      lockMode: lockMode,
      lockBehavior: lockBehavior,
    );
  }

  /// Inserts all [PlayerRecord]s in the list and returns the inserted rows.
  ///
  /// The returned [PlayerRecord]s will have their `id` fields set.
  ///
  /// This is an atomic operation, meaning that if one of the rows fails to
  /// insert, none of the rows will be inserted.
  ///
  /// If [ignoreConflicts] is set to `true`, rows that conflict with existing
  /// rows are silently skipped, and only the successfully inserted rows are
  /// returned.
  ///
  /// If [noReturn] is set to `true`, the inserted rows are not read back from
  /// the database and an empty list is returned. This avoids the overhead of
  /// transferring and deserializing the rows when the result is not needed.
  Future<List<PlayerRecord>> insert(
    _is.DatabaseSession session,
    List<PlayerRecord> rows, {
    _is.Transaction? transaction,
    bool ignoreConflicts = false,
    bool noReturn = false,
  }) async {
    return session.db.insert<PlayerRecord>(
      rows,
      transaction: transaction,
      ignoreConflicts: ignoreConflicts,
      noReturn: noReturn,
    );
  }

  /// Inserts a single [PlayerRecord] and returns the inserted row.
  ///
  /// The returned [PlayerRecord] will have its `id` field set.
  Future<PlayerRecord> insertRow(
    _is.DatabaseSession session,
    PlayerRecord row, {
    _is.Transaction? transaction,
  }) async {
    return session.db.insertRow<PlayerRecord>(
      row,
      transaction: transaction,
    );
  }

  /// Upserts all [PlayerRecord]s in the list and returns the resulting rows.
  ///
  /// If a row conflicts on the given [conflictColumns], the existing row is
  /// updated with the new values. Otherwise, a new row is inserted.
  ///
  /// If [updateColumns] is provided, only those columns will be updated on
  /// conflict. If null, all non-conflict, non-id columns are updated.
  ///
  /// If [updateWhere] is provided, the update only applies to rows matching the
  /// given expression. Conflicting rows that don't match are skipped and not
  /// returned, so the resulting list may be shorter than [rows].
  ///
  /// The returned [PlayerRecord]s will have their `id` fields set.
  ///
  /// This is an atomic operation, meaning that if one of the rows fails,
  /// none of the rows will be affected.
  ///
  /// If [noReturn] is set to `true`, the resulting rows are not read back from
  /// the database and an empty list is returned. This avoids the overhead of
  /// transferring and deserializing the rows when the result is not needed.
  Future<List<PlayerRecord>> upsert(
    _is.DatabaseSession session,
    List<PlayerRecord> rows, {
    required _is.ColumnSelections<PlayerRecordTable> conflictColumns,
    _is.ColumnSelections<PlayerRecordTable>? updateColumns,
    _is.WhereExpressionBuilder<PlayerRecordTable>? updateWhere,
    _is.Transaction? transaction,
    bool noReturn = false,
  }) async {
    return session.db.upsert<PlayerRecord>(
      rows,
      conflictColumns: conflictColumns(PlayerRecord.t),
      updateColumns: updateColumns?.call(PlayerRecord.t),
      updateWhere: updateWhere?.call(PlayerRecord.t),
      transaction: transaction,
      noReturn: noReturn,
    );
  }

  /// Upserts a single [PlayerRecord] and returns the resulting row.
  ///
  /// If the row conflicts on the given [conflictColumns], the existing row is
  /// updated. Otherwise, a new row is inserted.
  ///
  /// If [updateColumns] is provided, only those columns will be updated on
  /// conflict. If null, all non-conflict, non-id columns are updated.
  ///
  /// If [updateWhere] is provided, the update only applies when the existing
  /// row matches the expression. Returns `null` if no row was affected — for
  /// example when [updateWhere] does not match the conflicting row.
  ///
  /// The returned [PlayerRecord] will have its `id` field set.
  Future<PlayerRecord?> upsertRow(
    _is.DatabaseSession session,
    PlayerRecord row, {
    required _is.ColumnSelections<PlayerRecordTable> conflictColumns,
    _is.ColumnSelections<PlayerRecordTable>? updateColumns,
    _is.WhereExpressionBuilder<PlayerRecordTable>? updateWhere,
    _is.Transaction? transaction,
  }) async {
    return session.db.upsertRow<PlayerRecord>(
      row,
      conflictColumns: conflictColumns(PlayerRecord.t),
      updateColumns: updateColumns?.call(PlayerRecord.t),
      updateWhere: updateWhere?.call(PlayerRecord.t),
      transaction: transaction,
    );
  }

  /// Updates all [PlayerRecord]s in the list and returns the updated rows. If
  /// [columns] is provided, only those columns will be updated. Defaults to
  /// all columns.
  /// This is an atomic operation, meaning that if one of the rows fails to
  /// update, none of the rows will be updated.
  ///
  /// If [noReturn] is set to `true`, the updated rows are not read back from
  /// the database and an empty list is returned. This avoids the overhead of
  /// transferring and deserializing the rows when the result is not needed.
  Future<List<PlayerRecord>> update(
    _is.DatabaseSession session,
    List<PlayerRecord> rows, {
    _is.ColumnSelections<PlayerRecordTable>? columns,
    _is.Transaction? transaction,
    bool noReturn = false,
  }) async {
    return session.db.update<PlayerRecord>(
      rows,
      columns: columns?.call(PlayerRecord.t),
      transaction: transaction,
      noReturn: noReturn,
    );
  }

  /// Updates a single [PlayerRecord]. The row needs to have its id set.
  /// Optionally, a list of [columns] can be provided to only update those
  /// columns. Defaults to all columns.
  Future<PlayerRecord> updateRow(
    _is.DatabaseSession session,
    PlayerRecord row, {
    _is.ColumnSelections<PlayerRecordTable>? columns,
    _is.Transaction? transaction,
  }) async {
    return session.db.updateRow<PlayerRecord>(
      row,
      columns: columns?.call(PlayerRecord.t),
      transaction: transaction,
    );
  }

  /// Updates a single [PlayerRecord] by its [id] with the specified [columnValues].
  /// Returns the updated row or null if no row with the given id exists.
  Future<PlayerRecord?> updateById(
    _is.DatabaseSession session,
    int id, {
    required _is.ColumnValueListBuilder<PlayerRecordUpdateTable> columnValues,
    _is.Transaction? transaction,
  }) async {
    return session.db.updateById<PlayerRecord>(
      id,
      columnValues: columnValues(PlayerRecord.t.updateTable),
      transaction: transaction,
    );
  }

  /// Updates all [PlayerRecord]s matching the [where] expression with the specified [columnValues].
  /// Returns the list of updated rows.
  ///
  /// If [noReturn] is set to `true`, the updated rows are not read back from
  /// the database and an empty list is returned. This avoids the overhead of
  /// transferring and deserializing the rows when the result is not needed.
  Future<List<PlayerRecord>> updateWhere(
    _is.DatabaseSession session, {
    required _is.ColumnValueListBuilder<PlayerRecordUpdateTable> columnValues,
    required _is.WhereExpressionBuilder<PlayerRecordTable> where,
    int? limit,
    int? offset,
    _is.OrderByBuilder<PlayerRecordTable>? orderBy,
    _is.OrderByListBuilder<PlayerRecordTable>? orderByList,
    _is.Transaction? transaction,
    bool noReturn = false,
  }) async {
    return session.db.updateWhere<PlayerRecord>(
      columnValues: columnValues(PlayerRecord.t.updateTable),
      where: where(PlayerRecord.t),
      limit: limit,
      offset: offset,
      orderBy: orderBy?.call(PlayerRecord.t),
      orderByList: orderByList?.call(PlayerRecord.t),
      transaction: transaction,
      noReturn: noReturn,
    );
  }

  /// Deletes all [PlayerRecord]s in the list and returns the deleted rows.
  ///
  /// To specify the order of the returned rows use [orderBy] or [orderByList]
  /// when sorting by multiple columns.
  ///
  /// This is an atomic operation, meaning that if one of the rows fail to
  /// be deleted, none of the rows will be deleted.
  ///
  /// If [noReturn] is set to `true`, the deleted rows are not read back from
  /// the database and an empty list is returned. This avoids the overhead of
  /// transferring and deserializing the rows when the result is not needed.
  Future<List<PlayerRecord>> delete(
    _is.DatabaseSession session,
    List<PlayerRecord> rows, {
    _is.OrderByBuilder<PlayerRecordTable>? orderBy,
    _is.OrderByListBuilder<PlayerRecordTable>? orderByList,
    _is.Transaction? transaction,
    bool noReturn = false,
  }) async {
    return session.db.delete<PlayerRecord>(
      rows,
      orderBy: orderBy?.call(PlayerRecord.t),
      orderByList: orderByList?.call(PlayerRecord.t),
      transaction: transaction,
      noReturn: noReturn,
    );
  }

  /// Deletes a single [PlayerRecord].
  Future<PlayerRecord> deleteRow(
    _is.DatabaseSession session,
    PlayerRecord row, {
    _is.Transaction? transaction,
  }) async {
    return session.db.deleteRow<PlayerRecord>(
      row,
      transaction: transaction,
    );
  }

  /// Deletes all rows matching the [where] expression.
  ///
  /// To specify the order of the returned rows use [orderBy] or [orderByList]
  /// when sorting by multiple columns.
  ///
  /// If [noReturn] is set to `true`, the deleted rows are not read back from
  /// the database and an empty list is returned. This avoids the overhead of
  /// transferring and deserializing the rows when the result is not needed.
  Future<List<PlayerRecord>> deleteWhere(
    _is.DatabaseSession session, {
    required _is.WhereExpressionBuilder<PlayerRecordTable> where,
    _is.OrderByBuilder<PlayerRecordTable>? orderBy,
    _is.OrderByListBuilder<PlayerRecordTable>? orderByList,
    _is.Transaction? transaction,
    bool noReturn = false,
  }) async {
    return session.db.deleteWhere<PlayerRecord>(
      where: where(PlayerRecord.t),
      orderBy: orderBy?.call(PlayerRecord.t),
      orderByList: orderByList?.call(PlayerRecord.t),
      transaction: transaction,
      noReturn: noReturn,
    );
  }

  /// Counts the number of rows matching the [where] expression. If omitted,
  /// will return the count of all rows in the table.
  Future<int> count(
    _is.DatabaseSession session, {
    _is.WhereExpressionBuilder<PlayerRecordTable>? where,
    int? limit,
    _is.Transaction? transaction,
  }) async {
    return session.db.count<PlayerRecord>(
      where: where?.call(PlayerRecord.t),
      limit: limit,
      transaction: transaction,
    );
  }

  /// Acquires row-level locks on [PlayerRecord] rows matching the [where] expression.
  Future<void> lockRows(
    _is.DatabaseSession session, {
    required _is.WhereExpressionBuilder<PlayerRecordTable> where,
    required _is.LockMode lockMode,
    required _is.Transaction transaction,
    _is.LockBehavior lockBehavior = _is.LockBehavior.wait,
  }) async {
    return session.db.lockRows<PlayerRecord>(
      where: where(PlayerRecord.t),
      lockMode: lockMode,
      lockBehavior: lockBehavior,
      transaction: transaction,
    );
  }
}
