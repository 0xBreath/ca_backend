// use anyhow::Context;
// use postgres_client::client::{DbAccountEvent, PgClient, SimplePostgresClient};
// use postgres_client::configurations::get_configuration;
// use rand::{thread_rng, Rng};
// use solana_program::pubkey::Pubkey;
// use std::borrow::Cow;
// use fuzzy_lemur_messages::{BytesWrapper, SlotStatus, UpdateAccountEvent};
// use uuid::Uuid;

// #[tokio::test]
// async fn insert_processed_account_event() -> Result<(), anyhow::Error> {
//     let mut rng = thread_rng();
//     let configuration = {
//         let mut c = get_configuration().expect("Failed to read configuration.");
//         c.database_name = Some(Uuid::new_v4().to_string());
//         println!("Database: {:?}", c);
//         c
//     };
//     let client = SimplePostgresClient::new(&configuration)?;
//     let account = UpdateAccountEvent {
//         key: Pubkey::new_unique(),
//         slot: 1,
//         lamports: 10000,
//         owner: Pubkey::new_unique(),
//         executable: false,
//         rent_epoch: 100,
//         data: BytesWrapper(Cow::Borrowed(Box::leak(Box::new(vec![
//             rng.gen();
//             15
//         ])))),
//         write_version: 100,
//         is_startup: false,
//     };
//     let account_update = DbAccountEvent::new(&account, SlotStatus::Processed);
//
//     // let saved = sqlx::query!("SELECT key, slot, write_version, is_startup, lamports, owner, executable, rent_epoch, confirmation, data FROM accounts",)
//     //     .fetch_one(&connection)
//     //     .await
//     //     .expect("Failed to fetch account.");
//     // assert_eq!(saved.key, account.key.to_string());
//     // assert_eq!(saved.slot, account.slot as i64);
//     // assert_eq!(saved.lamports, account.lamports as i64);
//     // assert_eq!(saved.owner, Some(account.owner.to_string()));
//     // assert_eq!(saved.executable, account.executable);
//     // assert_eq!(saved.rent_epoch, account.rent_epoch as i64);
//     // assert_eq!(saved.confirmation, Some(String::from("PROCESSED")));
//     // assert_eq!(saved.data, Some(account.data.to_bytes().to_vec()));
//     Ok(())
// }

// #[test]
// fn test_db_connection() -> Result<(), anyhow::Error> {
//     let configuration = {
//         let mut c = get_configuration().expect("Failed to read configuration.");
//         c.database_name = Some(Uuid::new_v4().to_string());
//         println!("Database: {:?}", c);
//         c
//     };
//     SimplePostgresClient::new(&configuration)?;
//     Ok(())
// }
