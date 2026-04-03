
use borsh::{BorshDeserialize, BorshSerialize};
use num_derive::FromPrimitive;



#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, FromPrimitive)]
pub enum ProgramInstruction {
    /// Start root registry
    InitializeRoot,
    
    /// Create root registry
    RegisterRoot,

    /// Begin domain/name lifecycle
    BeginNameRegistration,

    /// Increase bid / price for a name
    IncreaseBid,

    /// Finalize name registration and settlement
    FinalizeName,

    /// Withdraw user funds or rewards
    Withdraw,

    /// Initialize a project under the protocol
    InitializeProject,

    /// Withdraw protocol/admin funds
    WithdrawAdmin,

    /// Init usr record account
    InitUsr,
}

