use miden_lib::transaction::TransactionKernel;
use miden_objects::{
    accounts::{
        Account, AccountCode, AccountId, AccountStorage, SlotItem, StorageSlot,
        ACCOUNT_ID_FUNGIBLE_FAUCET_ON_CHAIN, ACCOUNT_ID_REGULAR_ACCOUNT_UPDATABLE_CODE_ON_CHAIN,
        ACCOUNT_ID_SENDER,
    },
    assembly::{ModuleAst, ProgramAst},
    assets::{Asset, AssetVault, FungibleAsset},
    crypto::rand::{FeltRng, RpoRandomCoin},
    notes::{
        Note, NoteAssets, NoteExecutionMode, NoteInputs, NoteMetadata, NoteRecipient, NoteScript,
        NoteTag, NoteType,
    },
    transaction::TransactionArgs,
    Felt, NoteError, Word, ONE, ZERO,
};
use miden_processor::AdviceMap;
use miden_tx::TransactionExecutor;
use mock::mock::account::DEFAULT_AUTH_SCRIPT;

mod utils;
use utils::{get_new_key_pair_with_advice_map, MockDataStore};

// use std::include_str;
// use crate::prove_and_verify_transaction;

pub fn get_account_with_custom_account_code(
    account_id: AccountId,
    public_key: Word,
    assets: Option<Asset>,
) -> Account {
    let account_code_src = include_str!("../src/masm/lifecycle/test_account.masm");
    let account_code_ast = ModuleAst::parse(account_code_src).unwrap();
    let account_assembler = TransactionKernel::assembler().with_debug_mode(true);

    println!("is debug mode on: {}", account_assembler.in_debug_mode());
    assert!(account_assembler.in_debug_mode());

    let account_code = AccountCode::new(account_code_ast.clone(), &account_assembler).unwrap();
    let account_storage = AccountStorage::new(vec![SlotItem {
        index: 0,
        slot: StorageSlot::new_value(public_key),
    }])
    .unwrap();

    let account_vault = match assets {
        Some(asset) => AssetVault::new(&[asset]).unwrap(),
        None => AssetVault::new(&[]).unwrap(),
    };

    Account::new(
        account_id,
        account_vault,
        account_storage,
        account_code,
        Felt::new(1),
    )
}

fn create_note<R: FeltRng>(
    sender_account_id: AccountId,
    target_account_id: AccountId,
    assets: Vec<Asset>,
    mut rng: R,
) -> Result<Note, NoteError> {
    let note_script = include_str!("../src/masm/lifecycle/test_note_script.masm");

    let note_assembler = TransactionKernel::assembler().with_debug_mode(true);

    println!("is debug mode on: {}", note_assembler.in_debug_mode());
    assert!(note_assembler.in_debug_mode());

    let script_ast = ProgramAst::parse(note_script).unwrap();
    let (note_script, _) = NoteScript::new(script_ast, &note_assembler)?;

    let inputs = NoteInputs::new(vec![ONE, ONE])?;

    let tag = NoteTag::from_account_id(target_account_id, NoteExecutionMode::Local)?;
    let serial_num = rng.draw_word();
    let aux = ZERO;
    let note_type = NoteType::OffChain;
    let metadata = NoteMetadata::new(sender_account_id, note_type, tag, aux)?;

    let vault = NoteAssets::new(assets)?;

    let recipient = NoteRecipient::new(serial_num, note_script, inputs);

    Ok(Note::new(vault, metadata, recipient))
}

// Note TESTS
// ===============================================================================================
// We test the Note script.
#[test]
fn note_script_test() {
    // Create an asset
    let faucet_id = AccountId::try_from(ACCOUNT_ID_FUNGIBLE_FAUCET_ON_CHAIN).unwrap();
    let fungible_asset: Asset = FungibleAsset::new(faucet_id, 100).unwrap().into();

    // Create sender and target account
    let sender_account_id = AccountId::try_from(ACCOUNT_ID_SENDER).unwrap();

    let target_account_id =
        AccountId::try_from(ACCOUNT_ID_REGULAR_ACCOUNT_UPDATABLE_CODE_ON_CHAIN).unwrap();
    let (target_pub_key, target_sk_pk_felt) = get_new_key_pair_with_advice_map();
    let target_account =
        get_account_with_custom_account_code(target_account_id, target_pub_key, None);

    // Create the note
    let note = create_note(
        sender_account_id,
        target_account_id,
        vec![fungible_asset],
        RpoRandomCoin::new([Felt::new(1), Felt::new(2), Felt::new(3), Felt::new(4)]),
    )
    .unwrap();

    // CONSTRUCT AND EXECUTE TX (Success)
    // --------------------------------------------------------------------------------------------
    let data_store =
        MockDataStore::with_existing(Some(target_account.clone()), Some(vec![note.clone()]));

    let mut executor = TransactionExecutor::new(data_store.clone());
    executor.load_account(target_account_id).unwrap();

    let block_ref = data_store.block_header.block_num();
    let note_ids = data_store
        .notes
        .iter()
        .map(|note| note.id())
        .collect::<Vec<_>>();

    let tx_script_code = ProgramAst::parse(DEFAULT_AUTH_SCRIPT).unwrap();

    let tx_script_target = executor
        .compile_tx_script(
            tx_script_code.clone(),
            vec![(target_pub_key, target_sk_pk_felt)],
            vec![],
        )
        .unwrap();
    let tx_args_target = TransactionArgs::new(Some(tx_script_target), None, AdviceMap::default());

    // Execute the transaction and get the witness
    let _executed_transaction = executor
        .execute_transaction(target_account_id, block_ref, &note_ids, tx_args_target)
        .unwrap();

    // println!("{}", _executed_transaction.)

    // Prove, serialize/deserialize and verify the transaction
    // We can add this as a last step
    //assert!(prove_and_verify_transaction(executed_transaction.clone()).is_ok());

    // Not sure what you want to test after the account but we should see if the
    // account change is what you expect
    // let mut target_storage = target_account.storage().clone();
    // target_storage.set_item(100, [Felt::new(99), Felt::new(99), Felt::new(99), Felt::new(99)]).unwrap();
    // target_storage.set_item(101, [Felt::new(98), Felt::new(98), Felt::new(98), Felt::new(98)]).unwrap();

    // let target_account_after: Account = Account::new(
    //     target_account.id(),
    //     AssetVault::new(&[fungible_asset]).unwrap(),
    //     target_storage,
    //     target_account.code().clone(),
    //     Felt::new(2),
    // );
    // assert_eq!(executed_transaction.final_account().hash(), target_account_after.hash());
}
