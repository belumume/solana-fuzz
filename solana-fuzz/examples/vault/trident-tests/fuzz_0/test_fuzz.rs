use fuzz_accounts::*;
use trident_fuzz::fuzzing::*;
mod fuzz_accounts;
mod types;
use types::vault::*;
use types::Vault;

#[derive(FuzzTestMethods)]
struct FuzzTest {
    trident: Trident,
    fuzz_accounts: AccountAddresses,
}

#[flow_executor]
impl FuzzTest {
    fn new() -> Self {
        Self {
            trident: Trident::default(),
            fuzz_accounts: AccountAddresses::default(),
        }
    }

    #[init]
    fn start(&mut self) {}

    /// One self-contained scenario per iteration: spin up a fresh vault, give it
    /// a random balance with `deposit` (range 0..1_000_000), then `withdraw` a
    /// random amount (range 0..2_000_000). The withdraw range reaches well past
    /// the deposited balance, so it frequently exceeds it, which is exactly the
    /// case the planted `wrapping_sub` bug breaks.
    #[flow]
    fn deposit_then_withdraw(&mut self) {
        let authority = self.fuzz_accounts.authority.insert(&mut self.trident, None);
        let vault = self.fuzz_accounts.vault.insert(
            &mut self.trident,
            Some(PdaSeeds {
                seeds: &[b"vault".as_ref(), authority.as_ref()],
                program_id: program_id(),
            }),
        );
        self.trident.airdrop(&authority, 10 * LAMPORTS_PER_SOL);

        let init_ix = InitializeInstruction::data(InitializeInstructionData::new())
            .accounts(InitializeInstructionAccounts::new(vault, authority))
            .instruction();
        if !self.trident.process_transaction(&[init_ix], Some("initialize")).is_success() {
            return;
        }

        let deposit_amount = self.trident.random_from_range(0..1_000_000u64);
        let dep_ix = DepositInstruction::data(DepositInstructionData::new(deposit_amount))
            .accounts(DepositInstructionAccounts::new(vault, authority))
            .instruction();
        self.trident.process_transaction(&[dep_ix], Some("deposit"));

        let before = match self.trident.get_account_with_type::<Vault>(&vault, 8) {
            Some(v) => v,
            None => return,
        };

        let withdraw_amount = self.trident.random_from_range(0..2_000_000u64);
        let w_ix = WithdrawInstruction::data(WithdrawInstructionData::new(withdraw_amount))
            .accounts(WithdrawInstructionAccounts::new(vault, authority))
            .instruction();
        let res = self.trident.process_transaction(&[w_ix], Some("withdraw"));

        if res.is_success() {
            if let Some(after) = self.trident.get_account_with_type::<Vault>(&vault, 8) {
                self.withdraw_invariant(&before, &after, withdraw_amount);
            }
        }
    }

    /// INVARIANT: a successful withdraw must never increase the vault balance.
    /// The planted `wrapping_sub` bug violates this whenever amount > balance.
    fn withdraw_invariant(&self, before: &Vault, after: &Vault, amount: u64) {
        assert!(
            after.balance <= before.balance,
            "INVARIANT VIOLATED: withdraw of {} increased balance {} -> {} (underflow)",
            amount, before.balance, after.balance
        );
    }

    #[end]
    fn end(&mut self) {}
}

fn main() {
    FuzzTest::fuzz(1000, 100);
}
