use md5::Digest as _;

pub struct UserData {
    pub username: String,
    pub uuid: uuid::Uuid,
    pub access_token: Option<String>,
    /// True when `uuid` is a launcher-supplied profile UUID rather than a
    /// synthesized offline one (which has no Mojang profile to fetch).
    pub has_profile: bool,
}

impl UserData {
    /// Path to the personal username config file: `~/.rustcraft/username.cfg`.
    fn username_config_path() -> std::path::PathBuf {
        std::env::home_dir()
            .map(|d| d.join(".rustcraft/username.cfg"))
            .unwrap_or_default()
    }

    /// Try to read the username from the config file; returns None if the file
    /// doesn't exist or is empty/whitespace-only.
    fn username_from_config() -> Option<String> {
        std::fs::read_to_string(Self::username_config_path())
            .ok()
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string())
    }

    pub fn from_args(
        username: Option<String>,
        uuid: Option<String>,
        access_token: Option<String>,
    ) -> Self {
        let username = username
            .or_else(|| Self::username_from_config())
            .unwrap_or_else(|| "Steve".to_string());

        let uuid = uuid.and_then(|s| uuid::Uuid::parse_str(&s).ok());
        let has_profile = uuid.is_some();
        let uuid = uuid.unwrap_or_else(|| Self::offline_uuid(&username));

        Self {
            username,
            uuid,
            access_token,
            has_profile,
        }
    }

    /// Who a launch plays as. Account flags from a launcher are used when
    /// present, so an explicitly chosen account still works with online
    /// servers; without them the client plays offline, never reads the
    /// keychain, and never asks for a Microsoft sign-in.
    pub fn for_launch(
        username: Option<String>,
        uuid: Option<String>,
        access_token: Option<String>,
    ) -> Self {
        Self::from_args(username, uuid, access_token)
    }

    pub fn from_account(account: &rustcraft_auth::AuthAccount) -> Self {
        Self::from_args(
            Some(account.username.clone()),
            Some(account.uuid.clone()),
            Some(account.access_token.clone()),
        )
    }

    /// Vanilla `UUIDUtil.createOfflinePlayerUUID`: a bare MD5 of the prefixed
    /// name, not a namespaced v3 UUID.
    fn offline_uuid(username: &str) -> uuid::Uuid {
        let digest = md5::Md5::digest(format!("OfflinePlayer:{username}").as_bytes());
        uuid::Builder::from_md5_bytes(digest.into()).into_uuid()
    }
}

#[cfg(test)]
mod tests {
    use super::UserData;

    #[test]
    fn offline_uuid_matches_vanilla() {
        for (name, expected) in [
            ("Notch", "b50ad385-829d-3141-a216-7e7d7539ba7f"),
            ("Steve", "5627dd98-e6be-3c21-b8a8-e92344183641"),
            ("Alex", "36532b5e-c442-3dbb-a24c-c7e55d0f979a"),
        ] {
            assert_eq!(UserData::offline_uuid(name).to_string(), expected);
        }
    }

    #[test]
    fn launch_flags_are_used_without_requiring_sign_in() {
        let user = UserData::for_launch(
            Some("Steve".to_owned()),
            Some("5627dd98-e6be-3c21-b8a8-e92344183641".to_owned()),
            Some("token".to_owned()),
        );
        assert_eq!(user.access_token.as_deref(), Some("token"));
        assert!(user.has_profile);
    }

    #[test]
    fn bare_launches_play_offline() {
        let user = UserData::for_launch(None, None, None);
        assert_eq!(user.username, "Steve");
        assert!(!user.has_profile);
        assert!(user.access_token.is_none());
        assert_eq!(
            user.uuid.to_string(),
            "5627dd98-e6be-3c21-b8a8-e92344183641"
        );
    }
}
