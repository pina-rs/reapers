/// Merges the current branch into `main`, locally, once it passes the checks.
///
/// This repository has no pull requests. This is the whole flow:
///
///     git switch -c feat/your-change        # work and commit on a branch
///     dart tool/merge.dart                  # check it, then land it on main
///     dart tool/merge.dart --push           # the same, then push main
///
/// It runs every check in `tool/check.dart` on the commit being merged,
/// records that the commit passed, and fast-forwards `main` to it. The hooks
/// installed by `tool/hooks.dart` refuse to move `main` to a commit without
/// that record, so this is the only way in.
///
/// `main` only ever fast-forwards, which keeps its history a straight line.
/// If `main` has moved since the branch was cut, rebase the branch first.
library;

import 'dart:convert';
import 'dart:io';

import 'check.dart';
import 'hooks.dart';

Future<void> main(List<String> arguments) async {
  if (arguments.any(
    (argument) => const {'help', '--help', '-h'}.contains(argument),
  )) {
    _usage();

    return;
  }

  final unknown = arguments.where((argument) => argument != '--push');

  if (unknown.isNotEmpty) {
    stderr.writeln('Unknown argument "${unknown.first}".\n');
    _usage();
    exit(64);
  }

  final branch = _git(['rev-parse', '--abbrev-ref', 'HEAD']);

  if (branch == 'main' || branch == 'HEAD') {
    _fail(
      'There is nothing to merge from here. Switch to the branch that holds '
      'the work, then run this again.',
    );
  }

  final uncommitted = _git(['status', '--porcelain']);

  if (uncommitted.isNotEmpty) {
    _fail(
      'The working tree has uncommitted changes. Only committed work can be '
      'checked and merged, so commit or stash them first:\n$uncommitted',
    );
  }

  if (!status()) {
    _fail('Install the hooks first, so main cannot be moved around them.');
  }

  final head = _git(['rev-parse', 'HEAD']);
  final base = _git(['rev-parse', 'main']);

  if (head == base) {
    stdout.writeln('$branch is already on main. Nothing to merge.');

    return;
  }

  // If main has commits the branch lacks, merging would put a tree on main
  // that the checks never saw.
  if (!_succeeds(['merge-base', '--is-ancestor', base, head])) {
    _fail(
      'main has moved since $branch was cut. Rebase onto it, then run this '
      'again:\n    git rebase main',
    );
  }

  stdout.writeln('Checking $branch at ${_short(head)} before it reaches main.');

  final report = await runChecks(fast: false);

  if (!report.passed) {
    stderr.writeln('\nThe checks failed, so main was not moved.');
    exit(report.exitCode == 0 ? 1 : report.exitCode);
  }

  // The checks may not leave anything behind, or what was checked is not
  // what is committed.
  if (_git(['status', '--porcelain']).isNotEmpty ||
      _git(['rev-parse', 'HEAD']) != head) {
    _fail(
      'The tree changed while the checks ran. Commit the change and retry.',
    );
  }

  final record = _recordFor(branch)..parent.createSync(recursive: true);

  record.writeAsStringSync(
    '${const JsonEncoder.withIndent('  ').convert({'branch': branch, 'commit': head, 'checks': report.steps, 'verifiedAt': DateTime.now().toUtc().toIso8601String()})}\n',
  );

  try {
    _fastForwardMain(branch);
  } finally {
    // A record is good for one landing. Leaving it would let that commit be
    // put back on main later without being checked again.
    record.deleteSync();
  }

  final count = _git(['rev-list', '--count', '$base..$head']);

  stdout.writeln(
    '✓ main is now at ${_short(head)} ($count commit(s) from $branch).',
  );

  if (!arguments.contains('--push')) {
    stdout.writeln('Push it when you are ready:\n    git push origin main');

    return;
  }

  final push = await Process.start('git', [
    'push',
    'origin',
    'main',
  ], mode: ProcessStartMode.inheritStdio);

  exit(await push.exitCode);
}

/// Moves `main` up to [branch] without anyone having to leave the branch.
void _fastForwardMain(String branch) {
  final mainCheckout = _worktreeOn('main');
  // A checkout that has main out must be the one to move it, or its files
  // would be left behind. With main checked out nowhere, the ref alone moves.
  final result = mainCheckout == null
      ? Process.runSync('git', ['fetch', '--quiet', '.', '$branch:main'])
      : Process.runSync('git', [
          '-C',
          mainCheckout,
          'merge',
          '--ff-only',
          branch,
        ]);

  if (result.exitCode != 0) {
    _fail('main could not be fast-forwarded:\n${result.stderr}');
  }
}

/// The checkout that currently has [branch] out, if any.
String? _worktreeOn(String branch) {
  String? path;

  for (final line in _git(['worktree', 'list', '--porcelain']).split('\n')) {
    if (line.startsWith('worktree ')) {
      path = line.substring('worktree '.length).trim();
    } else if (line.trim() == 'branch refs/heads/$branch') {
      return path;
    }
  }

  return null;
}

File _recordFor(String branch) => File(
  '${commonGitDirectory()}/$verifiedDirectory/'
  '${branch.replaceAll('/', '-')}.json',
);

String _git(List<String> arguments) {
  final result = Process.runSync('git', arguments);

  if (result.exitCode != 0) {
    _fail('git ${arguments.join(' ')} failed:\n${result.stderr}');
  }

  return (result.stdout as String).trim();
}

bool _succeeds(List<String> arguments) =>
    Process.runSync('git', arguments).exitCode == 0;

String _short(String commit) => commit.substring(0, 8);

Never _fail(String message) {
  stderr.writeln(message);
  exit(1);
}

void _usage() {
  stdout.writeln('''
Merge the current branch into main, locally, once it passes the checks.

  dart tool/merge.dart           check this branch, then fast-forward main to it
  dart tool/merge.dart --push    the same, then push main to origin

Run it from the repository root, on the branch to merge, with a clean tree.''');
}
