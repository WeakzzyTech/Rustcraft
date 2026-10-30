//! The Microsoft account the client plays as when one is passed by a
//! launcher or explicitly signed in from the menu. Offline play never reads
//! or writes the store.

use std::path::Path;

use rustcraft_auth::SecretStore;

/// The OS credential store on desktop, shared with the launcher; Android has
/// none, so a file in the app's private storage.
// TODO: wrap the Android file in the Android Keystore.
pub fn store(game_dir: &Path) -> Box<dyn SecretStore> {
    #[cfg(target_os = "android")]
    return Box::new(rustcraft_auth::FileStore {
        path: game_dir.join("accounts.json"),
    });
    #[cfg(not(target_os = "android"))]
    {
        let _ = game_dir;
        Box::new(rustcraft_auth::KeyringStore)
    }
}

/// Removes `uuid`'s stored account, whichever way its uuid was written.
pub fn forget(store: &dyn SecretStore, uuid: &uuid::Uuid) {
    if let Some(account) = rustcraft_auth::get_all_accounts(store)
        .into_iter()
        .find(|a| uuid::Uuid::parse_str(&a.uuid).is_ok_and(|stored| stored == *uuid))
    {
        rustcraft_auth::remove_account(store, &account.uuid);
    }
}
