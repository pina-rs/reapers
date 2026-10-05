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

abstract class WalletIdentity
    implements _isc.SerializableModel, _isc.ProtocolSerialization {
  WalletIdentity._({
    this.id,
    required this.address,
    required this.authUserId,
  });

  factory WalletIdentity({
    int? id,
    required String address,
    required _isc.UuidValue authUserId,
  }) = _WalletIdentityImpl;

  factory WalletIdentity.fromJson(Map<String, dynamic> jsonSerialization) {
    return WalletIdentity(
      id: jsonSerialization['id'] as int?,
      address: jsonSerialization['address'] as String,
      authUserId: _isc.UuidValueJsonExtension.fromJson(
        jsonSerialization['authUserId'],
      ),
    );
  }

  /// The database id, set if the object has been inserted into the
  /// database or if it has been fetched from the database. Otherwise,
  /// the id will be null.
  int? id;

  String address;

  _isc.UuidValue authUserId;

  /// Returns a shallow copy of this [WalletIdentity]
  /// with some or all fields replaced by the given arguments.
  @_isc.useResult
  WalletIdentity copyWith({
    int? id,
    String? address,
    _isc.UuidValue? authUserId,
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

  @override
  String toString() {
    return _isc.SerializationManager.encode(this);
  }
}

class _Undefined {}

class _WalletIdentityImpl extends WalletIdentity {
  _WalletIdentityImpl({
    int? id,
    required String address,
    required _isc.UuidValue authUserId,
  }) : super._(
         id: id,
         address: address,
         authUserId: authUserId,
       );

  /// Returns a shallow copy of this [WalletIdentity]
  /// with some or all fields replaced by the given arguments.
  @_isc.useResult
  @override
  WalletIdentity copyWith({
    Object? id = _Undefined,
    String? address,
    _isc.UuidValue? authUserId,
  }) {
    return WalletIdentity(
      id: id is int? ? id : this.id,
      address: address ?? this.address,
      authUserId: authUserId ?? this.authUserId,
    );
  }
}
