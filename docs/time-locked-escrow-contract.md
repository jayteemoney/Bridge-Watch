# Time-Locked Escrow Contract for Bridge Transfers

Issue: #13

## Scope

This implementation adds a dedicated Soroban escrow contract for bridge-transfer custody with configurable lock windows, challenge/dispute handling, partial and batch releases, fee accounting, and emergency controls.

Implemented in:
- contracts/soroban/src/escrow_contract.rs

## Key Features

- Configurable lock periods per bridge and asset type.
- Escrow creation with transfer metadata and verification references.
- Challenge mechanism limited to lock window.
- Dispute resolution with multi-signature approvals.
- Verification sync integration point for bridge verification contracts.
- Automated release eligibility after lock and successful verification.
- Partial release support.
- Batch release support for operational efficiency.
- Refund flows for failed verification or dispute rejection.
- Fee accrual and fee collection for escrow service operations.
- Emergency pause and emergency recovery procedure.

## Contract API

Core lifecycle functions:
- create_escrow
- challenge_escrow
- resolve_challenge
- release_escrow
- refund_escrow
- extend_lock

Operational and security functions:
- initialize
- set_lock_period
- set_fee_config
- set_approvers
- set_emergency_pause
- emergency_recover
- collect_fees
- sync_verification
- batch_release

## Security Assumptions

- Admin key is trusted for emergency and configuration actions.
- Approver set is independent and threshold > 1 in production.
- Verification contract identity is enforced via `sync_verification` authorization.
- Emergency recovery is disabled unless explicit pause is active.

## Abuse and Failure Paths Covered

- Early release before lock expiry is rejected.
- Unauthorized release/refund/approval actions are rejected.
- Duplicate approver votes are rejected.
- Conflicting dispute decisions are rejected.
- Refund is blocked when escrow is already released/refunded.
- Emergency recovery is blocked unless paused.

## Test Coverage Summary

The escrow module includes deterministic unit tests for:
- lock and verification release flow
- challenge and multisig resolution flow
- dispute-reject refund flow
- batch release and fee collection
- emergency recovery flow

## Notes

This escrow contract currently performs deterministic accounting and authorization checks for transfer lifecycle management. Token settlement wiring can be integrated using asset contracts as a follow-up without changing dispute and lock state semantics.
