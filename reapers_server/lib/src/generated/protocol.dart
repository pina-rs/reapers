/* AUTOMATICALLY GENERATED CODE DO NOT MODIFY */
/*   To generate run: "serverpod generate"    */

// ignore_for_file: implementation_imports
// ignore_for_file: library_private_types_in_public_api
// ignore_for_file: non_constant_identifier_names
// ignore_for_file: public_member_api_docs
// ignore_for_file: type_literal_in_constant_pattern
// ignore_for_file: use_super_parameters
// ignore_for_file: invalid_use_of_internal_member
// ignore_for_file: dead_code, unnecessary_type_check

// ignore_for_file: no_leading_underscores_for_library_prefixes
import 'package:reapers_shared/reapers_shared.dart' as _izkb583i;
import 'package:serverpod/protocol.dart' as _isp;
import 'package:serverpod/serverpod.dart' as _is;
import 'package:serverpod_auth_core_server/serverpod_auth_core_server.dart'
    as _iacs;
import 'package:serverpod_auth_idp_server/serverpod_auth_idp_server.dart'
    as _iais;
import 'player/player_record.dart' as _iyg31bwx;
import 'player/refused.dart' as _i5uoxhqi;
export 'player/player_record.dart';
export 'player/refused.dart';

class Protocol extends _is.DatabaseSerializationManager {
  Protocol._();

  factory Protocol() => _instance;

  static final Protocol _instance = Protocol._().._registerHostProtocols();

  static List<_isp.TableDefinition> get targetTableDefinitions => [
    _isp.TableDefinition(
      name: 'player_record',
      dartName: 'PlayerRecord',
      schema: 'public',
      module: 'reapers',
      columns: [
        _isp.ColumnDefinition(
          name: 'id',
          columnType: _isp.ColumnType.bigint,
          isNullable: false,
          dartType: 'int?',
          columnDefault: 'serial',
        ),
        _isp.ColumnDefinition(
          name: 'authUserId',
          columnType: _isp.ColumnType.uuid,
          isNullable: false,
          dartType: 'UuidValue',
        ),
        _isp.ColumnDefinition(
          name: 'state',
          columnType: _isp.ColumnType.json,
          isNullable: false,
          dartType: 'package:reapers_shared/reapers_shared.dart:PlayerState',
        ),
        _isp.ColumnDefinition(
          name: 'location',
          columnType: _isp.ColumnType.json,
          isNullable: true,
          dartType: 'package:reapers_shared/reapers_shared.dart:GeoFix?',
        ),
        _isp.ColumnDefinition(
          name: 'createdAt',
          columnType: _isp.ColumnType.timestampWithoutTimeZone,
          isNullable: false,
          dartType: 'DateTime',
          columnDefault: 'now',
        ),
        _isp.ColumnDefinition(
          name: 'updatedAt',
          columnType: _isp.ColumnType.timestampWithoutTimeZone,
          isNullable: false,
          dartType: 'DateTime',
          columnDefault: 'now',
        ),
      ],
      foreignKeys: [],
      indexes: [
        _isp.IndexDefinition(
          indexName: 'player_record__authUserId__unique_idx',
          tableSpace: null,
          elements: [
            _isp.IndexElementDefinition(
              type: _isp.IndexElementDefinitionType.column,
              definition: 'authUserId',
            ),
          ],
          type: 'btree',
          isUnique: true,
          isPrimary: false,
        ),
      ],
      managed: true,
    ),
    ..._iais.Protocol.targetTableDefinitions,
    ..._iacs.Protocol.targetTableDefinitions,
    ..._isp.Protocol.targetTableDefinitions,
  ];

  static String? getClassNameFromObjectJson(dynamic data) {
    if (data is! Map) return null;
    final className = data['__className__'] as String?;
    return className;
  }

  @override
  T deserialize<T>(
    dynamic data, [
    Type? t,
  ]) {
    t ??= T;

    final dataClassName = getClassNameFromObjectJson(data);
    if (dataClassName != null && dataClassName != getClassNameForType(t)) {
      try {
        return deserializeByClassName({
          'className': dataClassName,
          'data': data,
        });
      } on _is.DeserializationClassNameNotFoundException catch (_) {
        // If the className is not recognized (e.g., older client receiving
        // data with a new subtype), fall back to deserializing without the
        // className, using the expected type T.
      }
    }

    if (t == _iyg31bwx.PlayerRecord) {
      return _iyg31bwx.PlayerRecord.fromJson(data) as T;
    }
    if (t == _i5uoxhqi.Refused) {
      return _i5uoxhqi.Refused.fromJson(data) as T;
    }
    if (t == _is.getType<_iyg31bwx.PlayerRecord?>()) {
      return (data != null ? _iyg31bwx.PlayerRecord.fromJson(data) : null) as T;
    }
    if (t == _is.getType<_i5uoxhqi.Refused?>()) {
      return (data != null ? _i5uoxhqi.Refused.fromJson(data) : null) as T;
    }
    if (t == _izkb583i.PlayerState) {
      return _izkb583i.PlayerState.fromJson(data) as T;
    }
    if (t == _is.getType<_izkb583i.GeoFix?>()) {
      return (data != null ? _izkb583i.GeoFix.fromJson(data) : null) as T;
    }
    if (t == _izkb583i.GeoFix) {
      return _izkb583i.GeoFix.fromJson(data) as T;
    }
    if (t == _izkb583i.HuntSettlement) {
      return _izkb583i.HuntSettlement.fromJson(data) as T;
    }
    if (t == _izkb583i.RiftFrame) {
      return _izkb583i.RiftFrame.fromJson(data) as T;
    }
    if (t == _izkb583i.RiftReport) {
      return _izkb583i.RiftReport.fromJson(data) as T;
    }
    if (t == _is.getType<_izkb583i.GeoFix?>()) {
      return (data != null ? _izkb583i.GeoFix.fromJson(data) : null) as T;
    }
    if (t == _is.getType<_izkb583i.PlayerState?>()) {
      return (data != null ? _izkb583i.PlayerState.fromJson(data) : null) as T;
    }
    if (t == _is.getType<_izkb583i.HuntSettlement?>()) {
      return (data != null ? _izkb583i.HuntSettlement.fromJson(data) : null)
          as T;
    }
    if (t == _is.getType<_izkb583i.RiftFrame?>()) {
      return (data != null ? _izkb583i.RiftFrame.fromJson(data) : null) as T;
    }
    if (t == _is.getType<_izkb583i.RiftReport?>()) {
      return (data != null ? _izkb583i.RiftReport.fromJson(data) : null) as T;
    }
    try {
      return _iais.Protocol().deserialize<T>(data, t);
    } on _is.DeserializationTypeNotFoundException catch (_) {}
    try {
      return _iacs.Protocol().deserialize<T>(data, t);
    } on _is.DeserializationTypeNotFoundException catch (_) {}
    try {
      return _isp.Protocol().deserialize<T>(data, t);
    } on _is.DeserializationTypeNotFoundException catch (_) {}
    return super.deserialize<T>(data, t);
  }

  static String? getClassNameForType(Type type) {
    return switch (type) {
      _izkb583i.GeoFix => 'GeoFix',
      _izkb583i.PlayerState => 'PlayerState',
      _izkb583i.HuntSettlement => 'HuntSettlement',
      _izkb583i.RiftFrame => 'RiftFrame',
      _izkb583i.RiftReport => 'RiftReport',
      _iyg31bwx.PlayerRecord => 'PlayerRecord',
      _i5uoxhqi.Refused => 'Refused',
      _ => null,
    };
  }

  @override
  String? getClassNameForObject(Object? data) {
    String? className = super.getClassNameForObject(data);
    if (className != null) return className;

    if (data is Map<String, dynamic> && data['__className__'] is String) {
      return (data['__className__'] as String).replaceFirst('reapers.', '');
    }

    switch (data) {
      case _izkb583i.GeoFix():
        return 'GeoFix';
      case _izkb583i.PlayerState():
        return 'PlayerState';
      case _izkb583i.HuntSettlement():
        return 'HuntSettlement';
      case _izkb583i.RiftFrame():
        return 'RiftFrame';
      case _izkb583i.RiftReport():
        return 'RiftReport';
      case _iyg31bwx.PlayerRecord():
        return 'PlayerRecord';
      case _i5uoxhqi.Refused():
        return 'Refused';
    }
    className = _iais.Protocol().getClassNameForObject(data);
    if (className != null) {
      return className.contains('.')
          ? className
          : 'serverpod_auth_idp.$className';
    }
    className = _iacs.Protocol().getClassNameForObject(data);
    if (className != null) {
      return className.contains('.')
          ? className
          : 'serverpod_auth_core.$className';
    }
    className = _isp.Protocol().getClassNameForObject(data);
    if (className != null) {
      return className.contains('.') ? className : 'serverpod.$className';
    }
    return null;
  }

  @override
  dynamic deserializeByClassName(Map<String, dynamic> data) {
    var dataClassName = data['className'];
    if (dataClassName is! String) {
      return super.deserializeByClassName(data);
    }
    if (dataClassName == 'GeoFix') {
      return deserialize<_izkb583i.GeoFix>(data['data']);
    }
    if (dataClassName == 'PlayerState') {
      return deserialize<_izkb583i.PlayerState>(data['data']);
    }
    if (dataClassName == 'HuntSettlement') {
      return deserialize<_izkb583i.HuntSettlement>(data['data']);
    }
    if (dataClassName == 'RiftFrame') {
      return deserialize<_izkb583i.RiftFrame>(data['data']);
    }
    if (dataClassName == 'RiftReport') {
      return deserialize<_izkb583i.RiftReport>(data['data']);
    }
    if (dataClassName == 'PlayerRecord') {
      return deserialize<_iyg31bwx.PlayerRecord>(data['data']);
    }
    if (dataClassName == 'Refused') {
      return deserialize<_i5uoxhqi.Refused>(data['data']);
    }
    if (dataClassName.startsWith('serverpod_auth_idp.')) {
      data['className'] = dataClassName.substring(19);
      return _iais.Protocol().deserializeByClassName(data);
    }
    if (dataClassName.startsWith('serverpod_auth_core.')) {
      data['className'] = dataClassName.substring(20);
      return _iacs.Protocol().deserializeByClassName(data);
    }
    if (dataClassName.startsWith('serverpod.')) {
      data['className'] = dataClassName.substring(10);
      return _isp.Protocol().deserializeByClassName(data);
    }
    return super.deserializeByClassName(data);
  }

  void _registerHostProtocols() {
    _iais.Protocol().registerHostProtocol('reapers', this);
    _iacs.Protocol().registerHostProtocol('reapers', this);
  }

  @override
  _is.Table? getTableForType(Type t) {
    {
      var table = _iais.Protocol().getTableForType(t);
      if (table != null) {
        return table;
      }
    }
    {
      var table = _iacs.Protocol().getTableForType(t);
      if (table != null) {
        return table;
      }
    }
    {
      var table = _isp.Protocol().getTableForType(t);
      if (table != null) {
        return table;
      }
    }
    switch (t) {
      case _iyg31bwx.PlayerRecord:
        return _iyg31bwx.PlayerRecord.t;
    }
    return null;
  }

  @override
  List<_isp.TableDefinition> getTargetTableDefinitions() =>
      targetTableDefinitions;

  @override
  String getModuleName() => 'reapers';

  /// Maps any `Record`s known to this [Protocol] to their JSON representation
  ///
  /// Throws in case the record type is not known.
  ///
  /// This method will return `null` (only) for `null` inputs.
  Map<String, dynamic>? mapRecordToJson(Record? record) {
    if (record == null) {
      return null;
    }
    try {
      return _iais.Protocol().mapRecordToJson(record);
    } catch (_) {}
    try {
      return _iacs.Protocol().mapRecordToJson(record);
    } catch (_) {}
    throw Exception('Unsupported record type ${record.runtimeType}');
  }
}
