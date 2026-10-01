import 'package:serverpod_auth_idp_server/providers/anonymous.dart';

/// Guest sign-in.
///
/// A fresh install plays at once against an anonymous account. Nothing else
/// about a player is tied to how they signed in, so another way to sign in
/// can be added later without touching what they own.
class AnonymousIdpEndpoint extends AnonymousIdpBaseEndpoint {}
