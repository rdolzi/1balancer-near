use near_sdk::{near, env, AccountId};

#[near(contract_state)]
pub struct SolverRegistry {
    owner: AccountId,
}

#[near]
impl SolverRegistry {
    #[init]
    pub fn new(owner: AccountId) -> Self {
        Self { owner }
    }
}