use libpulse_binding::context::subscribe::Operation;

use crate::backend::types::{InternalCommandSender, InternalRefresh};

pub(crate) async fn handle_change(operation: Operation, command_tx: &InternalCommandSender) {
    if operation == Operation::Changed {
        let _ = command_tx.send(InternalRefresh::ServerInfo);
    }
}
