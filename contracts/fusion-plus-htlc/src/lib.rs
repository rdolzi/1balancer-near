use near_sdk::{near, env, AccountId, Balance, Promise};

#[near(contract_state)]
pub struct FusionPlusContract {
    owner: AccountId,
}

#[near]
impl FusionPlusContract {
    #[init]
    pub fn new(owner: AccountId) -> Self {
        Self { owner }
    }
}