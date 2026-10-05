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

abstract class ChainStatus
    implements _isc.SerializableModel, _isc.ProtocolSerialization {
  ChainStatus._({
    required this.chain,
    required this.identityUri,
    this.collection,
    this.marketplaceUri,
    this.wallet,
    this.lamports,
  });

  factory ChainStatus({
    required String chain,
    required String identityUri,
    String? collection,
    String? marketplaceUri,
    String? wallet,
    int? lamports,
  }) = _ChainStatusImpl;

  factory ChainStatus.fromJson(Map<String, dynamic> jsonSerialization) {
    return ChainStatus(
      chain: jsonSerialization['chain'] as String,
      identityUri: jsonSerialization['identityUri'] as String,
      collection: jsonSerialization['collection'] as String?,
      marketplaceUri: jsonSerialization['marketplaceUri'] as String?,
      wallet: jsonSerialization['wallet'] as String?,
      lamports: jsonSerialization['lamports'] as int?,
    );
  }

  String chain;

  String identityUri;

  String? collection;

  String? marketplaceUri;

  String? wallet;

  int? lamports;

  /// Returns a shallow copy of this [ChainStatus]
  /// with some or all fields replaced by the given arguments.
  @_isc.useResult
  ChainStatus copyWith({
    String? chain,
    String? identityUri,
    String? collection,
    String? marketplaceUri,
    String? wallet,
    int? lamports,
  });
  @override
  Map<String, dynamic> toJson() {
    return {
      '__className__': 'ChainStatus',
      'chain': chain,
      'identityUri': identityUri,
      if (collection != null) 'collection': collection,
      if (marketplaceUri != null) 'marketplaceUri': marketplaceUri,
      if (wallet != null) 'wallet': wallet,
      if (lamports != null) 'lamports': lamports,
    };
  }

  @override
  Map<String, dynamic> toJsonForProtocol() {
    return {
      '__className__': 'ChainStatus',
      'chain': chain,
      'identityUri': identityUri,
      if (collection != null) 'collection': collection,
      if (marketplaceUri != null) 'marketplaceUri': marketplaceUri,
      if (wallet != null) 'wallet': wallet,
      if (lamports != null) 'lamports': lamports,
    };
  }

  @override
  String toString() {
    return _isc.SerializationManager.encode(this);
  }
}

class _Undefined {}

class _ChainStatusImpl extends ChainStatus {
  _ChainStatusImpl({
    required String chain,
    required String identityUri,
    String? collection,
    String? marketplaceUri,
    String? wallet,
    int? lamports,
  }) : super._(
         chain: chain,
         identityUri: identityUri,
         collection: collection,
         marketplaceUri: marketplaceUri,
         wallet: wallet,
         lamports: lamports,
       );

  /// Returns a shallow copy of this [ChainStatus]
  /// with some or all fields replaced by the given arguments.
  @_isc.useResult
  @override
  ChainStatus copyWith({
    String? chain,
    String? identityUri,
    Object? collection = _Undefined,
    Object? marketplaceUri = _Undefined,
    Object? wallet = _Undefined,
    Object? lamports = _Undefined,
  }) {
    return ChainStatus(
      chain: chain ?? this.chain,
      identityUri: identityUri ?? this.identityUri,
      collection: collection is String? ? collection : this.collection,
      marketplaceUri: marketplaceUri is String?
          ? marketplaceUri
          : this.marketplaceUri,
      wallet: wallet is String? ? wallet : this.wallet,
      lamports: lamports is int? ? lamports : this.lamports,
    );
  }
}
