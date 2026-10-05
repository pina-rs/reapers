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

/// One canonical growth record per NFT, shared across every owner.
abstract class ReaperRecord
    implements _isc.SerializableModel, _isc.ProtocolSerialization {
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

  /// The database id, set if the object has been inserted into the
  /// database or if it has been fetched from the database. Otherwise,
  /// the id will be null.
  int? id;

  String assetKey;

  String collection;

  _izkb583i.ReaperState state;

  /// Returns a shallow copy of this [ReaperRecord]
  /// with some or all fields replaced by the given arguments.
  @_isc.useResult
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
          state is _isc.ProtocolSerialization
          ? (state as _isc.ProtocolSerialization).toJsonForProtocol()
          :
            // ignore: dead_code
            state.toJson(),
    };
  }

  @override
  String toString() {
    return _isc.SerializationManager.encode(this);
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
  @_isc.useResult
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
