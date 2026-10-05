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
import 'package:serverpod/serverpod.dart' as _is;

abstract class WalletIdentity
    implements _is.TableRow<int?>, _is.ProtocolSerialization {
  WalletIdentity._({
    this.id,
    required this.address,
    required this.authUserId,
  });

  factory WalletIdentity({
    int? id,
    required String address,
    required _is.UuidValue authUserId,
  }) = _WalletIdentityImpl;

  factory WalletIdentity.fromJson(Map<String, dynamic> jsonSerialization) {
    return WalletIdentity(
      id: jsonSerialization['id'] as int?,
      address: jsonSerialization['address'] as String,
      authUserId: _is.UuidValueJsonExtension.fromJson(
        jsonSerialization['authUserId'],
      ),
    );
  }

  static final t = WalletIdentityTable();

  static const db = WalletIdentityRepository._();

  @override
  int? id;

  String address;

  _is.UuidValue authUserId;

  @override
  _is.Table<int?> get table => t;

  /// Returns a shallow copy of this [WalletIdentity]
  /// with some or all fields replaced by the given arguments.
  @_is.useResult
  WalletIdentity copyWith({
    int? id,
    String? address,
    _is.UuidValue? authUserId,
  });
  @override
  Map<String, dynamic> toJson() {
    return {
      '__className__': 'WalletIdentity',
      if (id != null) 'id': id,
      'address': address,
      'authUserId': authUserId.toJson(),
    };
  }

  @override
  Map<String, dynamic> toJsonForProtocol() {
    return {
      '__className__': 'WalletIdentity',
      if (id != null) 'id': id,
      'address': address,
      'authUserId': authUserId.toJson(),
    };
  }

  static WalletIdentityInclude include() {
    return WalletIdentityInclude._();
  }

  static WalletIdentityIncludeList includeList({
    _is.WhereExpressionBuilder<WalletIdentityTable>? where,
    int? limit,
    int? offset,
    _is.OrderByBuilder<WalletIdentityTable>? orderBy,
    _is.OrderByListBuilder<WalletIdentityTable>? orderByList,
    WalletIdentityInclude? include,
  }) {
    return WalletIdentityIncludeList._(
      where: where,
      limit: limit,
      offset: offset,
      orderBy: orderBy?.call(WalletIdentity.t),
      orderByList: orderByList?.call(WalletIdentity.t),
      include: include,
    );
  }

  @override
  String toString() {
    return _is.SerializationManager.encode(this);
  }
}

class _Undefined {}

class _WalletIdentityImpl extends WalletIdentity {
  _WalletIdentityImpl({
    int? id,
    required String address,
    required _is.UuidValue authUserId,
  }) : super._(
         id: id,
         address: address,
         authUserId: authUserId,
       );

  /// Returns a shallow copy of this [WalletIdentity]
  /// with some or all fields replaced by the given arguments.
  @_is.useResult
  @override
  WalletIdentity copyWith({
    Object? id = _Undefined,
    String? address,
    _is.UuidValue? authUserId,
  }) {
    return WalletIdentity(
      id: id is int? ? id : this.id,
      address: address ?? this.address,
      authUserId: authUserId ?? this.authUserId,
    );
  }
}

class WalletIdentityUpdateTable extends _is.UpdateTable<WalletIdentityTable> {
  WalletIdentityUpdateTable(super.table);

  _is.ColumnValue<String, String> address(String value) => _is.ColumnValue(
    table.address,
    value,
  );

  _is.ColumnValue<_is.UuidValue, _is.UuidValue> authUserId(
    _is.UuidValue value,
  ) => _is.ColumnValue(
    table.authUserId,
    value,
  );
}

class WalletIdentityTable extends _is.Table<int?> {
  WalletIdentityTable({super.tableRelation})
    : super(tableName: 'wallet_identity') {
    updateTable = WalletIdentityUpdateTable(this);
    address = _is.ColumnString(
      'address',
      this,
    );
    authUserId = _is.ColumnUuid(
      'authUserId',
      this,
    );
  }

  late final WalletIdentityUpdateTable updateTable;

  late final _is.ColumnString address;

  late final _is.ColumnUuid authUserId;

  @override
  List<_is.Column> get columns => [
    id,
    address,
    authUserId,
  ];
}

class WalletIdentityInclude extends _is.IncludeObject {
  WalletIdentityInclude._();

  @override
  Map<String, _is.Include?> get includes => {};

  @override
  _is.Table<int?> get table => WalletIdentity.t;
}

class WalletIdentityIncludeList extends _is.IncludeList {
  WalletIdentityIncludeList._({
    _is.WhereExpressionBuilder<WalletIdentityTable>? where,
    super.limit,
    super.offset,
    super.orderBy,
    super.orderByList,
    super.include,
  }) {
    super.where = where?.call(WalletIdentity.t);
  }

  @override
  Map<String, _is.Include?> get includes => include?.includes ?? {};

  @override
  _is.Table<int?> get table => WalletIdentity.t;
}

class WalletIdentityRepository {
  const WalletIdentityRepository._();

  /// Returns a list of [WalletIdentity]s matching the given query parameters.
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
  Future<List<WalletIdentity>> find(
    _is.DatabaseSession session, {
    _is.WhereExpressionBuilder<WalletIdentityTable>? where,
    int? limit,
    int? offset,
    _is.OrderByBuilder<WalletIdentityTable>? orderBy,
    _is.OrderByListBuilder<WalletIdentityTable>? orderByList,
    _is.Transaction? transaction,
    _is.LockMode? lockMode,
    _is.LockBehavior? lockBehavior,
  }) async {
    return session.db.find<WalletIdentity>(
      where: where?.call(WalletIdentity.t),
      orderBy: orderBy?.call(WalletIdentity.t),
      orderByList: orderByList?.call(WalletIdentity.t),
      limit: limit,
      offset: offset,
      transaction: transaction,
      lockMode: lockMode,
      lockBehavior: lockBehavior,
    );
  }

  /// Returns the first matching [WalletIdentity] matching the given query parameters.
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
  Future<WalletIdentity?> findFirstRow(
    _is.DatabaseSession session, {
    _is.WhereExpressionBuilder<WalletIdentityTable>? where,
    int? offset,
    _is.OrderByBuilder<WalletIdentityTable>? orderBy,
    _is.OrderByListBuilder<WalletIdentityTable>? orderByList,
    _is.Transaction? transaction,
    _is.LockMode? lockMode,
    _is.LockBehavior? lockBehavior,
  }) async {
    return session.db.findFirstRow<WalletIdentity>(
      where: where?.call(WalletIdentity.t),
      orderBy: orderBy?.call(WalletIdentity.t),
      orderByList: orderByList?.call(WalletIdentity.t),
      offset: offset,
      transaction: transaction,
      lockMode: lockMode,
      lockBehavior: lockBehavior,
    );
  }

  /// Finds a single [WalletIdentity] by its [id] or null if no such row exists.
  Future<WalletIdentity?> findById(
    _is.DatabaseSession session,
    int id, {
    _is.Transaction? transaction,
    _is.LockMode? lockMode,
    _is.LockBehavior? lockBehavior,
  }) async {
    return session.db.findById<WalletIdentity>(
      id,
      transaction: transaction,
      lockMode: lockMode,
      lockBehavior: lockBehavior,
    );
  }

  /// Inserts all [WalletIdentity]s in the list and returns the inserted rows.
  ///
  /// The returned [WalletIdentity]s will have their `id` fields set.
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
  Future<List<WalletIdentity>> insert(
    _is.DatabaseSession session,
    List<WalletIdentity> rows, {
    _is.Transaction? transaction,
    bool ignoreConflicts = false,
    bool noReturn = false,
  }) async {
    return session.db.insert<WalletIdentity>(
      rows,
      transaction: transaction,
      ignoreConflicts: ignoreConflicts,
      noReturn: noReturn,
    );
  }

  /// Inserts a single [WalletIdentity] and returns the inserted row.
  ///
  /// The returned [WalletIdentity] will have its `id` field set.
  Future<WalletIdentity> insertRow(
    _is.DatabaseSession session,
    WalletIdentity row, {
    _is.Transaction? transaction,
  }) async {
    return session.db.insertRow<WalletIdentity>(
      row,
      transaction: transaction,
    );
  }

  /// Upserts all [WalletIdentity]s in the list and returns the resulting rows.
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
  /// The returned [WalletIdentity]s will have their `id` fields set.
  ///
  /// This is an atomic operation, meaning that if one of the rows fails,
  /// none of the rows will be affected.
  ///
  /// If [noReturn] is set to `true`, the resulting rows are not read back from
  /// the database and an empty list is returned. This avoids the overhead of
  /// transferring and deserializing the rows when the result is not needed.
  Future<List<WalletIdentity>> upsert(
    _is.DatabaseSession session,
    List<WalletIdentity> rows, {
    required _is.ColumnSelections<WalletIdentityTable> conflictColumns,
    _is.ColumnSelections<WalletIdentityTable>? updateColumns,
    _is.WhereExpressionBuilder<WalletIdentityTable>? updateWhere,
    _is.Transaction? transaction,
    bool noReturn = false,
  }) async {
    return session.db.upsert<WalletIdentity>(
      rows,
      conflictColumns: conflictColumns(WalletIdentity.t),
      updateColumns: updateColumns?.call(WalletIdentity.t),
      updateWhere: updateWhere?.call(WalletIdentity.t),
      transaction: transaction,
      noReturn: noReturn,
    );
  }

  /// Upserts a single [WalletIdentity] and returns the resulting row.
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
  /// The returned [WalletIdentity] will have its `id` field set.
  Future<WalletIdentity?> upsertRow(
    _is.DatabaseSession session,
    WalletIdentity row, {
    required _is.ColumnSelections<WalletIdentityTable> conflictColumns,
    _is.ColumnSelections<WalletIdentityTable>? updateColumns,
    _is.WhereExpressionBuilder<WalletIdentityTable>? updateWhere,
    _is.Transaction? transaction,
  }) async {
    return session.db.upsertRow<WalletIdentity>(
      row,
      conflictColumns: conflictColumns(WalletIdentity.t),
      updateColumns: updateColumns?.call(WalletIdentity.t),
      updateWhere: updateWhere?.call(WalletIdentity.t),
      transaction: transaction,
    );
  }

  /// Updates all [WalletIdentity]s in the list and returns the updated rows. If
  /// [columns] is provided, only those columns will be updated. Defaults to
  /// all columns.
  /// This is an atomic operation, meaning that if one of the rows fails to
  /// update, none of the rows will be updated.
  ///
  /// If [noReturn] is set to `true`, the updated rows are not read back from
  /// the database and an empty list is returned. This avoids the overhead of
  /// transferring and deserializing the rows when the result is not needed.
  Future<List<WalletIdentity>> update(
    _is.DatabaseSession session,
    List<WalletIdentity> rows, {
    _is.ColumnSelections<WalletIdentityTable>? columns,
    _is.Transaction? transaction,
    bool noReturn = false,
  }) async {
    return session.db.update<WalletIdentity>(
      rows,
      columns: columns?.call(WalletIdentity.t),
      transaction: transaction,
      noReturn: noReturn,
    );
  }

  /// Updates a single [WalletIdentity]. The row needs to have its id set.
  /// Optionally, a list of [columns] can be provided to only update those
  /// columns. Defaults to all columns.
  Future<WalletIdentity> updateRow(
    _is.DatabaseSession session,
    WalletIdentity row, {
    _is.ColumnSelections<WalletIdentityTable>? columns,
    _is.Transaction? transaction,
  }) async {
    return session.db.updateRow<WalletIdentity>(
      row,
      columns: columns?.call(WalletIdentity.t),
      transaction: transaction,
    );
  }

  /// Updates a single [WalletIdentity] by its [id] with the specified [columnValues].
  /// Returns the updated row or null if no row with the given id exists.
  Future<WalletIdentity?> updateById(
    _is.DatabaseSession session,
    int id, {
    required _is.ColumnValueListBuilder<WalletIdentityUpdateTable> columnValues,
    _is.Transaction? transaction,
  }) async {
    return session.db.updateById<WalletIdentity>(
      id,
      columnValues: columnValues(WalletIdentity.t.updateTable),
      transaction: transaction,
    );
  }

  /// Updates all [WalletIdentity]s matching the [where] expression with the specified [columnValues].
  /// Returns the list of updated rows.
  ///
  /// If [noReturn] is set to `true`, the updated rows are not read back from
  /// the database and an empty list is returned. This avoids the overhead of
  /// transferring and deserializing the rows when the result is not needed.
  Future<List<WalletIdentity>> updateWhere(
    _is.DatabaseSession session, {
    required _is.ColumnValueListBuilder<WalletIdentityUpdateTable> columnValues,
    required _is.WhereExpressionBuilder<WalletIdentityTable> where,
    int? limit,
    int? offset,
    _is.OrderByBuilder<WalletIdentityTable>? orderBy,
    _is.OrderByListBuilder<WalletIdentityTable>? orderByList,
    _is.Transaction? transaction,
    bool noReturn = false,
  }) async {
    return session.db.updateWhere<WalletIdentity>(
      columnValues: columnValues(WalletIdentity.t.updateTable),
      where: where(WalletIdentity.t),
      limit: limit,
      offset: offset,
      orderBy: orderBy?.call(WalletIdentity.t),
      orderByList: orderByList?.call(WalletIdentity.t),
      transaction: transaction,
      noReturn: noReturn,
    );
  }

  /// Deletes all [WalletIdentity]s in the list and returns the deleted rows.
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
  Future<List<WalletIdentity>> delete(
    _is.DatabaseSession session,
    List<WalletIdentity> rows, {
    _is.OrderByBuilder<WalletIdentityTable>? orderBy,
    _is.OrderByListBuilder<WalletIdentityTable>? orderByList,
    _is.Transaction? transaction,
    bool noReturn = false,
  }) async {
    return session.db.delete<WalletIdentity>(
      rows,
      orderBy: orderBy?.call(WalletIdentity.t),
      orderByList: orderByList?.call(WalletIdentity.t),
      transaction: transaction,
      noReturn: noReturn,
    );
  }

  /// Deletes a single [WalletIdentity].
  Future<WalletIdentity> deleteRow(
    _is.DatabaseSession session,
    WalletIdentity row, {
    _is.Transaction? transaction,
  }) async {
    return session.db.deleteRow<WalletIdentity>(
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
  Future<List<WalletIdentity>> deleteWhere(
    _is.DatabaseSession session, {
    required _is.WhereExpressionBuilder<WalletIdentityTable> where,
    _is.OrderByBuilder<WalletIdentityTable>? orderBy,
    _is.OrderByListBuilder<WalletIdentityTable>? orderByList,
    _is.Transaction? transaction,
    bool noReturn = false,
  }) async {
    return session.db.deleteWhere<WalletIdentity>(
      where: where(WalletIdentity.t),
      orderBy: orderBy?.call(WalletIdentity.t),
      orderByList: orderByList?.call(WalletIdentity.t),
      transaction: transaction,
      noReturn: noReturn,
    );
  }

  /// Counts the number of rows matching the [where] expression. If omitted,
  /// will return the count of all rows in the table.
  Future<int> count(
    _is.DatabaseSession session, {
    _is.WhereExpressionBuilder<WalletIdentityTable>? where,
    int? limit,
    _is.Transaction? transaction,
  }) async {
    return session.db.count<WalletIdentity>(
      where: where?.call(WalletIdentity.t),
      limit: limit,
      transaction: transaction,
    );
  }

  /// Acquires row-level locks on [WalletIdentity] rows matching the [where] expression.
  Future<void> lockRows(
    _is.DatabaseSession session, {
    required _is.WhereExpressionBuilder<WalletIdentityTable> where,
    required _is.LockMode lockMode,
    required _is.Transaction transaction,
    _is.LockBehavior lockBehavior = _is.LockBehavior.wait,
  }) async {
    return session.db.lockRows<WalletIdentity>(
      where: where(WalletIdentity.t),
      lockMode: lockMode,
      lockBehavior: lockBehavior,
      transaction: transaction,
    );
  }
}
