//! Handler related to BNB-Smart-chain

use crate::{
    handler::register::EvmHandler,
    interpreter::Gas,
    primitives::{
        address, db::Database, spec_to_generic, Address, EVMError, Env, InvalidTransaction, Spec,
        SpecId, U256,
    },
    Context,
};
use revm_interpreter::gas;
use std::sync::Arc;

pub const SYSTEM_ADDRESS: Address = address!("fffffffffffffffffffffffffffffffffffffffe");

pub fn bsc_handle_register<DB: Database, EXT>(handler: &mut EvmHandler<'_, EXT, DB>) {
    spec_to_generic!(handler.cfg.spec_id, {
        handler.validation.initial_tx_gas = Arc::new(validate_initial_tx_gas::<SPEC, DB>);
        handler.post_execution.reward_beneficiary =
            Arc::new(collect_system_reward::<SPEC, EXT, DB>);
    });
}

/// Validate initial transaction gas.
pub fn validate_initial_tx_gas<SPEC: Spec, DB: Database>(
    env: &Env,
) -> Result<u64, EVMError<DB::Error>> {
    // no initial gas for system transactions
    if env.tx.bsc.is_system_transaction.unwrap_or(false) {
        return Ok(0);
    }

    let input = &env.tx.data;
    let is_create = env.tx.transact_to.is_create();
    let access_list = &env.tx.access_list;

    let initial_gas_spend = gas::validate_initial_tx_gas::<SPEC>(input, is_create, access_list);

    // Additional check to see if limit is big enough to cover initial gas.
    if initial_gas_spend > env.tx.gas_limit {
        return Err(InvalidTransaction::CallGasCostMoreThanGasLimit.into());
    }
    Ok(initial_gas_spend)
}

/// Collect gas fee to system account.
#[inline]
pub fn collect_system_reward<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    gas: &Gas,
) -> Result<(), EVMError<DB::Error>> {
    let effective_gas_price = context.evm.env.effective_gas_price();

    let (system_account, _) = context
        .evm
        .inner
        .journaled_state
        .load_account(SYSTEM_ADDRESS, &mut context.evm.inner.db)?;

    system_account.mark_touch();
    system_account.info.balance = system_account
        .info
        .balance
        .saturating_add(effective_gas_price * U256::from(gas.spent() - gas.refunded() as u64));

    Ok(())
}
