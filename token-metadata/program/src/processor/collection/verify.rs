use crate::{
    error::MetadataError,
    instruction::{Context, Verify, VerifyArgs},
};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

pub fn verify<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: VerifyArgs,
) -> ProgramResult {
    let context = Verify::to_context(accounts)?;

    match args {
        VerifyArgs::V1 { .. } => verify_v1(program_id, context, args),
    }
}

fn verify_v1(_program_id: &Pubkey, _ctx: Context<Verify>, _args: VerifyArgs) -> ProgramResult {
    Err(MetadataError::FeatureNotSupported.into())
}
