use anchor_lang::prelude::*;

use anchor_spl::
    metadata::{create_master_edition_v3, create_metadata_accounts_v3, mpl_token_metadata::types::{ CollectionDetails, DataV2},
     CreateMasterEditionV3, CreateMetadataAccountsV3};


use crate::states::contexts::*;

pub fn init_nft_collection(ctx: Context<InitializeNftCollection>, nft_uri: String) -> Result<()> {

    // Create Metadata Account For The Curachain Collection
    let data_v2 = DataV2 {
        name: format!("Master Recognition NFT"),
        symbol: "CURA".to_string(),
        uri: nft_uri,
        seller_fee_basis_points: 0,
        creators: None,
        collection: None,
        uses: None
    };

    // Setting up multisig seeds for the signer
    let seeds = &[
            b"multisig",
            b"escrow-authority".as_ref(),
            &[ctx.accounts.multisig.multisig_bump]
        ];
    let multisig_seeds = &[&seeds[..]];

    let cpi_program = ctx.accounts.metadata_program.to_account_info();
    let cpi_accounts = CreateMetadataAccountsV3 {
        metadata: ctx.accounts.parent_collection_nft_metadata.to_account_info(),
        mint: ctx.accounts.parent_collection_mint.to_account_info(),
        //mint_authority: ctx.accounts.multisig.to_account_info(),
        mint_authority: ctx.accounts.admin.to_account_info(),
        payer: ctx.accounts.admin.to_account_info(),
        update_authority: ctx.accounts.admin.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(cpi_program.clone(), cpi_accounts, multisig_seeds);
    // Let's call the Actual Function To Create The Metadata Account
    // NB: Gotta Set Collection Details As This is a Collection, and not a regular NFT. 
    create_metadata_accounts_v3(cpi_ctx, data_v2, true, true, Some(
        CollectionDetails::V1 { size: 0}
    ))?;

    // Create Master Edition Account For First-Time Donations
    let edition_cpi_accounts = CreateMasterEditionV3 {
        edition: ctx.accounts.parent_collection_master_edition.to_account_info(),
        mint: ctx.accounts.parent_collection_mint.to_account_info(),
        update_authority: ctx.accounts.admin.to_account_info(),
        //mint_authority: ctx.accounts.multisig.to_account_info(),
        mint_authority: ctx.accounts.admin.to_account_info(),
        payer: ctx.accounts.admin.to_account_info(),
        metadata: ctx.accounts.parent_collection_nft_metadata.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };
    let cpi_context = CpiContext::new_with_signer(cpi_program.clone(), edition_cpi_accounts, multisig_seeds);
    // Let's Call The Actual Function To Create The Edition Account for the Collection
    // By setting its max_supply to 0, we make it unlimited from which we can mint a ton of regular NFTs
    create_master_edition_v3(cpi_context, Some(0))?;

    Ok(())
}