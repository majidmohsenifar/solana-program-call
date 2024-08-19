use std::str::FromStr;

use arrayref::{array_ref, array_refs};
use solana_client::rpc_client::RpcClient;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    sysvar,
};
use solana_sdk::{
    commitment_config::CommitmentConfig, signature::Keypair, signer::Signer, system_instruction,
    transaction::Transaction,
};
use spl_token::{solana_program::program_pack::Pack, state::Account, state::Mint};

fn main() {
    let rpc_url = String::from("http://127.0.0.1:8899");
    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
    let from_bytes: [u8; 64] = [
        199, 152, 84, 95, 15, 116, 179, 94, 217, 17, 231, 179, 163, 230, 50, 45, 51, 62, 214, 62,
        245, 237, 19, 15, 228, 101, 70, 182, 134, 38, 245, 120, 122, 12, 179, 244, 134, 101, 93,
        243, 163, 112, 127, 217, 54, 218, 167, 252, 49, 191, 206, 66, 184, 3, 52, 95, 103, 177,
        132, 254, 177, 190, 169, 169,
    ];

    let owner = Keypair::from_bytes(&from_bytes).unwrap();

    let token_x_res = create_token(&client, &owner);
    let token_y_res = create_token(&client, &owner);

    println!("res_t1 {:?}", token_x_res.mint_address.to_string());
    println!("res_t2 {:?}", token_y_res.mint_address.to_string());

    let alice_keypair_bytes: [u8; 64] = [
        126, 76, 48, 2, 2, 128, 18, 80, 19, 7, 160, 195, 44, 38, 193, 178, 196, 22, 75, 41, 231,
        235, 163, 219, 212, 192, 89, 7, 236, 101, 40, 251, 69, 172, 136, 99, 149, 148, 184, 45,
        236, 48, 20, 123, 83, 1, 234, 237, 216, 196, 69, 51, 188, 212, 189, 220, 181, 217, 184,
        124, 121, 218, 104, 32,
    ];
    let alice_keypair = Keypair::from_bytes(&alice_keypair_bytes).unwrap();

    let bob_keypair_bytes: [u8; 64] = [
        150, 122, 25, 134, 32, 233, 229, 98, 79, 53, 251, 118, 50, 84, 35, 55, 144, 239, 80, 86,
        113, 230, 49, 154, 120, 8, 226, 34, 53, 229, 219, 56, 37, 231, 63, 146, 252, 157, 16, 132,
        53, 209, 118, 118, 220, 208, 220, 102, 44, 211, 108, 32, 94, 29, 53, 81, 216, 123, 55, 169,
        253, 52, 140, 189,
    ];
    let bob_keypair = Keypair::from_bytes(&bob_keypair_bytes).unwrap();

    //TODO: comment the following after the first run
    //client
    //.request_airdrop(&alice_keypair.pubkey(), 10_000_000_000_000)
    //.unwrap();
    //client
    //.request_airdrop(&bob_keypair.pubkey(), 10_000_000_000_000)
    //.unwrap();

    let alice_x_token_account = create_associated_account(
        &client,
        &token_x_res.mint_address,
        &owner,
        &alice_keypair.pubkey(),
    );

    let alice_y_token_account = create_associated_account(
        &client,
        &token_y_res.mint_address,
        &owner,
        &alice_keypair.pubkey(),
    );

    //let bob_x_token_account = create_associated_account(&client, &token_x_res.mint_address, &owner, &bob_keypair.pubkey());
    let bob_y_token_account = create_associated_account(
        &client,
        &token_y_res.mint_address,
        &owner,
        &bob_keypair.pubkey(),
    );

    transfer_token(
        &client,
        &owner,
        &token_x_res.assoc_account,
        &alice_x_token_account,
        1_500_000,
    );
    transfer_token(
        &client,
        &owner,
        &token_y_res.assoc_account,
        &bob_y_token_account,
        1_500_000,
    );

    let alice_x_token_balance = client
        .get_token_account_balance(&alice_x_token_account)
        .unwrap();
    println!("token balance: {}", alice_x_token_balance.ui_amount_string);

    let bob_y_token_balance = client
        .get_token_account_balance(&bob_y_token_account)
        .unwrap();
    println!("token balance: {}", bob_y_token_balance.ui_amount_string);

    let escrow_program_id = "83ETxBzF4fYEw9sHkfscTg9FufDErqRuTsvJvtMKohe9";
    let escrow_program_pubkey = Pubkey::from_str(escrow_program_id).unwrap();

    let expected_amount: u64 = 300_000;

    let min_rent_exempt_for_temp_token = client
        .get_minimum_balance_for_rent_exemption(Account::LEN)
        .unwrap();

    let alice_temp_token_account_keypair = Keypair::new();
    let alice_x_token_temp_account_inx = system_instruction::create_account(
        &alice_keypair.pubkey(),
        &alice_temp_token_account_keypair.pubkey(),
        min_rent_exempt_for_temp_token,
        Account::LEN as u64,
        &spl_token::ID,
    );
    println!(
        "temp token account: {:?}",
        alice_temp_token_account_keypair.pubkey().to_string()
    );

    let init_temp_account_inx = spl_token::instruction::initialize_account(
        &spl_token::ID,
        &alice_temp_token_account_keypair.pubkey(),
        &token_x_res.mint_address,
        &alice_keypair.pubkey(),
    )
    .unwrap();

    let transfer_token_to_temp_account_inx = spl_token::instruction::transfer(
        &spl_token::ID,
        &alice_x_token_account,
        &alice_temp_token_account_keypair.pubkey(),
        &alice_keypair.pubkey(),
        &[&alice_keypair.pubkey()],
        500_000,
    )
    .unwrap();

    let mut dst = vec![0; 9];
    dst[0] = 0;
    dst[1..].clone_from_slice(&expected_amount.to_le_bytes());

    let space: u64 = 105;
    let min_rent_exempt = client
        .get_minimum_balance_for_rent_exemption(space as usize)
        .unwrap();

    let escrow_account_keypair = Keypair::new();
    let create_escrow_account_inx = system_instruction::create_account(
        &alice_keypair.pubkey(),
        &escrow_account_keypair.pubkey(),
        min_rent_exempt,
        space,
        &escrow_program_pubkey,
    );

    let account_metas = vec![
        AccountMeta::new_readonly(alice_keypair.pubkey(), true),
        AccountMeta::new(alice_x_token_account, false),
        AccountMeta::new_readonly(alice_y_token_account, false),
        AccountMeta::new(escrow_account_keypair.pubkey(), false),
        AccountMeta::new(sysvar::rent::ID, false),
        AccountMeta::new(spl_token::ID, false),
    ];

    let init_escrow_inx = Instruction {
        program_id: escrow_program_pubkey,
        accounts: account_metas,
        data: dst,
    };

    let block_hash = client.get_latest_blockhash().unwrap();

    let final_tx = Transaction::new_signed_with_payer(
        &[
            alice_x_token_temp_account_inx,
            init_temp_account_inx,
            transfer_token_to_temp_account_inx,
            create_escrow_account_inx,
            init_escrow_inx,
        ],
        Some(&alice_keypair.pubkey()),
        &[
            &alice_keypair,
            &alice_temp_token_account_keypair,
            &escrow_account_keypair,
        ],
        block_hash,
    );

    let final_tx_result = client.send_and_confirm_transaction(&final_tx).unwrap();
    println!("final tx hash is: {}", final_tx_result);

    let account_data = client
        .get_account_data(&escrow_account_keypair.pubkey())
        .unwrap();
    let escrow = unpack_from_slice(&account_data.to_vec());
    println!("escrow is {:?}", escrow);
}

struct CreateTokenResult {
    mint_address: Pubkey,
    assoc_account: Pubkey,
}

fn create_token(client: &RpcClient, owner: &Keypair) -> CreateTokenResult {
    let mint_account = Keypair::new();
    let decimals = 9;
    let min_rent_exempt = client
        .get_minimum_balance_for_rent_exemption(Mint::LEN)
        .unwrap();

    let create_account_inx: Instruction = system_instruction::create_account(
        &owner.pubkey(),
        &mint_account.pubkey(),
        min_rent_exempt,
        Mint::LEN as u64,
        &spl_token::ID,
    );

    let init_mint_inx = spl_token::instruction::initialize_mint(
        &spl_token::ID,
        &mint_account.pubkey(),
        &owner.pubkey(),
        None,
        decimals,
    )
    .unwrap();
    let block_hash = client.get_latest_blockhash().unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[create_account_inx, init_mint_inx],
        Some(&owner.pubkey()),
        &[&mint_account, owner],
        block_hash,
    );

    let tx_result = client.send_and_confirm_transaction(&tx).unwrap();
    println!("hash is: {}", tx_result);
    println!("mint account: {:?}", mint_account.pubkey().to_string());

    //here we are going to mint
    let mint_amount = 1_000_000_000_000_000_000;
    let assoc_account = spl_associated_token_account::get_associated_token_address(
        &owner.pubkey(),
        &mint_account.pubkey(),
    );

    //let assoc_inx = spl_associated_token_account::instruction::create_associated_token_account(
    //&owner.pubkey(),
    //&assoc_account,
    //&owner.pubkey(),
    //&system_program::ID,
    //);

    #[allow(deprecated)]
    // https://github.com/solana-labs/solana-program-library/issues/2791
    let assoc_inx = spl_associated_token_account::create_associated_token_account(
        &owner.pubkey(),
        &owner.pubkey(),
        &mint_account.pubkey(),
    );

    //spl_associated_token_account::create_associated_token_account(funding_address, wallet_address, token_mint_address)

    let mint_inx = spl_token::instruction::mint_to(
        &spl_token::ID,
        &mint_account.pubkey(),
        &assoc_account, //assouc
        &owner.pubkey(),
        &[&owner.pubkey()],
        mint_amount,
    )
    .unwrap();

    let block_hash = client.get_latest_blockhash().unwrap();
    let mint_tx = Transaction::new_signed_with_payer(
        &[assoc_inx, mint_inx],
        Some(&owner.pubkey()),
        &[&owner],
        //&[&mint_account, &owner],
        block_hash,
    );

    let mint_tx_result = client.send_and_confirm_transaction(&mint_tx).unwrap();
    println!("hash is: {}", mint_tx_result);
    println!("Associated token account: {:?}", assoc_account.to_string());

    CreateTokenResult {
        mint_address: mint_account.pubkey(),
        assoc_account,
    }
}

struct TransferTokenResult {}

fn transfer_token(
    client: &RpcClient,
    owner: &Keypair,
    owner_associate_account: &Pubkey,
    associated_to: &Pubkey,
    amount: u64,
) -> TransferTokenResult {
    let transfer_inx = spl_token::instruction::transfer(
        &spl_token::ID,
        owner_associate_account,
        associated_to,
        &owner.pubkey(),
        &[&owner.pubkey()],
        amount,
    )
    .unwrap();

    let block_hash = client.get_latest_blockhash().unwrap();

    let transfer_tx = Transaction::new_signed_with_payer(
        &[transfer_inx],
        Some(&owner.pubkey()),
        &[&owner],
        block_hash,
    );
    let _tx_result = client.send_and_confirm_transaction(&transfer_tx).unwrap();
    TransferTokenResult {}
}

fn create_associated_account(
    client: &RpcClient,
    mint_account: &Pubkey,
    owner: &Keypair,
    to: &Pubkey,
) -> Pubkey {
    let assoc_account =
        spl_associated_token_account::get_associated_token_address(to, mint_account);

    #[allow(deprecated)]
    // https://github.com/solana-labs/solana-program-library/issues/2791
    //
    //
    //
    let assoc_inx = spl_associated_token_account::create_associated_token_account(
        &owner.pubkey(),
        to,
        mint_account,
    );

    let block_hash = client.get_latest_blockhash().unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[assoc_inx],
        Some(&owner.pubkey()),
        &[&owner],
        block_hash,
    );

    let _tx_result = client.send_and_confirm_transaction(&tx).unwrap();
    assoc_account
}

#[derive(Debug)]
pub struct Escrow {
    pub is_initialized: bool,
    pub initializer_pubkey: Pubkey,
    pub temp_token_account_pubkey: Pubkey,
    pub initializer_token_to_receive_account_pubkey: Pubkey,
    pub expected_amount: u64,
}

fn unpack_from_slice(src: &[u8]) -> Escrow {
    let src = array_ref![src, 0, 105];
    let (
        is_initialized,
        initializer_pubkey,
        temp_token_account_pubkey,
        initializer_token_to_receive_account_pubkey,
        expected_amount,
    ) = array_refs![src, 1, 32, 32, 32, 8];

    let is_initialized = match is_initialized {
        [0] => false,
        [1] => true,
        _ => {
            panic!("cannot do unpack ")
        }
    };

    Escrow {
        is_initialized,
        initializer_pubkey: Pubkey::new_from_array(*initializer_pubkey),
        temp_token_account_pubkey: Pubkey::new_from_array(*temp_token_account_pubkey),
        initializer_token_to_receive_account_pubkey: Pubkey::new_from_array(
            *initializer_token_to_receive_account_pubkey,
        ),
        expected_amount: u64::from_le_bytes(*expected_amount),
    }
}
