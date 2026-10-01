import 'dart:io';

import 'package:serverpod_auth_idp_server/core.dart';
import 'package:serverpod_auth_idp_server/providers/anonymous.dart';

import 'src/generated/serverpod.dart';
import 'src/receipts/absorption_receipt.dart';

/// The starting point of the Reapers server.
///
/// Secrets are never committed. Serverpod reads `config/passwords.yaml` when
/// it is present and otherwise takes every secret from the environment, so a
/// deployment supplies `SERVERPOD_PASSWORD_<name>` variables while a
/// developer's machine keeps a local passwords file.
Future<void> run(List<String> args) async {
  final pod = Serverpod(args);

  // A game should not ask for a sign-up before the first hunt, so a fresh
  // install plays as a guest. The guest account is what a wallet will be
  // linked to when the on-chain side arrives.
  pod.initializeAuthServices(
    tokenManagerBuilders: [JwtConfigFromPasswords()],
    identityProviderBuilders: [const AnonymousIdpConfig()],
  );

  _requireDeploymentSecrets(pod);

  await pod.start();
}

/// Every secret a deployed server cannot run without: the database password,
/// the service secret, the two keys that sign and pepper sign-in tokens, and
/// the secret the receipt signing key is derived from.
const requiredDeploymentSecrets = [
  'database',
  'serviceSecret',
  'jwtHmacSha512PrivateKey',
  'jwtRefreshTokenHashPepper',
  ReceiptSigner.secretName,
];

/// Fails fast when a deployed server is missing a secret it cannot run
/// without.
///
/// Development and test are exempt: the local passwords file supplies these,
/// and nobody should need production credentials to run the game locally.
void _requireDeploymentSecrets(Serverpod pod) {
  final mode = pod.config.runMode;

  if (mode == ServerpodRunMode.development || mode == ServerpodRunMode.test) {
    return;
  }

  final missing = [
    for (final secret in requiredDeploymentSecrets)
      if ((pod.getPassword(secret) ?? '').trim().isEmpty) secret,
  ];

  if (missing.isEmpty) {
    return;
  }

  stderr.writeln(
    'Refusing to start in "$mode" mode without: ${missing.join(', ')}. '
    'Set SERVERPOD_PASSWORD_<name> for each one.',
  );
  exit(78);
}
