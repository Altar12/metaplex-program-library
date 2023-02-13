use crate::{
    assertions::{
        assert_owned_by,
        collection::{assert_collection_verify_is_valid, assert_has_collection_authority},
    },
    error::MetadataError,
    instruction::{Context, Verify, VerifyArgs},
    state::{Metadata, TokenMetadataAccount},
    utils::{clean_write_metadata, increment_collection_size},
};
use borsh::BorshSerialize;
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

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
    if let Some(creator_info) = ctx.accounts.creator_info {
        // Creator verification.
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
        // Sized collection verification.
        let collection_authority_info = ctx
            .accounts
            .collection_authority_info
            .ok_or(MetadataError::MissingCollectionAuthority)?;

        assert_signer(collection_authority_info)?;

        assert_signer(ctx.accounts.payer_info)?;
        assert_owned_by(ctx.accounts.metadata_info, program_id)?;

        let collection_metadata_info = ctx
            .accounts
            .collection_metadata_info
            .ok_or(MetadataError::MissingCollectionMetadata)?;

        assert_owned_by(collection_metadata_info, program_id)?;

        let collection_mint_info = ctx
            .accounts
            .collection_mint_info
            .ok_or(MetadataError::MissingCollectionMint)?;

        assert_owned_by(collection_mint_info, &spl_token::id())?;

        let collection_master_edition_info = ctx
            .accounts
            .collection_master_edition_info
            .ok_or(MetadataError::MissingCollectionMasterEdition)?;

        assert_owned_by(collection_master_edition_info, program_id)?;

        let mut metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;
        let mut collection_metadata = Metadata::from_account_info(collection_metadata_info)?;

        // Don't verify already verified items, otherwise we end up with invalid size data.
        if let Some(collection) = &metadata.collection {
            if collection.verified {
                return Err(MetadataError::AlreadyVerified.into());
            }
        }

        assert_collection_verify_is_valid(
            &metadata.collection,
            &collection_metadata,
            collection_mint_info,
            collection_master_edition_info,
        )?;

        assert_has_collection_authority(
            collection_authority_info,
            &collection_metadata,
            collection_mint_info.key,
            ctx.accounts.collection_authority_record_info,
        )?;

        // If the NFT has unverified collection data, we set it to be verified and then update the collection
        // size on the Collection Parent.
        if let Some(collection) = &mut metadata.collection {
            msg!("Verifying sized collection item");
            increment_collection_size(&mut collection_metadata, collection_metadata_info)?;

            collection.verified = true;
            clean_write_metadata(&mut metadata, ctx.accounts.metadata_info)?;
        } else {
            return Err(MetadataError::CollectionNotFound.into());
        }
        Ok(())
    }
}
