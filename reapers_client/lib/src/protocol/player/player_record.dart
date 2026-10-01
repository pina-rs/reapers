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
import 'package:serverpod_client/serverpod_client.dart' as _isc;

/// One player's books: everything they own and have done.
///
/// The state is the shared `PlayerState`, stored whole. The server never
/// edits it field by field: every change is one of the shared rules applied
/// to the whole state, exactly as the offline game applies it.
abstract class PlayerRecord
    implements _isc.SerializableModel, _isc.ProtocolSerialization {
  PlayerRecord._({
    this.id,
    required this.authUserId,
    required this.state,
    DateTime? createdAt,
    DateTime? updatedAt,
  }) : createdAt = createdAt ?? DateTime.now(),
       updatedAt = updatedAt ?? DateTime.now();

  factory PlayerRecord({
    int? id,
    required _isc.UuidValue authUserId,
    required _izkb583i.PlayerState state,
    DateTime? createdAt,
    DateTime? updatedAt,
  }) = _PlayerRecordImpl;

  factory PlayerRecord.fromJson(Map<String, dynamic> jsonSerialization) {
    return PlayerRecord(
      id: jsonSerialization['id'] as int?,
      authUserId: _isc.UuidValueJsonExtension.fromJson(
        jsonSerialization['authUserId'],
      ),
      state: _izkb583i.PlayerState.fromJson(jsonSerialization['state']),
      createdAt: jsonSerialization['createdAt'] == null
          ? null
          : _isc.DateTimeJsonExtension.fromJson(jsonSerialization['createdAt']),
      updatedAt: jsonSerialization['updatedAt'] == null
          ? null
          : _isc.DateTimeJsonExtension.fromJson(jsonSerialization['updatedAt']),
    );
  }

  /// The database id, set if the object has been inserted into the
  /// database or if it has been fetched from the database. Otherwise,
  /// the id will be null.
  int? id;

  /// The signed-in user these books belong to.
  _isc.UuidValue authUserId;

  /// The player as they stand now.
  _izkb583i.PlayerState state;

  /// When the books were opened.
  DateTime createdAt;

  /// When they last changed.
  DateTime updatedAt;

  /// Returns a shallow copy of this [PlayerRecord]
  /// with some or all fields replaced by the given arguments.
  @_isc.useResult
  PlayerRecord copyWith({
    int? id,
    _isc.UuidValue? authUserId,
    _izkb583i.PlayerState? state,
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
          state is _isc.ProtocolSerialization
          ? (state as _isc.ProtocolSerialization).toJsonForProtocol()
          :
            // ignore: dead_code
            state.toJson(),
      'createdAt': createdAt.toJson(),
      'updatedAt': updatedAt.toJson(),
    };
  }

  @override
  String toString() {
    return _isc.SerializationManager.encode(this);
  }
}

class _Undefined {}

class _PlayerRecordImpl extends PlayerRecord {
  _PlayerRecordImpl({
    int? id,
    required _isc.UuidValue authUserId,
    required _izkb583i.PlayerState state,
    DateTime? createdAt,
    DateTime? updatedAt,
  }) : super._(
         id: id,
         authUserId: authUserId,
         state: state,
         createdAt: createdAt,
         updatedAt: updatedAt,
       );

  /// Returns a shallow copy of this [PlayerRecord]
  /// with some or all fields replaced by the given arguments.
  @_isc.useResult
  @override
  PlayerRecord copyWith({
    Object? id = _Undefined,
    _isc.UuidValue? authUserId,
    _izkb583i.PlayerState? state,
    DateTime? createdAt,
    DateTime? updatedAt,
  }) {
    return PlayerRecord(
      id: id is int? ? id : this.id,
      authUserId: authUserId ?? this.authUserId,
      state: state ?? this.state.copyWith(),
      createdAt: createdAt ?? this.createdAt,
      updatedAt: updatedAt ?? this.updatedAt,
    );
  }
}
