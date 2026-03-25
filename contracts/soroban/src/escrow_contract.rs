use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol,
    Vec,
};

const BPS_DENOMINATOR: i128 = 10_000;
const DEFAULT_LOCK_SECS: u64 = 3_600;
const MAX_METADATA_LEN: u32 = 512;

const EVENT_ESCROW_CREATED: Symbol = symbol_short!("esc_cre");
const EVENT_ESCROW_CHALLENGED: Symbol = symbol_short!("esc_chl");
const EVENT_CHALLENGE_RESOLVED: Symbol = symbol_short!("esc_res");
const EVENT_ESCROW_RELEASED: Symbol = symbol_short!("esc_rel");
const EVENT_ESCROW_REFUNDED: Symbol = symbol_short!("esc_ref");
const EVENT_ESCROW_EXTENDED: Symbol = symbol_short!("esc_ext");
const EVENT_VERIFICATION_SYNCED: Symbol = symbol_short!("esc_vrf");
const EVENT_LOCK_CONFIG_SET: Symbol = symbol_short!("esc_lck");
const EVENT_FEES_COLLECTED: Symbol = symbol_short!("esc_fee");
const EVENT_EMERGENCY_RECOVERY: Symbol = symbol_short!("esc_emg");
const EVENT_BATCH_RELEASE: Symbol = symbol_short!("esc_bat");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum EscrowError {
    NotAuthorized = 1,
    InvalidAmount = 2,
    InvalidLockConfig = 3,
    EscrowNotFound = 4,
    LockNotElapsed = 5,
    AlreadyChallenged = 6,
    NoChallenge = 7,
    ChallengeWindowClosed = 8,
    InvalidEscrowState = 9,
    VerificationPending = 10,
    VerificationFailed = 11,
    AlreadyApproved = 12,
    InvalidDisputeDecision = 13,
    MetadataTooLarge = 14,
    EmergencyNotEnabled = 15,
    FeeConfigInvalid = 16,
    NothingToRelease = 17,
    NothingToRefund = 18,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EscrowStatus {
    Pending,
    Challenged,
    Released,
    Refunded,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowDeposit {
    pub escrow_id: u64,
    pub depositor: Address,
    pub recipient: Address,
    pub bridge_id: String,
    pub asset_type: String,
    pub amount: i128,
    pub fee_total: i128,
    pub released_amount: i128,
    pub lock_until: u64,
    pub status: EscrowStatus,
    pub metadata: String,
    pub verification_contract: Address,
    pub verification_ref: String,
    pub verified: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChallengeInfo {
    pub escrow_id: u64,
    pub challenger: Address,
    pub reason: String,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeResolution {
    pub escrow_id: u64,
    pub decision_release: bool,
    pub approvals: Vec<Address>,
    pub required_approvals: u32,
    pub resolved: bool,
}

#[contracttype]
pub enum DataKey {
    Admin,
    FeeCollector,
    FeeBps,
    EscrowSeq,
    Escrow(u64),
    Challenge(u64),
    Dispute(u64),
    LockPeriod(String, String),
    Approvers,
    ApprovalThreshold,
    EmergencyPause,
    AccruedFees,
}

#[contract]
pub struct TimeLockedEscrowContract;

#[contractimpl]
impl TimeLockedEscrowContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        fee_collector: Address,
        fee_bps: u32,
        approvers: Vec<Address>,
        approval_threshold: u32,
    ) -> Result<(), EscrowError> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(EscrowError::NotAuthorized);
        }
        if fee_bps > 2_000 {
            return Err(EscrowError::FeeConfigInvalid);
        }
        if approvers.is_empty() || approval_threshold == 0 || approval_threshold > approvers.len() {
            return Err(EscrowError::InvalidDisputeDecision);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::FeeCollector, &fee_collector);
        env.storage().instance().set(&DataKey::FeeBps, &fee_bps);
        env.storage().instance().set(&DataKey::Approvers, &approvers);
        env.storage().instance().set(&DataKey::ApprovalThreshold, &approval_threshold);
        env.storage().instance().set(&DataKey::EscrowSeq, &0u64);
        env.storage().instance().set(&DataKey::EmergencyPause, &false);
        env.storage().instance().set(&DataKey::AccruedFees, &0i128);
        Ok(())
    }

    pub fn set_lock_period(
        env: Env,
        admin: Address,
        bridge_id: String,
        asset_type: String,
        lock_secs: u64,
    ) -> Result<(), EscrowError> {
        require_admin(&env, &admin)?;
        if lock_secs == 0 {
            return Err(EscrowError::InvalidLockConfig);
        }

        env.storage()
            .instance()
            .set(&DataKey::LockPeriod(bridge_id.clone(), asset_type.clone()), &lock_secs);
        env.events()
            .publish((EVENT_LOCK_CONFIG_SET, admin), (bridge_id, asset_type, lock_secs));
        Ok(())
    }

    pub fn set_fee_config(env: Env, admin: Address, fee_collector: Address, fee_bps: u32) -> Result<(), EscrowError> {
        require_admin(&env, &admin)?;
        if fee_bps > 2_000 {
            return Err(EscrowError::FeeConfigInvalid);
        }
        env.storage().instance().set(&DataKey::FeeCollector, &fee_collector);
        env.storage().instance().set(&DataKey::FeeBps, &fee_bps);
        Ok(())
    }

    pub fn set_approvers(
        env: Env,
        admin: Address,
        approvers: Vec<Address>,
        approval_threshold: u32,
    ) -> Result<(), EscrowError> {
        require_admin(&env, &admin)?;
        if approvers.is_empty() || approval_threshold == 0 || approval_threshold > approvers.len() {
            return Err(EscrowError::InvalidDisputeDecision);
        }
        env.storage().instance().set(&DataKey::Approvers, &approvers);
        env.storage().instance().set(&DataKey::ApprovalThreshold, &approval_threshold);
        Ok(())
    }

    pub fn set_emergency_pause(env: Env, admin: Address, paused: bool) -> Result<(), EscrowError> {
        require_admin(&env, &admin)?;
        env.storage().instance().set(&DataKey::EmergencyPause, &paused);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_escrow(
        env: Env,
        depositor: Address,
        recipient: Address,
        bridge_id: String,
        asset_type: String,
        amount: i128,
        metadata: String,
        verification_contract: Address,
        verification_ref: String,
    ) -> Result<u64, EscrowError> {
        require_not_paused(&env)?;
        depositor.require_auth();
        if amount <= 0 {
            return Err(EscrowError::InvalidAmount);
        }
        if metadata.len() > MAX_METADATA_LEN {
            return Err(EscrowError::MetadataTooLarge);
        }

        let fee_bps: u32 = env.storage().instance().get(&DataKey::FeeBps).unwrap_or(0);
        let fee_total = amount
            .checked_mul(fee_bps as i128)
            .and_then(|v| v.checked_div(BPS_DENOMINATOR))
            .ok_or(EscrowError::InvalidAmount)?;

        let lock_period = env
            .storage()
            .instance()
            .get(&DataKey::LockPeriod(bridge_id.clone(), asset_type.clone()))
            .unwrap_or(DEFAULT_LOCK_SECS);
        let lock_until = env.ledger().timestamp().saturating_add(lock_period);

        let escrow_id = next_id(&env);
        let escrow = EscrowDeposit {
            escrow_id,
            depositor: depositor.clone(),
            recipient: recipient.clone(),
            bridge_id: bridge_id.clone(),
            asset_type: asset_type.clone(),
            amount,
            fee_total,
            released_amount: 0,
            lock_until,
            status: EscrowStatus::Pending,
            metadata,
            verification_contract,
            verification_ref,
            verified: false,
        };

        let accrued: i128 = env.storage().instance().get(&DataKey::AccruedFees).unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::AccruedFees, &(accrued + fee_total));
        env.storage().instance().set(&DataKey::Escrow(escrow_id), &escrow);

        env.events().publish(
            (EVENT_ESCROW_CREATED, depositor, recipient),
            (escrow_id, bridge_id, asset_type, amount, fee_total, lock_until),
        );
        Ok(escrow_id)
    }

    pub fn sync_verification(
        env: Env,
        verifier_contract: Address,
        escrow_id: u64,
        verified: bool,
    ) -> Result<(), EscrowError> {
        verifier_contract.require_auth();
        let mut escrow = get_escrow_mut(&env, escrow_id)?;
        if verifier_contract != escrow.verification_contract {
            return Err(EscrowError::NotAuthorized);
        }
        if matches!(escrow.status, EscrowStatus::Released | EscrowStatus::Refunded) {
            return Err(EscrowError::InvalidEscrowState);
        }
        escrow.verified = verified;
        env.storage().instance().set(&DataKey::Escrow(escrow_id), &escrow);
        env.events().publish((EVENT_VERIFICATION_SYNCED, verifier_contract), (escrow_id, verified));
        Ok(())
    }

    pub fn challenge_escrow(
        env: Env,
        challenger: Address,
        escrow_id: u64,
        reason: String,
    ) -> Result<(), EscrowError> {
        require_not_paused(&env)?;
        challenger.require_auth();

        let mut escrow = get_escrow_mut(&env, escrow_id)?;
        if !matches!(escrow.status, EscrowStatus::Pending) {
            return Err(EscrowError::InvalidEscrowState);
        }
        if env.ledger().timestamp() >= escrow.lock_until {
            return Err(EscrowError::ChallengeWindowClosed);
        }
        if env.storage().instance().has(&DataKey::Challenge(escrow_id)) {
            return Err(EscrowError::AlreadyChallenged);
        }

        escrow.status = EscrowStatus::Challenged;
        let challenge = ChallengeInfo {
            escrow_id,
            challenger: challenger.clone(),
            reason,
            timestamp: env.ledger().timestamp(),
        };
        env.storage().instance().set(&DataKey::Escrow(escrow_id), &escrow);
        env.storage()
            .instance()
            .set(&DataKey::Challenge(escrow_id), &challenge);

        env.events()
            .publish((EVENT_ESCROW_CHALLENGED, challenger), (escrow_id, challenge.timestamp));
        Ok(())
    }

    pub fn resolve_challenge(
        env: Env,
        approver: Address,
        escrow_id: u64,
        release_allowed: bool,
    ) -> Result<bool, EscrowError> {
        require_not_paused(&env)?;
        approver.require_auth();
        require_approver(&env, &approver)?;

        let mut escrow = get_escrow_mut(&env, escrow_id)?;
        if !matches!(escrow.status, EscrowStatus::Challenged) {
            return Err(EscrowError::NoChallenge);
        }

        let required: u32 = env.storage().instance().get(&DataKey::ApprovalThreshold).unwrap_or(1);
        let mut dispute: DisputeResolution = env
            .storage()
            .instance()
            .get(&DataKey::Dispute(escrow_id))
            .unwrap_or(DisputeResolution {
                escrow_id,
                decision_release: release_allowed,
                approvals: Vec::new(&env),
                required_approvals: required,
                resolved: false,
            });

        if dispute.resolved {
            return Ok(true);
        }
        if dispute.decision_release != release_allowed {
            return Err(EscrowError::InvalidDisputeDecision);
        }
        if contains_addr(&dispute.approvals, &approver) {
            return Err(EscrowError::AlreadyApproved);
        }

        dispute.approvals.push_back(approver.clone());
        if dispute.approvals.len() >= dispute.required_approvals {
            dispute.resolved = true;
            if dispute.decision_release {
                escrow.status = EscrowStatus::Pending;
            }
        }

        env.storage().instance().set(&DataKey::Dispute(escrow_id), &dispute);
        env.storage().instance().set(&DataKey::Escrow(escrow_id), &escrow);
        env.events().publish(
            (EVENT_CHALLENGE_RESOLVED, approver),
            (
                escrow_id,
                dispute.decision_release,
                dispute.approvals.len(),
                dispute.required_approvals,
                dispute.resolved,
            ),
        );

        Ok(dispute.resolved)
    }

    pub fn release_escrow(
        env: Env,
        caller: Address,
        escrow_id: u64,
        release_amount: i128,
    ) -> Result<i128, EscrowError> {
        Self::release_escrow_internal(env, caller, escrow_id, release_amount, true)
    }

    fn release_escrow_internal(
        env: Env,
        caller: Address,
        escrow_id: u64,
        release_amount: i128,
        enforce_auth: bool,
    ) -> Result<i128, EscrowError> {
        require_not_paused(&env)?;
        if enforce_auth {
            caller.require_auth();
        }

        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        let mut escrow = get_escrow_mut(&env, escrow_id)?;
        if caller != escrow.recipient && caller != admin {
            return Err(EscrowError::NotAuthorized);
        }
        if matches!(escrow.status, EscrowStatus::Refunded | EscrowStatus::Released) {
            return Err(EscrowError::InvalidEscrowState);
        }
        if matches!(escrow.status, EscrowStatus::Challenged) {
            return Err(EscrowError::NoChallenge);
        }
        if !escrow.verified {
            return Err(EscrowError::VerificationPending);
        }
        if env.ledger().timestamp() < escrow.lock_until {
            return Err(EscrowError::LockNotElapsed);
        }
        if release_amount <= 0 {
            return Err(EscrowError::InvalidAmount);
        }

        let releasable_total = escrow.amount.saturating_sub(escrow.fee_total);
        let remaining = releasable_total.saturating_sub(escrow.released_amount);
        if remaining <= 0 {
            return Err(EscrowError::NothingToRelease);
        }
        if release_amount > remaining {
            return Err(EscrowError::InvalidAmount);
        }

        escrow.released_amount = escrow.released_amount.saturating_add(release_amount);
        if escrow.released_amount >= releasable_total {
            escrow.status = EscrowStatus::Released;
        }
        env.storage().instance().set(&DataKey::Escrow(escrow_id), &escrow);
        env.events().publish(
            (EVENT_ESCROW_RELEASED, caller),
            (escrow_id, release_amount, escrow.released_amount),
        );

        Ok(release_amount)
    }

    pub fn batch_release(
        env: Env,
        caller: Address,
        escrow_ids: Vec<u64>,
        release_amount: i128,
    ) -> Result<Vec<i128>, EscrowError> {
        require_not_paused(&env)?;
        caller.require_auth();
        if escrow_ids.is_empty() {
            return Ok(Vec::new(&env));
        }

        let mut released = Vec::new(&env);
        let mut total: i128 = 0;
        for i in 0..escrow_ids.len() {
            let escrow_id = escrow_ids.get(i).unwrap();
            let amt = Self::release_escrow_internal(
                env.clone(),
                caller.clone(),
                escrow_id,
                release_amount,
                false,
            )?;
            released.push_back(amt);
            total = total.saturating_add(amt);
        }

        env.events()
            .publish((EVENT_BATCH_RELEASE, caller), (escrow_ids.len(), total));
        Ok(released)
    }

    pub fn refund_escrow(env: Env, caller: Address, escrow_id: u64) -> Result<i128, EscrowError> {
        require_not_paused(&env)?;
        caller.require_auth();

        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        let mut escrow = get_escrow_mut(&env, escrow_id)?;
        if caller != escrow.depositor && caller != admin {
            return Err(EscrowError::NotAuthorized);
        }
        if matches!(escrow.status, EscrowStatus::Released | EscrowStatus::Refunded) {
            return Err(EscrowError::InvalidEscrowState);
        }

        if matches!(escrow.status, EscrowStatus::Challenged) {
            let dispute: DisputeResolution = env
                .storage()
                .instance()
                .get(&DataKey::Dispute(escrow_id))
                .ok_or(EscrowError::NoChallenge)?;
            if !dispute.resolved || dispute.decision_release {
                return Err(EscrowError::InvalidEscrowState);
            }
        } else if escrow.verified {
            return Err(EscrowError::VerificationFailed);
        }

        let refund_amount = escrow
            .amount
            .saturating_sub(escrow.released_amount)
            .saturating_sub(escrow.fee_total);
        if refund_amount <= 0 {
            return Err(EscrowError::NothingToRefund);
        }

        escrow.status = EscrowStatus::Refunded;
        env.storage().instance().set(&DataKey::Escrow(escrow_id), &escrow);
        env.events()
            .publish((EVENT_ESCROW_REFUNDED, caller), (escrow_id, refund_amount));
        Ok(refund_amount)
    }

    pub fn extend_lock(
        env: Env,
        admin: Address,
        escrow_id: u64,
        extra_secs: u64,
    ) -> Result<u64, EscrowError> {
        require_admin(&env, &admin)?;
        if extra_secs == 0 {
            return Err(EscrowError::InvalidLockConfig);
        }

        let mut escrow = get_escrow_mut(&env, escrow_id)?;
        if matches!(escrow.status, EscrowStatus::Released | EscrowStatus::Refunded) {
            return Err(EscrowError::InvalidEscrowState);
        }

        escrow.lock_until = escrow.lock_until.saturating_add(extra_secs);
        env.storage().instance().set(&DataKey::Escrow(escrow_id), &escrow);
        env.events()
            .publish((EVENT_ESCROW_EXTENDED, admin), (escrow_id, escrow.lock_until));
        Ok(escrow.lock_until)
    }

    pub fn emergency_recover(
        env: Env,
        admin: Address,
        escrow_id: u64,
        recipient: Address,
        amount: i128,
    ) -> Result<(), EscrowError> {
        require_admin(&env, &admin)?;
        let paused: bool = env.storage().instance().get(&DataKey::EmergencyPause).unwrap_or(false);
        if !paused {
            return Err(EscrowError::EmergencyNotEnabled);
        }

        let mut escrow = get_escrow_mut(&env, escrow_id)?;
        if amount <= 0 || amount > escrow.amount.saturating_sub(escrow.released_amount) {
            return Err(EscrowError::InvalidAmount);
        }

        escrow.released_amount = escrow.released_amount.saturating_add(amount);
        if escrow.released_amount >= escrow.amount.saturating_sub(escrow.fee_total) {
            escrow.status = EscrowStatus::Released;
        }
        env.storage().instance().set(&DataKey::Escrow(escrow_id), &escrow);
        env.events().publish(
            (EVENT_EMERGENCY_RECOVERY, admin),
            (escrow_id, recipient, amount),
        );
        Ok(())
    }

    pub fn collect_fees(env: Env, collector: Address, amount: i128) -> Result<i128, EscrowError> {
        collector.require_auth();
        let configured: Address = env.storage().instance().get(&DataKey::FeeCollector).unwrap();
        if collector != configured {
            return Err(EscrowError::NotAuthorized);
        }
        if amount <= 0 {
            return Err(EscrowError::InvalidAmount);
        }

        let accrued: i128 = env.storage().instance().get(&DataKey::AccruedFees).unwrap_or(0);
        if amount > accrued {
            return Err(EscrowError::InvalidAmount);
        }

        let remaining = accrued - amount;
        env.storage().instance().set(&DataKey::AccruedFees, &remaining);
        env.events()
            .publish((EVENT_FEES_COLLECTED, collector), (amount, remaining));
        Ok(amount)
    }

    pub fn get_escrow(env: Env, escrow_id: u64) -> Option<EscrowDeposit> {
        env.storage().instance().get(&DataKey::Escrow(escrow_id))
    }

    pub fn get_challenge(env: Env, escrow_id: u64) -> Option<ChallengeInfo> {
        env.storage().instance().get(&DataKey::Challenge(escrow_id))
    }

    pub fn get_dispute(env: Env, escrow_id: u64) -> Option<DisputeResolution> {
        env.storage().instance().get(&DataKey::Dispute(escrow_id))
    }

    pub fn get_accrued_fees(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::AccruedFees).unwrap_or(0)
    }
}

fn next_id(env: &Env) -> u64 {
    let current: u64 = env.storage().instance().get(&DataKey::EscrowSeq).unwrap_or(0);
    let next = current.saturating_add(1);
    env.storage().instance().set(&DataKey::EscrowSeq, &next);
    current
}

fn get_escrow_mut(env: &Env, escrow_id: u64) -> Result<EscrowDeposit, EscrowError> {
    env.storage()
        .instance()
        .get(&DataKey::Escrow(escrow_id))
        .ok_or(EscrowError::EscrowNotFound)
}

fn require_admin(env: &Env, admin: &Address) -> Result<(), EscrowError> {
    admin.require_auth();
    let stored: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
    if &stored != admin {
        return Err(EscrowError::NotAuthorized);
    }
    Ok(())
}

fn require_not_paused(env: &Env) -> Result<(), EscrowError> {
    let paused: bool = env.storage().instance().get(&DataKey::EmergencyPause).unwrap_or(false);
    if paused {
        return Err(EscrowError::EmergencyNotEnabled);
    }
    Ok(())
}

fn require_approver(env: &Env, approver: &Address) -> Result<(), EscrowError> {
    let approvers: Vec<Address> = env.storage().instance().get(&DataKey::Approvers).unwrap();
    if !contains_addr(&approvers, approver) {
        return Err(EscrowError::NotAuthorized);
    }
    Ok(())
}

fn contains_addr(list: &Vec<Address>, who: &Address) -> bool {
    for i in 0..list.len() {
        if list.get(i).unwrap() == *who {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};

    #[contract]
    pub struct MockBridgeVerifier;

    #[contractimpl]
    impl MockBridgeVerifier {
        pub fn set_verified(env: Env, admin: Address, reference: String, verified: bool) {
            admin.require_auth();
            env.storage().instance().set(&(symbol_short!("ref"), reference), &verified);
        }

        pub fn is_verified(env: Env, reference: String) -> bool {
            env.storage()
                .instance()
                .get(&(symbol_short!("ref"), reference))
                .unwrap_or(false)
        }
    }

    fn setup() -> (
        Env,
        TimeLockedEscrowContractClient<'static>,
        Address,
        Address,
        Address,
        Address,
    ) {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_000_000);

        let contract_id = env.register_contract(None, TimeLockedEscrowContract);
        let client = TimeLockedEscrowContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let fee_collector = Address::generate(&env);
        let approver_1 = Address::generate(&env);
        let approver_2 = Address::generate(&env);

        let mut approvers = Vec::new(&env);
        approvers.push_back(approver_1.clone());
        approvers.push_back(approver_2.clone());

        client.initialize(&admin, &fee_collector, &100u32, &approvers, &2u32);

        (env, client, admin, fee_collector, approver_1, approver_2)
    }

    fn verifier(env: &Env) -> (Address, MockBridgeVerifierClient<'static>, Address) {
        let verifier_admin = Address::generate(env);
        let verifier_id = env.register_contract(None, MockBridgeVerifier);
        let verifier_client = MockBridgeVerifierClient::new(env, &verifier_id);
        (verifier_id, verifier_client, verifier_admin)
    }

    #[test]
    fn create_and_release_after_lock_and_verification() {
        let (env, client, admin, _fee_collector, _a1, _a2) = setup();
        let (verifier_id, verifier_client, verifier_admin) = verifier(&env);
        let depositor = Address::generate(&env);
        let recipient = Address::generate(&env);

        let bridge = String::from_str(&env, "stellar-eth");
        let asset = String::from_str(&env, "USDC");
        client.set_lock_period(&admin, &bridge, &asset, &120u64);

        let escrow_id = client
            .create_escrow(
                &depositor,
                &recipient,
                &bridge,
                &asset,
                &100_000i128,
                &String::from_str(&env, "tx:abc"),
                &verifier_id,
                &String::from_str(&env, "proof:1"),
            );

        let early = client.try_release_escrow(&recipient, &escrow_id, &99_000i128);
        assert!(early.is_err());

        verifier_client.set_verified(
            &verifier_admin,
            &String::from_str(&env, "proof:1"),
            &true,
        );
        client
            .sync_verification(&verifier_id, &escrow_id, &verifier_client.is_verified(&String::from_str(&env, "proof:1")));

        env.ledger().set_timestamp(1_000_130);
        let released = client.release_escrow(&recipient, &escrow_id, &50_000i128);
        assert_eq!(released, 50_000);

        let escrow = client.get_escrow(&escrow_id).unwrap();
        assert_eq!(escrow.released_amount, 50_000);
    }

    #[test]
    fn challenge_then_multisig_resolve_release_path() {
        let (env, client, _admin, _fee_collector, approver_1, approver_2) = setup();
        let (verifier_id, verifier_client, verifier_admin) = verifier(&env);
        let depositor = Address::generate(&env);
        let recipient = Address::generate(&env);
        let challenger = Address::generate(&env);

        let escrow_id = client
            .create_escrow(
                &depositor,
                &recipient,
                &String::from_str(&env, "bridge-a"),
                &String::from_str(&env, "XLM"),
                &50_000,
                &String::from_str(&env, "meta"),
                &verifier_id,
                &String::from_str(&env, "proof:2"),
            );

        client.challenge_escrow(&challenger, &escrow_id, &String::from_str(&env, "suspicious"));

        let r1 = client.resolve_challenge(&approver_1, &escrow_id, &true);
        assert!(!r1);
        let r2 = client.resolve_challenge(&approver_2, &escrow_id, &true);
        assert!(r2);

        verifier_client.set_verified(&verifier_admin, &String::from_str(&env, "proof:2"), &true);
        client.sync_verification(&verifier_id, &escrow_id, &true);

        env.ledger().set_timestamp(1_004_000);
        let released = client.release_escrow(&recipient, &escrow_id, &49_500);
        assert_eq!(released, 49_500);
    }

    #[test]
    fn dispute_reject_then_refund() {
        let (_env, client, _admin, _fee_collector, approver_1, approver_2) = setup();
        let (verifier_id, _verifier_client, _verifier_admin) = verifier(&_env);
        let depositor = Address::generate(&_env);
        let recipient = Address::generate(&_env);
        let challenger = Address::generate(&_env);

        let escrow_id = client
            .create_escrow(
                &depositor,
                &recipient,
                &String::from_str(&_env, "bridge-r"),
                &String::from_str(&_env, "USDT"),
                &10_000,
                &String::from_str(&_env, "meta"),
                &verifier_id,
                &String::from_str(&_env, "proof:3"),
            );
        client.challenge_escrow(&challenger, &escrow_id, &String::from_str(&_env, "bad"));

        client.resolve_challenge(&approver_1, &escrow_id, &false);
        client.resolve_challenge(&approver_2, &escrow_id, &false);

        let refunded = client.refund_escrow(&depositor, &escrow_id);
        assert_eq!(refunded, 9_900);
    }

    #[test]
    fn batch_release_and_fee_collection() {
        let (env, client, admin, fee_collector, _a1, _a2) = setup();
        let (verifier_id, _verifier_client, _verifier_admin) = verifier(&env);
        let depositor = Address::generate(&env);
        let recipient = Address::generate(&env);

        client.set_lock_period(
                &admin,
                &String::from_str(&env, "bridge-b"),
                &String::from_str(&env, "EURC"),
                &1,
            );

        let e1 = client
            .create_escrow(
                &depositor,
                &recipient,
                &String::from_str(&env, "bridge-b"),
                &String::from_str(&env, "EURC"),
                &1_000,
                &String::from_str(&env, "m1"),
                &verifier_id,
                &String::from_str(&env, "proof:4"),
            );
        let e2 = client
            .create_escrow(
                &depositor,
                &recipient,
                &String::from_str(&env, "bridge-b"),
                &String::from_str(&env, "EURC"),
                &2_000,
                &String::from_str(&env, "m2"),
                &verifier_id,
                &String::from_str(&env, "proof:5"),
            );

        client.sync_verification(&verifier_id, &e1, &true);
        client.sync_verification(&verifier_id, &e2, &true);
        env.ledger().set_timestamp(1_000_100);

        let mut ids = Vec::new(&env);
        ids.push_back(e1);
        ids.push_back(e2);
        let batch = client.batch_release(&recipient, &ids, &500);
        assert_eq!(batch.len(), 2);
        assert_eq!(client.get_accrued_fees(), 30);

        let collected = client.collect_fees(&fee_collector, &10);
        assert_eq!(collected, 10);
        assert_eq!(client.get_accrued_fees(), 20);
    }

    #[test]
    fn emergency_recovery_requires_pause() {
        let (env, client, admin, _fee_collector, _a1, _a2) = setup();
        let (verifier_id, _verifier_client, _verifier_admin) = verifier(&env);
        let depositor = Address::generate(&env);
        let recipient = Address::generate(&env);

        let escrow_id = client
            .create_escrow(
                &depositor,
                &recipient,
                &String::from_str(&env, "bridge-x"),
                &String::from_str(&env, "XLM"),
                &8_000,
                &String::from_str(&env, "meta"),
                &verifier_id,
                &String::from_str(&env, "proof:6"),
            );

        assert!(client
            .try_emergency_recover(&admin, &escrow_id, &recipient, &100)
            .is_err());

        client.set_emergency_pause(&admin, &true);
        client.emergency_recover(&admin, &escrow_id, &recipient, &500);

        let esc = client.get_escrow(&escrow_id).unwrap();
        assert_eq!(esc.released_amount, 500);
    }
}
