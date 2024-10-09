//! Handler related to BNB-Smart-chain

use crate::{
    handler::register::EvmHandler,
    interpreter::Gas,
    primitives::{
        address, db::Database, spec_to_generic, Address, EVMError, Env, ExecutionResult,
        InvalidTransaction, ResultAndState, Spec, SpecId, CANCUN, U256,
    },
    Context, FrameResult,
};
use revm_interpreter::{gas, SuccessOrHalt};
use std::sync::Arc;

pub const SYSTEM_ADDRESS: Address = address!("fffffffffffffffffffffffffffffffffffffffe");

pub fn bsc_handle_register<DB: Database, EXT>(handler: &mut EvmHandler<'_, EXT, DB>) {
    spec_to_generic!(handler.cfg.spec_id, {
        handler.validation.initial_tx_gas = Arc::new(validate_initial_tx_gas::<SPEC, DB>);
        handler.post_execution.reward_beneficiary =
            Arc::new(collect_system_reward::<SPEC, EXT, DB>);
        handler.post_execution.output = Arc::new(output::<EXT, DB>);
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
    let authorization_list_num = env
        .tx
        .authorization_list
        .as_ref()
        .map(|l| l.len() as u64)
        .unwrap_or_default();

    let initial_gas_spend = gas::validate_initial_tx_gas(
        SPEC::SPEC_ID,
        input,
        is_create,
        access_list,
        authorization_list_num,
    );

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
    if context
        .evm
        .env
        .tx
        .bsc
        .is_system_transaction
        .unwrap_or(false)
    {
        return Ok(());
    }

    let effective_gas_price = context.evm.env.effective_gas_price();
    let mut tx_fee = U256::from(gas.spent() - gas.refunded() as u64) * effective_gas_price;

    // EIP-4844
    if SPEC::enabled(CANCUN) {
        let data_fee = &context
            .evm
            .inner
            .env
            .calc_data_fee()
            .expect("already checked");
        tx_fee = tx_fee.saturating_add(*data_fee);
    }

    let system_account = context
        .evm
        .inner
        .journaled_state
        .load_account(SYSTEM_ADDRESS, &mut context.evm.inner.db)?;

    system_account.data.mark_touch();
    system_account.data.info.balance = system_account.data.info.balance.saturating_add(tx_fee);

    Ok(())
}

/// Main return handle, returns the output of the transaction.
#[inline]
pub fn output<EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    result: FrameResult,
) -> Result<ResultAndState, EVMError<DB::Error>> {
    context.evm.take_error()?;
    // used gas with refund calculated.
    let gas_refunded = if context
        .evm
        .env
        .tx
        .bsc
        .is_system_transaction
        .unwrap_or(false)
    {
        0
    } else {
        result.gas().refunded() as u64
    };
    let final_gas_used = result.gas().spent() - gas_refunded;
    let output = result.output();
    let instruction_result = result.into_interpreter_result();

    // reset journal and return present state.
    let (state, logs) = context.evm.journaled_state.finalize();

    let result = match instruction_result.result.into() {
        SuccessOrHalt::Success(reason) => ExecutionResult::Success {
            reason,
            gas_used: final_gas_used,
            gas_refunded,
            logs,
            output,
        },
        SuccessOrHalt::Revert => ExecutionResult::Revert {
            gas_used: final_gas_used,
            output: output.into_data(),
        },
        SuccessOrHalt::Halt(reason) => ExecutionResult::Halt {
            reason,
            gas_used: final_gas_used,
        },
        // Only two internal return flags.
        flag @ (SuccessOrHalt::FatalExternalError | SuccessOrHalt::Internal(_)) => {
            panic!(
                "Encountered unexpected internal return flag: {:?} with instruction result: {:?}",
                flag, instruction_result
            )
        }
    };

    Ok(ResultAndState { result, state })
}
