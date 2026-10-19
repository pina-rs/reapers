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
import 'dart:typed_data' as _idt;
import 'package:reapers_shared/src/geo.dart' as _i3e1ekpd;
import 'package:reapers_shared/src/rift/rift_frame.dart' as _ivi8svxc;
import 'package:serverpod/serverpod.dart' as _is;
import 'package:serverpod_auth_core_server/serverpod_auth_core_server.dart'
    as _iacs;
import 'package:serverpod_auth_idp_server/serverpod_auth_idp_server.dart'
    as _iais;
import '../auth/anonymous_idp_endpoint.dart' as _in0zita6;
import '../auth/jwt_refresh_endpoint.dart' as _inwq3ztq;
import '../player/player_endpoint.dart' as _ikcol2tc;
import '../rift/rift_endpoint.dart' as _iz5xxncc;
import '../solana/solana_endpoint.dart' as _ic4phh6x;

class Endpoints extends _is.EndpointDispatch {
  @override
  void initializeEndpoints(_is.Server server) {
    var endpoints = <String, _is.Endpoint>{
      'anonymousIdp': _in0zita6.AnonymousIdpEndpoint()
        ..initialize(
          server,
          'anonymousIdp',
          null,
        ),
      'jwtRefresh': _inwq3ztq.JwtRefreshEndpoint()
        ..initialize(
          server,
          'jwtRefresh',
          null,
        ),
      'player': _ikcol2tc.PlayerEndpoint()
        ..initialize(
          server,
          'player',
          null,
        ),
      'rift': _iz5xxncc.RiftEndpoint()
        ..initialize(
          server,
          'rift',
          null,
        ),
      'solana': _ic4phh6x.SolanaEndpoint()
        ..initialize(
          server,
          'solana',
          null,
        ),
    };
    connectors['anonymousIdp'] = _is.EndpointConnector(
      name: 'anonymousIdp',
      endpoint: endpoints['anonymousIdp']!,
      methodConnectors: {
        'login': _is.MethodConnector(
          name: 'login',
          params: {
            'token': _is.ParameterDescription(
              name: 'token',
              type: _is.getType<String?>(),
              nullable: true,
            ),
          },
          call:
              (
                _is.Session session,
                Map<String, dynamic> params,
              ) async =>
                  (endpoints['anonymousIdp'] as _in0zita6.AnonymousIdpEndpoint)
                      .login(
                        session,
                        token: params['token'],
                      ),
        ),
      },
    );
    connectors['jwtRefresh'] = _is.EndpointConnector(
      name: 'jwtRefresh',
      endpoint: endpoints['jwtRefresh']!,
      methodConnectors: {
        'refreshAccessToken': _is.MethodConnector(
          name: 'refreshAccessToken',
          params: {
            'refreshToken': _is.ParameterDescription(
              name: 'refreshToken',
              type: _is.getType<String?>(),
              nullable: true,
            ),
          },
          call:
              (
                _is.Session session,
                Map<String, dynamic> params,
              ) async =>
                  (endpoints['jwtRefresh'] as _inwq3ztq.JwtRefreshEndpoint)
                      .refreshAccessToken(
                        session,
                        refreshToken: params['refreshToken'],
                      ),
        ),
      },
    );
    connectors['player'] = _is.EndpointConnector(
      name: 'player',
      endpoint: endpoints['player']!,
      methodConnectors: {
        'load': _is.MethodConnector(
          name: 'load',
          params: {},
          call:
              (
                _is.Session session,
                Map<String, dynamic> params,
              ) async => (endpoints['player'] as _ikcol2tc.PlayerEndpoint).load(
                session,
              ),
        ),
        'updateLocation': _is.MethodConnector(
          name: 'updateLocation',
          params: {
            'fix': _is.ParameterDescription(
              name: 'fix',
              type: _is.getType<_i3e1ekpd.GeoFix?>(),
              nullable: true,
            ),
          },
          call:
              (
                _is.Session session,
                Map<String, dynamic> params,
              ) async => (endpoints['player'] as _ikcol2tc.PlayerEndpoint)
                  .updateLocation(
                    session,
                    params['fix'],
                  ),
        ),
        'location': _is.MethodConnector(
          name: 'location',
          params: {},
          call:
              (
                _is.Session session,
                Map<String, dynamic> params,
              ) async => (endpoints['player'] as _ikcol2tc.PlayerEndpoint)
                  .location(session),
        ),
        'takeOath': _is.MethodConnector(
          name: 'takeOath',
          params: {},
          call:
              (
                _is.Session session,
                Map<String, dynamic> params,
              ) async => (endpoints['player'] as _ikcol2tc.PlayerEndpoint)
                  .takeOath(session),
        ),
        'selectReaper': _is.MethodConnector(
          name: 'selectReaper',
          params: {
            'id': _is.ParameterDescription(
              name: 'id',
              type: _is.getType<String>(),
              nullable: false,
            ),
          },
          call:
              (
                _is.Session session,
                Map<String, dynamic> params,
              ) async => (endpoints['player'] as _ikcol2tc.PlayerEndpoint)
                  .selectReaper(
                    session,
                    params['id'],
                  ),
        ),
        'acquire': _is.MethodConnector(
          name: 'acquire',
          params: {
            'nftId': _is.ParameterDescription(
              name: 'nftId',
              type: _is.getType<String>(),
              nullable: false,
            ),
          },
          call:
              (
                _is.Session session,
                Map<String, dynamic> params,
              ) async =>
                  (endpoints['player'] as _ikcol2tc.PlayerEndpoint).acquire(
                    session,
                    params['nftId'],
                  ),
        ),
        'release': _is.MethodConnector(
          name: 'release',
          params: {
            'nftId': _is.ParameterDescription(
              name: 'nftId',
              type: _is.getType<String>(),
              nullable: false,
            ),
          },
          call:
              (
                _is.Session session,
                Map<String, dynamic> params,
              ) async =>
                  (endpoints['player'] as _ikcol2tc.PlayerEndpoint).release(
                    session,
                    params['nftId'],
                  ),
        ),
        'settleHunt': _is.MethodConnector(
          name: 'settleHunt',
          params: {
            'sightingId': _is.ParameterDescription(
              name: 'sightingId',
              type: _is.getType<int>(),
              nullable: false,
            ),
            'log': _is.ParameterDescription(
              name: 'log',
              type: _is.getType<_idt.ByteData>(),
              nullable: false,
            ),
          },
          call:
              (
                _is.Session session,
                Map<String, dynamic> params,
              ) async =>
                  (endpoints['player'] as _ikcol2tc.PlayerEndpoint).settleHunt(
                    session,
                    params['sightingId'],
                    params['log'],
                  ),
        ),
        'receiptKey': _is.MethodConnector(
          name: 'receiptKey',
          params: {},
          call:
              (
                _is.Session session,
                Map<String, dynamic> params,
              ) async => (endpoints['player'] as _ikcol2tc.PlayerEndpoint)
                  .receiptKey(session),
        ),
        'buyWeapon': _is.MethodConnector(
          name: 'buyWeapon',
          params: {
            'weaponId': _is.ParameterDescription(
              name: 'weaponId',
              type: _is.getType<String>(),
              nullable: false,
            ),
          },
          call:
              (
                _is.Session session,
                Map<String, dynamic> params,
              ) async =>
                  (endpoints['player'] as _ikcol2tc.PlayerEndpoint).buyWeapon(
                    session,
                    params['weaponId'],
                  ),
        ),
        'wield': _is.MethodConnector(
          name: 'wield',
          params: {
            'weaponId': _is.ParameterDescription(
              name: 'weaponId',
              type: _is.getType<String>(),
              nullable: false,
            ),
          },
          call:
              (
                _is.Session session,
                Map<String, dynamic> params,
              ) async =>
                  (endpoints['player'] as _ikcol2tc.PlayerEndpoint).wield(
                    session,
                    params['weaponId'],
                  ),
        ),
        'buyOutfit': _is.MethodConnector(
          name: 'buyOutfit',
          params: {
            'outfit': _is.ParameterDescription(
              name: 'outfit',
              type: _is.getType<String>(),
              nullable: false,
            ),
          },
          call:
              (
                _is.Session session,
                Map<String, dynamic> params,
              ) async =>
                  (endpoints['player'] as _ikcol2tc.PlayerEndpoint).buyOutfit(
                    session,
                    params['outfit'],
                  ),
        ),
        'wear': _is.MethodConnector(
          name: 'wear',
          params: {
            'outfit': _is.ParameterDescription(
              name: 'outfit',
              type: _is.getType<String>(),
              nullable: false,
            ),
          },
          call:
              (
                _is.Session session,
                Map<String, dynamic> params,
              ) async => (endpoints['player'] as _ikcol2tc.PlayerEndpoint).wear(
                session,
                params['outfit'],
              ),
        ),
        'attune': _is.MethodConnector(
          name: 'attune',
          params: {
            'art': _is.ParameterDescription(
              name: 'art',
              type: _is.getType<String>(),
              nullable: false,
            ),
          },
          call:
              (
                _is.Session session,
                Map<String, dynamic> params,
              ) async =>
                  (endpoints['player'] as _ikcol2tc.PlayerEndpoint).attune(
                    session,
                    params['art'],
                  ),
        ),
      },
    );
    connectors['rift'] = _is.EndpointConnector(
      name: 'rift',
      endpoint: endpoints['rift']!,
      methodConnectors: {
        'enter': _is.MethodStreamConnector(
          name: 'enter',
          params: {
            'site': _is.ParameterDescription(
              name: 'site',
              type: _is.getType<String>(),
              nullable: false,
            ),
          },
          streamParams: {
            'reports': _is.StreamParameterDescription<_ivi8svxc.RiftReport>(
              name: 'reports',
              nullable: false,
            ),
          },
          returnType: _is.MethodStreamReturnType.streamType,
          call:
              (
                _is.Session session,
                Map<String, dynamic> params,
                Map<String, Stream> streamParams,
              ) => (endpoints['rift'] as _iz5xxncc.RiftEndpoint).enter(
                session,
                params['site'],
                streamParams['reports']!.cast<_ivi8svxc.RiftReport>(),
              ),
        ),
      },
    );
    connectors['solana'] = _is.EndpointConnector(
      name: 'solana',
      endpoint: endpoints['solana']!,
      methodConnectors: {
        'status': _is.MethodConnector(
          name: 'status',
          params: {},
          call:
              (
                _is.Session session,
                Map<String, dynamic> params,
              ) async => (endpoints['solana'] as _ic4phh6x.SolanaEndpoint)
                  .status(session),
        ),
        'challenge': _is.MethodConnector(
          name: 'challenge',
          params: {
            'address': _is.ParameterDescription(
              name: 'address',
              type: _is.getType<String>(),
              nullable: false,
            ),
          },
          call:
              (
                _is.Session session,
                Map<String, dynamic> params,
              ) async =>
                  (endpoints['solana'] as _ic4phh6x.SolanaEndpoint).challenge(
                    session,
                    params['address'],
                  ),
        ),
        'login': _is.MethodConnector(
          name: 'login',
          params: {
            'challengeId': _is.ParameterDescription(
              name: 'challengeId',
              type: _is.getType<_is.UuidValue>(),
              nullable: false,
            ),
            'signature': _is.ParameterDescription(
              name: 'signature',
              type: _is.getType<String>(),
              nullable: false,
            ),
          },
          call:
              (
                _is.Session session,
                Map<String, dynamic> params,
              ) async =>
                  (endpoints['solana'] as _ic4phh6x.SolanaEndpoint).login(
                    session,
                    params['challengeId'],
                    params['signature'],
                  ),
        ),
        'importReaper': _is.MethodConnector(
          name: 'importReaper',
          params: {
            'asset': _is.ParameterDescription(
              name: 'asset',
              type: _is.getType<String>(),
              nullable: false,
            ),
          },
          call:
              (
                _is.Session session,
                Map<String, dynamic> params,
              ) async => (endpoints['solana'] as _ic4phh6x.SolanaEndpoint)
                  .importReaper(
                    session,
                    params['asset'],
                  ),
        ),
      },
    );
    modules['serverpod_auth_idp'] = _iais.Endpoints()
      ..initializeEndpoints(server);
    modules['serverpod_auth_core'] = _iacs.Endpoints()
      ..initializeEndpoints(server);
  }
}
