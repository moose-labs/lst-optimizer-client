use controller_lib::calculator::typedefs::SControllerError;
use lst_optimizer_utils::logger::error;
use rust_decimal::prelude::FromPrimitive;
use solana_client::client_error::{ClientError, ClientErrorKind};
use solana_sdk::transaction::TransactionError;

pub fn unhandled_error<E>(e: E)
where
    E: std::fmt::Display,
{
    error!("Unhandled error: {}", e);
}

pub fn handle_transaction_error(e: &TransactionError) {
    match e {
        solana_sdk::transaction::TransactionError::InstructionError(ix_index, err) => match err {
            solana_sdk::instruction::InstructionError::Custom(n) => {
                let serr = SControllerError::from_u32(*n);
                if let Some(e) = serr {
                    error!("Instruction {} error: {}", ix_index, e);
                } else {
                    unhandled_error(e);
                }
            }
            _ => unhandled_error(e),
        },
        _ => unhandled_error(e),
    }
}

pub fn handle_rpc_client_error(e: &ClientError) {
    match e.kind() {
        ClientErrorKind::RpcError(e) => match e {
            solana_client::rpc_request::RpcError::RpcResponseError {
                code,
                message,
                data,
            } => {
                match data {
                    solana_client::rpc_request::RpcResponseErrorData::SendTransactionPreflightFailure(
                        e,
                    ) => {
                        if let Some(e) = &e.err {
                            handle_transaction_error(e);
                        }else{
                            error!("RPC response error: code: {}, message: {}", code, message);
                        }
                    }
                    _ =>  unhandled_error(e)
                }
            }
            _ => unhandled_error(e),
        },
        _ => unhandled_error(e)
    }
}

pub fn handle_error(e: anyhow::Error) {
    if let Some(err) = e.downcast_ref::<ClientError>() {
        handle_rpc_client_error(err);
    } else {
        error!("Unhandled error: {}", e);
    }
}
