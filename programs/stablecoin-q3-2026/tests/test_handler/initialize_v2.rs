use {
    anchor_lang::{
        solana_program::{instruction::Instruction, system_program},
        InstructionData, ToAccountMetas,
    },
    anchor_spl::token_2022::spl_token_2022::{self, extension::ExtensionType},
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

pub fn fixed_v2_extensions() -> [ExtensionType; 6] {
    [
        ExtensionType::TransferFeeConfig,
        ExtensionType::MintCloseAuthority,
        ExtensionType::DefaultAccountState,
        ExtensionType::MetadataPointer,
        ExtensionType::PermanentDelegate,
        ExtensionType::ConfidentialTransferMint,
    ]
}

pub fn init_mint_v2(svm: &mut LiteSVM, payer: &Keypair) -> Keypair {
    let mint = Keypair::new();
    let ix = Instruction {
        program_id: token22_ct::ID,
        accounts: token22_ct::accounts::InitalizeMintV2 {
            mint: mint.pubkey(),
            payer: payer.pubkey(),
            system_program: system_program::ID,
            token_program: spl_token_2022::ID,
        }
        .to_account_metas(None),
        data: token22_ct::instruction::InitalizeV2 {}.data(),
    };
    crate::send(svm, payer, &[&mint], vec![ix]);
    mint
}

pub fn mint_data_v2(svm: &LiteSVM, mint: &Keypair) -> Vec<u8> {
    let account = svm
        .get_account(&mint.pubkey())
        .expect("mint account must exist after initialize_v2");
    assert_eq!(
        account.owner.to_bytes(),
        spl_token_2022::ID.to_bytes(),
        "mint owner must be the Token-2022 program"
    );
    account.data.clone()
}
