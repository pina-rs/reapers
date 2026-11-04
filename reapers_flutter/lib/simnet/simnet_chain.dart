import 'dart:async';
import 'dart:convert';
import 'dart:math';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:reapers_shared/reapers_shared.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:solana_kit/solana_kit.dart' as kit;

/// What a simnet transaction does. These are the four things the chain is
/// meant to do for Reapers (see docs/nfts.md); everything else in the game
/// stays off-chain.
enum SimnetInstruction {
  /// The faucet drops the starting SOL into a new wallet.
  airdrop,

  /// The wallet pays the market and receives an NFT.
  buy,

  /// The wallet moves an NFT into the program's escrow: the husk is loose.
  escrow,

  /// The program burns an escrowed NFT, carrying the keeper's receipt.
  burn,
}

/// What an account does in a transaction, as an explorer would label it.
enum SimnetRole { wallet, faucet, market, mint, escrow, program, keeper }

/// One account a transaction touched.
typedef SimnetAccount = ({SimnetRole role, String address});

/// A transaction on the simnet: signed by the player's burner wallet over a
/// description of what it does, and recorded on the phone.
///
/// It is not a Solana wire transaction. It names the same accounts and moves
/// the same amounts a devnet one would, and its signature is a real Ed25519
/// signature that verifies against the wallet's address.
final class SimnetTransaction {
  const SimnetTransaction({
    required this.signature,
    required this.slot,
    required this.at,
    required this.instruction,
    required this.accounts,
    this.lamports = 0,
    this.nft,
    this.receipt,
    this.receiptSignature,
  });

  factory SimnetTransaction.fromJson(Map<String, dynamic> json) =>
      SimnetTransaction(
        signature: json['signature'] as String,
        slot: json['slot'] as int,
        at: DateTime.parse(json['at'] as String),
        instruction: SimnetInstruction.values.byName(
          json['instruction'] as String,
        ),
        accounts: [
          for (final account in json['accounts'] as List)
            (
              role: SimnetRole.values.byName(account['role'] as String),
              address: account['address'] as String,
            ),
        ],
        lamports: json['lamports'] as int,
        nft: switch (json['nft']) {
          final String id => Nft.parse(id),
          _ => null,
        },
        receipt: switch (json['receipt']) {
          final String bytes => base64Decode(bytes),
          _ => null,
        },
        receiptSignature: switch (json['receiptSignature']) {
          final String bytes => base64Decode(bytes),
          _ => null,
        },
      );

  /// The wallet's signature, in base58, as an explorer shows it.
  final String signature;

  final int slot;
  final DateTime at;
  final SimnetInstruction instruction;
  final List<SimnetAccount> accounts;

  /// SOL moved, in lamports. Zero for a transaction that moves only an NFT.
  final int lamports;

  /// The NFT the transaction moved or burned.
  final Nft? nft;

  /// For a burn: the 98-byte [AbsorptionReceipt] the burn carries, and the
  /// keeper's Ed25519 signature over it.
  final Uint8List? receipt;
  final Uint8List? receiptSignature;

  /// The address that plays [role], if any account does.
  String? account(SimnetRole role) => accounts
      .where((account) => account.role == role)
      .map((account) => account.address)
      .firstOrNull;

  /// Whether the receipt this burn carries is the keeper's: the check the
  /// Ed25519 program would make before the burn is allowed.
  bool get receiptVerifies {
    final keeper = account(SimnetRole.keeper);

    if (receipt == null || receiptSignature == null || keeper == null) {
      return false;
    }

    return kit.verifySignature(
      kit.getPublicKeyFromAddress(kit.Address(keeper)),
      kit.SignatureBytes(receiptSignature!),
      receipt!,
    );
  }

  Map<String, dynamic> toJson() => {
    'signature': signature,
    'slot': slot,
    'at': at.toIso8601String(),
    'instruction': instruction.name,
    'accounts': [
      for (final account in accounts)
        {'role': account.role.name, 'address': account.address},
    ],
    'lamports': lamports,
    'nft': nft?.id,
    'receipt': receipt == null ? null : base64Encode(receipt!),
    'receiptSignature': receiptSignature == null
        ? null
        : base64Encode(receiptSignature!),
  };
}

/// A pretend devnet that lives on the phone, for showing the game where no
/// cluster has the Reapers program.
///
/// It holds a burner wallet (a real Ed25519 key pair, generated here, that
/// never holds anything of value), the program's addresses, and a ledger of
/// every transaction the wallet has signed. It decides nothing about the
/// game: amounts and NFTs are handed to it by whoever keeps the books.
///
/// The wallet's key is kept in plain preferences. It is a simnet key and
/// must never be used anywhere real.
final class SimnetChain {
  SimnetChain._(
    this._preferences,
    this._clock,
    this._random,
    this.latency,
    this._state,
  );

  /// Opens the simnet stored in [preferences], making a new one with a fresh
  /// wallet if there is none.
  ///
  /// Every transaction waits [latency] before it is confirmed, as one sent to
  /// a cluster would. Tests pass zero.
  factory SimnetChain.open(
    SharedPreferences preferences, {
    DateTime Function()? clock,
    Random? random,
    Duration latency = const Duration(milliseconds: 650),
  }) {
    final now = clock ?? () => DateTime.now().toUtc();
    final draw = random ?? Random.secure();
    final stored = preferences.getString(storageKey);
    final chain = SimnetChain._(
      preferences,
      now,
      draw,
      latency,
      stored == null
          ? _Genesis.fresh(draw, now())
          : _Genesis.fromJson((jsonDecode(stored) as Map).cast()),
    );

    if (stored == null) unawaited(chain._save());

    return chain;
  }

  static const storageKey = 'reapers.simnet.v1';

  /// How long a slot lasts, as on Solana's clusters.
  static const slotLength = Duration(milliseconds: 400);

  /// The program, the market and the faucet: addresses made from names, so
  /// every phone's simnet agrees on them. No key exists for any of them.
  static final program = _named('program');
  static final market = _named('market');
  static final faucet = _named('faucet');

  /// The keeper that signs absorption receipts on the simnet. Its key is
  /// derived from a public name, so its receipts prove only that the layout
  /// and the check work, which is what a rehearsal needs.
  static final _keeperKey = kit.createKeyPairFromPrivateKeyBytes(
    Uint8List.fromList(
      sha256.convert(utf8.encode('reapers-simnet:keeper')).bytes,
    ),
  );
  static final keeper = kit.getAddressFromPublicKey(_keeperKey.publicKey).value;

  final SharedPreferences _preferences;
  final DateTime Function() _clock;
  final Random _random;
  final Duration latency;
  _Genesis _state;
  final _posted = StreamController<SimnetTransaction>.broadcast();

  /// The burner wallet's address.
  String get wallet => _state.wallet;

  /// Every transaction, oldest first.
  List<SimnetTransaction> get transactions =>
      List.unmodifiable(_state.transactions);

  /// Each transaction as it is confirmed.
  Stream<SimnetTransaction> get posted => _posted.stream;

  /// The slot the chain is at now.
  int get slot => _slotAt(_clock());

  /// Where [nft] waits while it is loose: the program's escrow for its mint.
  static Future<String> escrowFor(Nft nft) async {
    final (address, _) = await kit.getProgramDerivedAddress(
      programAddress: kit.Address(program),
      seeds: [
        'escrow',
        Uint8List.fromList(sha256.convert(utf8.encode(nft.mockMint)).bytes),
      ],
    );

    return address.value;
  }

  /// Signs [receipt] as the simnet keeper.
  static Uint8List signReceipt(Uint8List receipt) =>
      kit.signBytes(_keeperKey.privateKey, receipt).value;

  /// Signs and records a transaction, and returns it once it is confirmed.
  Future<SimnetTransaction> submit(
    SimnetInstruction instruction, {
    required List<SimnetAccount> accounts,
    int lamports = 0,
    Nft? nft,
    Uint8List? receipt,
    Uint8List? receiptSignature,
  }) async {
    await Future<void>.delayed(latency);

    final at = _clock();
    final slot = _slotAt(at);
    final previous = _state.transactions.lastOrNull?.signature ?? wallet;
    final blockhash = _base58(
      sha256.convert(utf8.encode('$previous:$slot')).bytes,
    );
    final message = utf8.encode(
      [
        'reapers-simnet/1',
        instruction.name,
        blockhash,
        for (final account in accounts)
          '${account.role.name}=${account.address}',
        '$lamports',
        nft?.mockMint ?? '',
        if (receipt != null) base64Encode(receipt),
      ].join('\n'),
    );
    final signature = kit.signBytes(_state.secret, message).value;
    final transaction = SimnetTransaction(
      signature: _base58(signature),
      slot: slot,
      at: at,
      instruction: instruction,
      accounts: accounts,
      lamports: lamports,
      nft: nft,
      receipt: receipt,
      receiptSignature: receiptSignature,
    );

    _state.transactions.add(transaction);
    await _save();
    _posted.add(transaction);

    return transaction;
  }

  /// Forgets every transaction and makes a new wallet: a new player at the
  /// next table.
  Future<void> reset() async {
    _state = _Genesis.fresh(_random, _clock());
    await _save();
  }

  int _slotAt(DateTime at) =>
      1 +
      at.difference(_state.genesis).inMilliseconds ~/ slotLength.inMilliseconds;

  Future<void> _save() =>
      _preferences.setString(storageKey, jsonEncode(_state.toJson()));

  static String _named(String name) => kit
      .getAddressFromPublicKey(
        Uint8List.fromList(
          sha256.convert(utf8.encode('reapers-simnet:$name')).bytes,
        ),
      )
      .value;

  static String _base58(List<int> bytes) =>
      kit.getBase58Decoder().decode(Uint8List.fromList(bytes));
}

/// The stored simnet: the wallet's secret, when the chain began, and what
/// has happened on it.
final class _Genesis {
  _Genesis(this.secret, this.genesis, this.transactions)
    : wallet = kit
          .getAddressFromPublicKey(
            kit.createKeyPairFromPrivateKeyBytes(secret).publicKey,
          )
          .value;

  factory _Genesis.fresh(Random random, DateTime now) => _Genesis(
    Uint8List.fromList([for (var i = 0; i < 32; i++) random.nextInt(256)]),
    now,
    [],
  );

  factory _Genesis.fromJson(Map<String, dynamic> json) => _Genesis(
    base64Decode(json['secret'] as String),
    DateTime.parse(json['genesis'] as String),
    [
      for (final transaction in json['transactions'] as List)
        SimnetTransaction.fromJson((transaction as Map).cast()),
    ],
  );

  final Uint8List secret;
  final String wallet;
  final DateTime genesis;
  final List<SimnetTransaction> transactions;

  Map<String, dynamic> toJson() => {
    'secret': base64Encode(secret),
    'genesis': genesis.toIso8601String(),
    'transactions': [
      for (final transaction in transactions) transaction.toJson(),
    ],
  };
}
