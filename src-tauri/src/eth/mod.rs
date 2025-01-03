pub mod mutators;
pub mod queries;

use alloy::sol;

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    CHAT,
    "../contracts/out/Chat.sol/Chat.json"
);
