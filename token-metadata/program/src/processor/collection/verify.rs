use crate::{
    assertions::{
        assert_keys_equal, assert_owned_by, assert_token_matches_owner_and_mint,
        metadata::assert_holding_amount,
    },
    error::MetadataError,
    instruction::{Context, Verify, VerifyArgs},
    state::{
        AuthorityRequest, AuthorityType, Metadata, Operation, Resizable, TokenDelegateRole,
        TokenMetadataAccount, TokenRecord, TokenStandard,
    },
};
use borsh::{maybestd::io::Error as BorshError, BorshDeserialize, BorshSerialize};
use mpl_utils::{assert_signer, token::TokenTransferParams};
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

fn verify_v1(program_id: &Pubkey, ctx: Context<Verify>, _args: VerifyArgs) -> ProgramResult {
    // Creator verification.
    if let Some(creator_info) = ctx.accounts.creator_info {
        assert_signer(creator_info)?;
        assert_owned_by(ctx.accounts.metadata_info, program_id)?;

        let mut metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;

        if let Some(creators) = &mut metadata.data.creators {
            let mut found = false;
            for creator in creators {
                if creator.address == *creator_info.key {
                    creator.verified = true;
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(MetadataError::CreatorNotFound.into());
            }
        } else {
            return Err(MetadataError::NoCreatorsPresentOnMetadata.into());
        }
        metadata.serialize(&mut *ctx.accounts.metadata_info.try_borrow_mut_data()?)?;

        Ok(())
    } else {
        Err(MetadataError::FeatureNotSupported.into())
    }
}
