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
import 'package:serverpod_client/serverpod_client.dart' as _isc;

abstract class WalletChallenge
    implements _isc.SerializableModel, _isc.ProtocolSerialization {
  WalletChallenge._({
    this.id,
    required this.challengeId,
    required this.authUserId,
    required this.address,
    required this.message,
    required this.expiresAt,
    bool? consumed,
  }) : consumed = consumed ?? false;

  factory WalletChallenge({
    int? id,
    required _isc.UuidValue challengeId,
    required _isc.UuidValue authUserId,
    required String address,
    required String message,
    required DateTime expiresAt,
    bool? consumed,
  }) = _WalletChallengeImpl;

  factory WalletChallenge.fromJson(Map<String, dynamic> jsonSerialization) {
    return WalletChallenge(
      id: jsonSerialization['id'] as int?,
      challengeId: _isc.UuidValueJsonExtension.fromJson(
        jsonSerialization['challengeId'],
      ),
      authUserId: _isc.UuidValueJsonExtension.fromJson(
        jsonSerialization['authUserId'],
      ),
      address: jsonSerialization['address'] as String,
      message: jsonSerialization['message'] as String,
      expiresAt: _isc.DateTimeJsonExtension.fromJson(
        jsonSerialization['expiresAt'],
      ),
      consumed: jsonSerialization['consumed'] == null
          ? null
          : _isc.BoolJsonExtension.fromJson(jsonSerialization['consumed']),
    );
  }

  /// The database id, set if the object has been inserted into the
  /// database or if it has been fetched from the database. Otherwise,
  /// the id will be null.
  int? id;

  _isc.UuidValue challengeId;

  _isc.UuidValue authUserId;

  String address;

  String message;

  DateTime expiresAt;

  bool consumed;

  /// Returns a shallow copy of this [WalletChallenge]
  /// with some or all fields replaced by the given arguments.
  @_isc.useResult
  WalletChallenge copyWith({
    int? id,
    _isc.UuidValue? challengeId,
    _isc.UuidValue? authUserId,
    String? address,
    String? message,
    DateTime? expiresAt,
    bool? consumed,
  });
  @override
  Map<String, dynamic> toJson() {
    return {
      '__className__': 'WalletChallenge',
      if (id != null) 'id': id,
      'challengeId': challengeId.toJson(),
      'authUserId': authUserId.toJson(),
      'address': address,
      'message': message,
      'expiresAt': expiresAt.toJson(),
      'consumed': consumed,
    };
  }

  @override
  Map<String, dynamic> toJsonForProtocol() {
    return {
      '__className__': 'WalletChallenge',
      if (id != null) 'id': id,
      'challengeId': challengeId.toJson(),
      'authUserId': authUserId.toJson(),
      'address': address,
      'message': message,
      'expiresAt': expiresAt.toJson(),
      'consumed': consumed,
    };
  }

  @override
  String toString() {
    return _isc.SerializationManager.encode(this);
  }
}

class _Undefined {}

class _WalletChallengeImpl extends WalletChallenge {
  _WalletChallengeImpl({
    int? id,
    required _isc.UuidValue challengeId,
    required _isc.UuidValue authUserId,
    required String address,
    required String message,
    required DateTime expiresAt,
    bool? consumed,
  }) : super._(
         id: id,
         challengeId: challengeId,
         authUserId: authUserId,
         address: address,
         message: message,
         expiresAt: expiresAt,
         consumed: consumed,
       );

  /// Returns a shallow copy of this [WalletChallenge]
  /// with some or all fields replaced by the given arguments.
  @_isc.useResult
  @override
  WalletChallenge copyWith({
    Object? id = _Undefined,
    _isc.UuidValue? challengeId,
    _isc.UuidValue? authUserId,
    String? address,
    String? message,
    DateTime? expiresAt,
    bool? consumed,
  }) {
    return WalletChallenge(
      id: id is int? ? id : this.id,
      challengeId: challengeId ?? this.challengeId,
      authUserId: authUserId ?? this.authUserId,
      address: address ?? this.address,
      message: message ?? this.message,
      expiresAt: expiresAt ?? this.expiresAt,
      consumed: consumed ?? this.consumed,
    );
  }
}
