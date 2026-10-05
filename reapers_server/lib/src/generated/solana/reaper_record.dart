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

/// One canonical growth record per NFT, shared across every owner.
abstract class ReaperRecord
    implements _is.TableRow<int?>, _is.ProtocolSerialization {
  ReaperRecord._({
    this.id,
    required this.assetKey,
    required this.collection,
    required this.state,
  });

  factory ReaperRecord({
    int? id,
    required String assetKey,
    required String collection,
    required _izkb583i.ReaperState state,
  }) = _ReaperRecordImpl;

  factory ReaperRecord.fromJson(Map<String, dynamic> jsonSerialization) {
    return ReaperRecord(
      id: jsonSerialization['id'] as int?,
      assetKey: jsonSerialization['assetKey'] as String,
      collection: jsonSerialization['collection'] as String,
      state: _izkb583i.ReaperState.fromJson(jsonSerialization['state']),
    );
  }

  static final t = ReaperRecordTable();

  static const db = ReaperRecordRepository._();

  @override
  int? id;

  String assetKey;

  String collection;

  _izkb583i.ReaperState state;

  @override
  _is.Table<int?> get table => t;

  /// Returns a shallow copy of this [ReaperRecord]
  /// with some or all fields replaced by the given arguments.
  @_is.useResult
  ReaperRecord copyWith({
    int? id,
    String? assetKey,
    String? collection,
    _izkb583i.ReaperState? state,
  });
  @override
  Map<String, dynamic> toJson() {
    return {
      '__className__': 'ReaperRecord',
      if (id != null) 'id': id,
      'assetKey': assetKey,
      'collection': collection,
      'state': state.toJson(),
    };
  }

  @override
  Map<String, dynamic> toJsonForProtocol() {
    return {
      '__className__': 'ReaperRecord',
      if (id != null) 'id': id,
      'assetKey': assetKey,
      'collection': collection,
      'state':
          // ignore: unnecessary_type_check
          state is _is.ProtocolSerialization
          ? (state as _is.ProtocolSerialization).toJsonForProtocol()
          :
            // ignore: dead_code
            state.toJson(),
    };
  }

  static ReaperRecordInclude include() {
    return ReaperRecordInclude._();
  }

  static ReaperRecordIncludeList includeList({
    _is.WhereExpressionBuilder<ReaperRecordTable>? where,
    int? limit,
    int? offset,
    _is.OrderByBuilder<ReaperRecordTable>? orderBy,
    _is.OrderByListBuilder<ReaperRecordTable>? orderByList,
    ReaperRecordInclude? include,
  }) {
    return ReaperRecordIncludeList._(
      where: where,
      limit: limit,
      offset: offset,
      orderBy: orderBy?.call(ReaperRecord.t),
      orderByList: orderByList?.call(ReaperRecord.t),
      include: include,
    );
  }

  @override
  String toString() {
    return _is.SerializationManager.encode(this);
  }
}

class _Undefined {}

class _ReaperRecordImpl extends ReaperRecord {
  _ReaperRecordImpl({
    int? id,
    required String assetKey,
    required String collection,
    required _izkb583i.ReaperState state,
  }) : super._(
         id: id,
         assetKey: assetKey,
         collection: collection,
         state: state,
       );

  /// Returns a shallow copy of this [ReaperRecord]
  /// with some or all fields replaced by the given arguments.
  @_is.useResult
  @override
  ReaperRecord copyWith({
    Object? id = _Undefined,
    String? assetKey,
    String? collection,
    _izkb583i.ReaperState? state,
  }) {
    return ReaperRecord(
      id: id is int? ? id : this.id,
      assetKey: assetKey ?? this.assetKey,
      collection: collection ?? this.collection,
      state: state ?? this.state.copyWith(),
    );
  }
}

class ReaperRecordUpdateTable extends _is.UpdateTable<ReaperRecordTable> {
  ReaperRecordUpdateTable(super.table);

  _is.ColumnValue<String, String> assetKey(String value) => _is.ColumnValue(
    table.assetKey,
    value,
  );

  _is.ColumnValue<String, String> collection(String value) => _is.ColumnValue(
    table.collection,
    value,
  );

  _is.ColumnValue<_izkb583i.ReaperState, _izkb583i.ReaperState> state(
    _izkb583i.ReaperState value,
  ) => _is.ColumnValue(
    table.state,
    value,
  );
}

class ReaperRecordTable extends _is.Table<int?> {
  ReaperRecordTable({super.tableRelation}) : super(tableName: 'reaper_record') {
    updateTable = ReaperRecordUpdateTable(this);
    assetKey = _is.ColumnString(
      'assetKey',
      this,
    );
    collection = _is.ColumnString(
      'collection',
      this,
    );
    state = _is.ColumnSerializable<_izkb583i.ReaperState>(
      'state',
      this,
    );
  }

  late final ReaperRecordUpdateTable updateTable;

  late final _is.ColumnString assetKey;

  late final _is.ColumnString collection;

  late final _is.ColumnSerializable<_izkb583i.ReaperState> state;

  @override
  List<_is.Column> get columns => [
    id,
    assetKey,
    collection,
    state,
  ];
}

class ReaperRecordInclude extends _is.IncludeObject {
  ReaperRecordInclude._();

  @override
  Map<String, _is.Include?> get includes => {};

  @override
  _is.Table<int?> get table => ReaperRecord.t;
}

class ReaperRecordIncludeList extends _is.IncludeList {
  ReaperRecordIncludeList._({
    _is.WhereExpressionBuilder<ReaperRecordTable>? where,
    super.limit,
    super.offset,
    super.orderBy,
    super.orderByList,
    super.include,
  }) {
    super.where = where?.call(ReaperRecord.t);
  }

  @override
  Map<String, _is.Include?> get includes => include?.includes ?? {};

  @override
  _is.Table<int?> get table => ReaperRecord.t;
}

class ReaperRecordRepository {
  const ReaperRecordRepository._();

  /// Returns a list of [ReaperRecord]s matching the given query parameters.
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
  Future<List<ReaperRecord>> find(
    _is.DatabaseSession session, {
    _is.WhereExpressionBuilder<ReaperRecordTable>? where,
    int? limit,
    int? offset,
    _is.OrderByBuilder<ReaperRecordTable>? orderBy,
    _is.OrderByListBuilder<ReaperRecordTable>? orderByList,
    _is.Transaction? transaction,
    _is.LockMode? lockMode,
    _is.LockBehavior? lockBehavior,
  }) async {
    return session.db.find<ReaperRecord>(
      where: where?.call(ReaperRecord.t),
      orderBy: orderBy?.call(ReaperRecord.t),
      orderByList: orderByList?.call(ReaperRecord.t),
      limit: limit,
      offset: offset,
      transaction: transaction,
      lockMode: lockMode,
      lockBehavior: lockBehavior,
    );
  }

  /// Returns the first matching [ReaperRecord] matching the given query parameters.
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
  Future<ReaperRecord?> findFirstRow(
    _is.DatabaseSession session, {
    _is.WhereExpressionBuilder<ReaperRecordTable>? where,
    int? offset,
    _is.OrderByBuilder<ReaperRecordTable>? orderBy,
    _is.OrderByListBuilder<ReaperRecordTable>? orderByList,
    _is.Transaction? transaction,
    _is.LockMode? lockMode,
    _is.LockBehavior? lockBehavior,
  }) async {
    return session.db.findFirstRow<ReaperRecord>(
      where: where?.call(ReaperRecord.t),
      orderBy: orderBy?.call(ReaperRecord.t),
      orderByList: orderByList?.call(ReaperRecord.t),
      offset: offset,
      transaction: transaction,
      lockMode: lockMode,
      lockBehavior: lockBehavior,
    );
  }

  /// Finds a single [ReaperRecord] by its [id] or null if no such row exists.
  Future<ReaperRecord?> findById(
    _is.DatabaseSession session,
    int id, {
    _is.Transaction? transaction,
    _is.LockMode? lockMode,
    _is.LockBehavior? lockBehavior,
  }) async {
    return session.db.findById<ReaperRecord>(
      id,
      transaction: transaction,
      lockMode: lockMode,
      lockBehavior: lockBehavior,
    );
  }

  /// Inserts all [ReaperRecord]s in the list and returns the inserted rows.
  ///
  /// The returned [ReaperRecord]s will have their `id` fields set.
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
  Future<List<ReaperRecord>> insert(
    _is.DatabaseSession session,
    List<ReaperRecord> rows, {
    _is.Transaction? transaction,
    bool ignoreConflicts = false,
    bool noReturn = false,
  }) async {
    return session.db.insert<ReaperRecord>(
      rows,
      transaction: transaction,
      ignoreConflicts: ignoreConflicts,
      noReturn: noReturn,
    );
  }

  /// Inserts a single [ReaperRecord] and returns the inserted row.
  ///
  /// The returned [ReaperRecord] will have its `id` field set.
  Future<ReaperRecord> insertRow(
    _is.DatabaseSession session,
    ReaperRecord row, {
    _is.Transaction? transaction,
  }) async {
    return session.db.insertRow<ReaperRecord>(
      row,
      transaction: transaction,
    );
  }

  /// Upserts all [ReaperRecord]s in the list and returns the resulting rows.
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
  /// The returned [ReaperRecord]s will have their `id` fields set.
  ///
  /// This is an atomic operation, meaning that if one of the rows fails,
  /// none of the rows will be affected.
  ///
  /// If [noReturn] is set to `true`, the resulting rows are not read back from
  /// the database and an empty list is returned. This avoids the overhead of
  /// transferring and deserializing the rows when the result is not needed.
  Future<List<ReaperRecord>> upsert(
    _is.DatabaseSession session,
    List<ReaperRecord> rows, {
    required _is.ColumnSelections<ReaperRecordTable> conflictColumns,
    _is.ColumnSelections<ReaperRecordTable>? updateColumns,
    _is.WhereExpressionBuilder<ReaperRecordTable>? updateWhere,
    _is.Transaction? transaction,
    bool noReturn = false,
  }) async {
    return session.db.upsert<ReaperRecord>(
      rows,
      conflictColumns: conflictColumns(ReaperRecord.t),
      updateColumns: updateColumns?.call(ReaperRecord.t),
      updateWhere: updateWhere?.call(ReaperRecord.t),
      transaction: transaction,
      noReturn: noReturn,
    );
  }

  /// Upserts a single [ReaperRecord] and returns the resulting row.
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
  /// The returned [ReaperRecord] will have its `id` field set.
  Future<ReaperRecord?> upsertRow(
    _is.DatabaseSession session,
    ReaperRecord row, {
    required _is.ColumnSelections<ReaperRecordTable> conflictColumns,
    _is.ColumnSelections<ReaperRecordTable>? updateColumns,
    _is.WhereExpressionBuilder<ReaperRecordTable>? updateWhere,
    _is.Transaction? transaction,
  }) async {
    return session.db.upsertRow<ReaperRecord>(
      row,
      conflictColumns: conflictColumns(ReaperRecord.t),
      updateColumns: updateColumns?.call(ReaperRecord.t),
      updateWhere: updateWhere?.call(ReaperRecord.t),
      transaction: transaction,
    );
  }

  /// Updates all [ReaperRecord]s in the list and returns the updated rows. If
  /// [columns] is provided, only those columns will be updated. Defaults to
  /// all columns.
  /// This is an atomic operation, meaning that if one of the rows fails to
  /// update, none of the rows will be updated.
  ///
  /// If [noReturn] is set to `true`, the updated rows are not read back from
  /// the database and an empty list is returned. This avoids the overhead of
  /// transferring and deserializing the rows when the result is not needed.
  Future<List<ReaperRecord>> update(
    _is.DatabaseSession session,
    List<ReaperRecord> rows, {
    _is.ColumnSelections<ReaperRecordTable>? columns,
    _is.Transaction? transaction,
    bool noReturn = false,
  }) async {
    return session.db.update<ReaperRecord>(
      rows,
      columns: columns?.call(ReaperRecord.t),
      transaction: transaction,
      noReturn: noReturn,
    );
  }

  /// Updates a single [ReaperRecord]. The row needs to have its id set.
  /// Optionally, a list of [columns] can be provided to only update those
  /// columns. Defaults to all columns.
  Future<ReaperRecord> updateRow(
    _is.DatabaseSession session,
    ReaperRecord row, {
    _is.ColumnSelections<ReaperRecordTable>? columns,
    _is.Transaction? transaction,
  }) async {
    return session.db.updateRow<ReaperRecord>(
      row,
      columns: columns?.call(ReaperRecord.t),
      transaction: transaction,
    );
  }

  /// Updates a single [ReaperRecord] by its [id] with the specified [columnValues].
  /// Returns the updated row or null if no row with the given id exists.
  Future<ReaperRecord?> updateById(
    _is.DatabaseSession session,
    int id, {
    required _is.ColumnValueListBuilder<ReaperRecordUpdateTable> columnValues,
    _is.Transaction? transaction,
  }) async {
    return session.db.updateById<ReaperRecord>(
      id,
      columnValues: columnValues(ReaperRecord.t.updateTable),
      transaction: transaction,
    );
  }

  /// Updates all [ReaperRecord]s matching the [where] expression with the specified [columnValues].
  /// Returns the list of updated rows.
  ///
  /// If [noReturn] is set to `true`, the updated rows are not read back from
  /// the database and an empty list is returned. This avoids the overhead of
  /// transferring and deserializing the rows when the result is not needed.
  Future<List<ReaperRecord>> updateWhere(
    _is.DatabaseSession session, {
    required _is.ColumnValueListBuilder<ReaperRecordUpdateTable> columnValues,
    required _is.WhereExpressionBuilder<ReaperRecordTable> where,
    int? limit,
    int? offset,
    _is.OrderByBuilder<ReaperRecordTable>? orderBy,
    _is.OrderByListBuilder<ReaperRecordTable>? orderByList,
    _is.Transaction? transaction,
    bool noReturn = false,
  }) async {
    return session.db.updateWhere<ReaperRecord>(
      columnValues: columnValues(ReaperRecord.t.updateTable),
      where: where(ReaperRecord.t),
      limit: limit,
      offset: offset,
      orderBy: orderBy?.call(ReaperRecord.t),
      orderByList: orderByList?.call(ReaperRecord.t),
      transaction: transaction,
      noReturn: noReturn,
    );
  }

  /// Deletes all [ReaperRecord]s in the list and returns the deleted rows.
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
  Future<List<ReaperRecord>> delete(
    _is.DatabaseSession session,
    List<ReaperRecord> rows, {
    _is.OrderByBuilder<ReaperRecordTable>? orderBy,
    _is.OrderByListBuilder<ReaperRecordTable>? orderByList,
    _is.Transaction? transaction,
    bool noReturn = false,
  }) async {
    return session.db.delete<ReaperRecord>(
      rows,
      orderBy: orderBy?.call(ReaperRecord.t),
      orderByList: orderByList?.call(ReaperRecord.t),
      transaction: transaction,
      noReturn: noReturn,
    );
  }

  /// Deletes a single [ReaperRecord].
  Future<ReaperRecord> deleteRow(
    _is.DatabaseSession session,
    ReaperRecord row, {
    _is.Transaction? transaction,
  }) async {
    return session.db.deleteRow<ReaperRecord>(
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
  Future<List<ReaperRecord>> deleteWhere(
    _is.DatabaseSession session, {
    required _is.WhereExpressionBuilder<ReaperRecordTable> where,
    _is.OrderByBuilder<ReaperRecordTable>? orderBy,
    _is.OrderByListBuilder<ReaperRecordTable>? orderByList,
    _is.Transaction? transaction,
    bool noReturn = false,
  }) async {
    return session.db.deleteWhere<ReaperRecord>(
      where: where(ReaperRecord.t),
      orderBy: orderBy?.call(ReaperRecord.t),
      orderByList: orderByList?.call(ReaperRecord.t),
      transaction: transaction,
      noReturn: noReturn,
    );
  }

  /// Counts the number of rows matching the [where] expression. If omitted,
  /// will return the count of all rows in the table.
  Future<int> count(
    _is.DatabaseSession session, {
    _is.WhereExpressionBuilder<ReaperRecordTable>? where,
    int? limit,
    _is.Transaction? transaction,
  }) async {
    return session.db.count<ReaperRecord>(
      where: where?.call(ReaperRecord.t),
      limit: limit,
      transaction: transaction,
    );
  }

  /// Acquires row-level locks on [ReaperRecord] rows matching the [where] expression.
  Future<void> lockRows(
    _is.DatabaseSession session, {
    required _is.WhereExpressionBuilder<ReaperRecordTable> where,
    required _is.LockMode lockMode,
    required _is.Transaction transaction,
    _is.LockBehavior lockBehavior = _is.LockBehavior.wait,
  }) async {
    return session.db.lockRows<ReaperRecord>(
      where: where(ReaperRecord.t),
      lockMode: lockMode,
      lockBehavior: lockBehavior,
      transaction: transaction,
    );
  }
}
