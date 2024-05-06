//! Handler related to BNB-Smart-chain

use crate::{
    handler::register::EvmHandler,
    interpreter::Gas,
    primitives::{address, db::Database, spec_to_generic, Address, EVMError, Spec, SpecId, U256},
    Context,
};
use std::sync::Arc;

pub const SYSTEM_ADDRESS: Address = address!("fffffffffffffffffffffffffffffffffffffffe");

pub fn bsc_handle_register<DB: Database, EXT>(handler: &mut EvmHandler<'_, EXT, DB>) {
    spec_to_generic!(handler.cfg.spec_id, {
        handler.post_execution.reward_beneficiary =
            Arc::new(collect_system_reward::<SPEC, EXT, DB>);
    });
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

#[inline]
pub fn reimburse_caller<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    gas: &Gas,
) -> Result<(), EVMError<DB::Error>> {
    if context.evm.env.tx.bsc.is_system_transaction.unwrap_or(false) {
        return Ok(());
    }

    let caller = context.evm.env.tx.caller;
    let effective_gas_price = context.evm.env.effective_gas_price();

    // return balance of not spend gas.
    let (caller_account, _) =
        context.evm.inner.journaled_state.load_account(caller, &mut context.evm.inner.db)?;

    caller_account.info.balance = caller_account
        .info
        .balance
        .saturating_add(effective_gas_price * U256::from(gas.remaining() + gas.refunded() as u64));

    Ok(())
}
