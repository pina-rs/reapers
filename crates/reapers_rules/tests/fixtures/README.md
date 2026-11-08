# The conformance fixtures

These files are the record of what the rules did before they were Rust. The
game and its server were first written in Dart, with the rules in a package
called `reapers_shared`; `generate.dart` asked that package a few thousand
questions with definite answers and wrote the answers here. `tests/conformance.rs`
asks `reapers_rules` the same questions and expects the same answers, to the
last bit of every double, so that saves, hunt logs and receipts written
before the port still read and replay the same way.

The Dart package was retired on 8 November 2026, and the generator went with
it: it is kept here, as it was, but it cannot run in this tree. The fixtures
are not regenerated. A rule that changes on purpose changes its test, not
the fixture.

To run the generator again, check out the last commit that still had the
Dart code beside it, in a worktree of its own:

```sh
git worktree add ../reapers-dart a7c2456
cd ../reapers-dart
fvm install                      # the Flutter and Dart SDK pinned in .fvmrc
fvm dart pub get
fvm dart run crates/reapers_rules/tests/fixtures/generate.dart
```

`a7c2456` is "docs: say how a burn that did not land is finished", the
last commit before the work that retired the Dart packages; the generator
and `reapers_shared` are both there, and agree. If history has been
rewritten since and that name no longer resolves, take the parent of the
commit that removed `reapers_shared` instead:

```sh
git worktree add ../reapers-dart "$(git rev-list -n 1 HEAD -- reapers_shared)^"
```

The generator writes into its own checkout; copy the files you need from
`../reapers-dart/crates/reapers_rules/tests/fixtures/` into this directory,
then remove the worktree with `git worktree remove ../reapers-dart`.
