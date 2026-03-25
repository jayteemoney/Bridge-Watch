# Insurance Pool Contract for Bridge Risk Protection

## Scope

This document describes the Soroban insurance pool contract implemented for issue #10 in Bridge-Watch.

Implemented in:
- contracts/soroban/src/insurance_pool.rs

The contract focuses on deterministic on-chain accounting for:
- staking insurance liquidity
- risk-aware premium pricing
- claim submission and verification
- multi-signature claim approval
- payout execution
- withdrawal queue handling
- anti-fraud slashing controls
- historical claim and payout tracking

## Core Data Model

### GovernanceConfig
- approvers: signer set allowed to approve verified claims
- approval_threshold: minimum approvals required to move a claim to Approved
- withdrawal_delay_secs: queue lock period for withdrawal requests

### PoolInfo
- pool_id / asset_code for multi-asset coverage pools
- total_liquidity, staked_liquidity, queued_withdrawals
- active_coverage
- premium_rate_bps, risk_score_bps
- acc_premium_per_share, premium_collected_total
- payout_total, paid_claims, rejected_claims

### StakerPosition
- staked_amount and pending_withdrawal per staker per pool
- premium reward accounting with reward_debt and accrued_premium
- claimed_premium and slashed_total audit fields

### ClaimInfo
- claimant, pool_id, amount, evidence_hash
- status lifecycle: Submitted -> Verified -> Approved -> Paid
- approvals list for multisig accounting
- slash_bps and slashed_amount for rejected/fraudulent claims

### WithdrawalRequest
- time-locked queue request with unlock_time and claimed marker

## Functional Behavior

### Staking and Liquidity Management
- `stake_liquidity` increases pool and position balances.
- `request_withdrawal` moves stake into queued state (not immediately withdrawn).
- `execute_withdrawal` enforces time-lock and then decreases pool liquidity.

This queue structure prevents instant liquidity drain and reduces solvency shock risk.

### Premium Pricing and Distribution
- `quote_premium` and `purchase_coverage` compute premium from:
  - base pool premium rate
  - pool risk score
  - coverage tier multiplier
- Coverage tiers also enforce maximum utilization caps.
- Premium is distributed to stakers via cumulative per-share accounting.

### Claims Workflow
- `submit_claim` requires positive amount and existing active coverage.
- `verify_claim` is admin-gated and supports valid/invalid outcomes.
- Invalid claims can trigger slashing (`slash_bps`) against claimant stake.
- `approve_claim` requires signer membership in governance approvers.
- Claim status becomes Approved only after threshold approvals.
- `execute_payout` is admin-gated and checks liquidity and coverage accounting.

## Security Assumptions

- Admin key is trusted for governance configuration, risk score updates, and claim verification.
- Approver keys are independent and securely controlled.
- Multi-sig threshold is configured above 1 for production.
- Premium and payout token transfer rails are out-of-scope in this accounting-only contract; external transfer integrations must enforce equivalent authorization and replay protections.

## Abuse and Failure Path Coverage

Tests cover:
- unauthorized claim approval attempts
- duplicate approver votes
- conservative tier coverage cap enforcement
- withdrawal execution before queue delay expiry
- fraudulent claim rejection with slashing
- full verified claim flow with multisig approvals and payout
- risk-score-driven premium increase behavior

## Economic Security Notes

- Coverage utilization caps by tier reduce insolvency probability under correlated claims.
- Withdrawal queue delays mitigate bank-run style instant exits.
- Fraud slashing creates direct economic cost for malicious claim submission.
- Premium per-share accounting prevents premium leakage and keeps payout incentives aligned with active stakers.

## Determinism and Test Status

Current Soroban crate test status after implementation:
- 44 passed
- 0 failed

This includes insurance-specific tests plus existing bridge/liquidity tests.

## Integration Notes

- `set_risk_score` is the integration point for external bridge health metrics.
- For production token movement, integrate asset contracts in payout and premium collection paths with strict auth and replay protections.
