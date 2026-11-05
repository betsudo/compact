# PR703 canonical reconciliation

The owner's 5 November request is to resolve PR703's conflicts and CI directly.
It supersedes the earlier instruction to keep that PR head frozen while opening
all prerequisite PRs. The broader Rust AST goal and source-wide review remain
incomplete. Forge remains deliberately deferred.

Preserve both parents: the actual PR701 merge `e20d77f81` and canonical's PR700
merge `e605f962c`. Reconcile by a signed forward merge, without squashing,
rebasing, resetting either line or admitting the separate local follow-up work.
The original source branches, proof identities and failure artifacts stay intact.

## Conflict owners

- Keep the union of compiler workflow qualification steps and receipt uploads,
  including wide-zero and later product-Vector checks. No gate is waived.
- Keep all ten additional workspace members from the two branches and their
  automatic lock merge. Do not alter external dependency versions.
- Retain both changelog histories. A complete local source-branch inventory
  found the highest reserved compiler patch to be192; use193 for this combined
  source, consistently in the compiler and flake. Regenerate the ledger reference
  through its Scheme owner and check the output.
- Review the automatically merged typed return/coercion lowering and proof-runner
  dispatch, not just files Git marked conflicted. Preserve #700 singleton-zero
  behavior and the stack's Vector coercion and source-map publication behavior.

## Validation and delivery

Run workflow/header/public Python controls, formatting, backend/runtime/macro/
testkit checks and strict Clippy. Build the original compiler recipe at the clean
combined head and retain a new immutable recipe receipt. Run the maintained
source/API/refusal and applicable genuine proof/strict-application/replay gates
on the combined source, including Fold, wide-zero and outer-Vector products.
Retain the automatic k<=8 cap and complete original source/IR/Program binding;
keep the original over-cap export intact and unproved.

Push a normal forward update to PR703's existing head, request fresh Claude CLI
review (Codex CLI fallback only if unavailable), and inspect actual current-head
CI. Fix actionable failures at their owner. Required Linux and arm64/compiler/
E2E checks remain mandatory; local results are not hosted acceptance. This plan
records no successful new compiler, proof or CI result before execution.

The parallel146–163 lines, local proof-CI successor, constructor/compiler192/
archive/wallet work remain separately preserved. Their historical receipts are
not relabeled193. Do not merge PR703 or clean worktrees/targets merely because
the conflict repair itself passes focused checks.
