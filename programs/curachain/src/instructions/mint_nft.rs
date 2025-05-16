use anchor_spl::{
    metadata::{create_master_edition_v3, create_metadata_accounts_v3, 
        mpl_token_metadata::{ types::{Collection, DataV2}}, 
        update_metadata_accounts_v2, verify_sized_collection_item, VerifySizedCollectionItem, 
        CreateMasterEditionV3, CreateMetadataAccountsV3, MetadataAccount, UpdateMetadataAccountsV2}, 
    token_interface::{mint_to, MintTo}
    };

use anchor_lang::prelude::*;

use crate::states::{contexts::*, CuraChainError};


 //    ---------------------  NFT MINTING LOGIC AND METADATA UPDATES    ------------------------------- //
    // Let's Have The Logic To Mint The Recognition NFT to donor For First Time Case Donation
   

pub fn nft_mint(ctx: Context<MintNFT>, case_id: String, nft_uri: String) -> Result<()> {

    let donor_info = &mut ctx.accounts.donor_account;
     let case_id_bytes = {
        let bytes = case_id.as_bytes();
        let mut arr = [0u8;8];
        arr.copy_from_slice(bytes);
        arr
    };

    let seeds = &[
            b"multisig",
            b"escrow-authority".as_ref(),
            &[ctx.accounts.multisig.multisig_bump]
        ];
    let multisig_seeds = &[&seeds[..]];
    let first_time_donation = !donor_info.donated_cases.contains(&case_id_bytes);

    let donor_meta = ctx.accounts.donor_nft_metadata.to_account_info();
    let has_metadata = donor_meta.lamports() > 0 && donor_meta.data_len() > 0;

    if first_time_donation {
        donor_info.donated_cases.push(case_id_bytes);
    }
    // If First Time Case Donation, Mint Recognition NFT
    
    if !has_metadata {
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        

        let accounts = MintTo {
            mint: ctx.accounts.donor_nft_mint.to_account_info(),
            to: ctx.accounts.donor_nft_account.to_account_info(),
            authority: ctx.accounts.multisig.to_account_info()
        };

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, accounts, multisig_seeds);
        mint_to(cpi_ctx, 1)?;

        // Create Metadata Account For First-time Donations
        let data_v2 = DataV2 {
            name: format!("Recognition NFT -- {}", case_id),
            symbol: "CURA".to_string(),
            uri: nft_uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: Some(Collection {
                verified: false,// 
                key: ctx.accounts.parent_recognition_collection_nft.key(),
            }),
            uses: None
        };
        let cpi_metadata_program = ctx.accounts.metadata_program.to_account_info();
        let cpi_accounts = CreateMetadataAccountsV3 {
            metadata: ctx.accounts.donor_nft_metadata.to_account_info(),
            mint: ctx.accounts.donor_nft_mint.to_account_info(),
            mint_authority: ctx.accounts.multisig.to_account_info(),
            payer: ctx.accounts.donor.to_account_info(),
            update_authority: ctx.accounts.multisig.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(cpi_metadata_program.clone(), cpi_accounts, multisig_seeds);
        // Let's call the Actual Function To Create The Metadata Account
        // NB: Already Set Collection Details to None in the anchor Constraints on the metadata_account. 
        create_metadata_accounts_v3(cpi_ctx, data_v2, true, true, None)?;

        // Create Master Edition Account For First-Time Donations
        let edition_cpi_accounts = CreateMasterEditionV3 {
            edition: ctx.accounts.master_edition.to_account_info(),
            mint: ctx.accounts.donor_nft_mint.to_account_info(),
            update_authority: ctx.accounts.multisig.to_account_info(),
            mint_authority: ctx.accounts.multisig.to_account_info(),
            payer: ctx.accounts.donor.to_account_info(),
            metadata: ctx.accounts.donor_nft_metadata.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };
        let cpi_context = CpiContext::new_with_signer(cpi_metadata_program.clone(), edition_cpi_accounts, multisig_seeds);
        // Let's Call The Actual Function To Create The Edition Account
        create_master_edition_v3(cpi_context, Some(1))?;

        // We Need To Verify The Collection
        /*let verify_cpi_accounts = VerifyCollection{
            payer: ctx.accounts.donor.to_account_info(),
            collection_metadata: ctx.accounts.parent_collection_nft_metadata.to_account_info(),
            metadata: ctx.accounts.donor_nft_metadata.to_account_info(),
            //collection_authority: ctx.accounts.multisig.to_account_info(),
            collection_authority: ctx.accounts.admin.to_account_info(),
            collection_mint: ctx.accounts.parent_recognition_collection_nft.to_account_info(),
            collection_master_edition: ctx.accounts.parent_collection_master_edition.to_account_info(),
        };*/
        let verify_sized_cpi_accounts = VerifySizedCollectionItem{
            payer: ctx.accounts.donor.to_account_info(),
            collection_metadata: ctx.accounts.parent_collection_nft_metadata.to_account_info(),
            metadata: ctx.accounts.donor_nft_metadata.to_account_info(),
            collection_authority: ctx.accounts.admin.to_account_info(),
            collection_mint: ctx.accounts.parent_recognition_collection_nft.to_account_info(),
            collection_master_edition: ctx.accounts.parent_collection_master_edition.to_account_info(),
        };
        let verify_cpi_ctx = CpiContext::new_with_signer(cpi_metadata_program, verify_sized_cpi_accounts, multisig_seeds);
        // Call The Actual Verify instruction
/* 
        let (authority_record_pda, authority_record_bump)= Pubkey::find_program_address(
            &[
                b"metadata",
                ctx.accounts.metadata_program.key().as_ref(),
                ctx.accounts.parent_recognition_collection_nft.key().as_ref(),
                b"collection_authority",
                ctx.accounts.admin.key().as_ref()
            ],
            &mpl_token_metadata::ID
        );*/
        //verify_collection(verify_cpi_ctx, None)?;
        verify_sized_collection_item(verify_cpi_ctx, None)?;
        
    } else {
        // Making Further Contribution To Same Case, Just Update Metadata Account
        let donor_meta = ctx.accounts.donor_nft_metadata.to_account_info();
        let mut prev_metadata = &donor_meta.data.borrow()[..];
        //let meta_data = Metadata::from_account_info(&prev_metadata)?;
        let existing_meta = MetadataAccount::try_deserialize_unchecked( &mut prev_metadata)
            .map_err(|_| CuraChainError::InvalidMetadata)?;
        //let existing_meta = MetadataAccount::from_account

        let data_v2 = DataV2 {
            name: existing_meta.name.clone(),
            symbol: existing_meta.symbol.clone(),
            uri: nft_uri,
            seller_fee_basis_points: existing_meta.seller_fee_basis_points.clone(),

            creators: existing_meta.creators.clone(),
            collection: existing_meta.collection.clone(),
            uses: existing_meta.uses.clone()
        };
        
        let metadata_program = ctx.accounts.metadata_program.to_account_info();
        let update_cpi_accounts = UpdateMetadataAccountsV2 {
            metadata: ctx.accounts.donor_nft_metadata.to_account_info(),
            update_authority: ctx.accounts.multisig.to_account_info()
        };
        let update_cpi_ctx = CpiContext::new_with_signer(metadata_program, update_cpi_accounts, multisig_seeds);
        // Let's Call The Actual Function To Update The Metadata Account
        update_metadata_accounts_v2(
            update_cpi_ctx,
            None,
            Some(data_v2),
            None,
            Some(true)
        )?;
    }
    Ok(())
}