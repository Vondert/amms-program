use anchor_lang::{account, InitSpace};
use anchor_lang::prelude::*;

/// Represents the manager for AMM configurations.
///
/// This account manages multiple AMM configurations and tracks the authority
/// responsible for their governance. It also maintains a count of configurations
/// and provides utility methods for updating key parameters.
#[account]
#[derive(InitSpace)]
pub struct AmmsConfigsManager {
    /// The public key of the authority responsible for managing configurations.
    authority: Pubkey,  // 32 bytes

    /// The public key of the head authority, which may have broader governance privileges.
    head_authority: Pubkey, // 32 bytes

    /// The number of configurations managed by this account.
    configs_count: u64, // 8 bytes

    /// The canonical bump seed used for the account's PDA.
    bump: u8,   // 1 byte
}

impl AmmsConfigsManager {
    /// The seed used to derive the account's PDA.
    pub const SEED: &'static [u8] = b"amms_configs_manager";

    /// Initializes the `AmmsConfigsManager` with the provided parameters.
    ///
    /// # Parameters
    /// - `authority`: The public key of the initial authority for managing configurations.
    /// - `head_authority`: The public key of the head authority with broader governance privileges.
    /// - `bump`: The bump seed for the account's PDA.
    ///
    /// # Behavior
    /// - Sets the initial `configs_count` to 0.
    /// - Updates the authority and head authority fields with the provided values.
    pub(crate) fn initialize(&mut self, authority: Pubkey, head_authority: Pubkey, bump: u8) {
        self.bump = bump;
        self.configs_count = 0;
        self.update_authority(authority);
        self.update_head_authority(head_authority);
    }


    /// Updates the `authority` field with a new public key.
    ///
    /// # Parameters
    /// - `authority`: The new public key for the authority managing configurations.
    pub(crate) fn update_authority(&mut self, authority: Pubkey) {
        self.authority = authority;
    }

    /// Updates the `head_authority` field with a new public key.
    ///
    /// # Parameters
    /// - `head_authority`: The new public key for the head authority with broader governance privileges.
    pub(crate) fn update_head_authority(&mut self, head_authority: Pubkey) {
        self.head_authority = head_authority;
    }

    /// Increments the `configs_count` field by 1.
    ///
    /// # Behavior
    /// - Safely increments the count of configurations, ensuring no overflow occurs.
    /// - Panics if the increment operation fails (e.g., due to an overflow).
    pub(crate) fn increment_configs_count(&mut self) {
        self.configs_count = self.configs_count.checked_add(1).unwrap()
    }

    pub fn head_authority(&self) -> &Pubkey {
        &self.head_authority
    }
    pub fn authority(&self) -> &Pubkey {
        &self.authority
    }
    pub fn configs_count(&self) -> u64 {
        self.configs_count
    }
    pub fn bump(&self) -> u8 {
        self.bump
    }
}

#[cfg(test)]
mod amms_configs_manager_tests {
    use anchor_lang::Discriminator;
    use crate::constants::ANCHOR_DISCRIMINATOR;
    use super::*;

    /// Tests the correct initialization of the `AmmsConfigsManager` struct.
    #[test]
    fn test_amms_configs_manager_initialize() {
        let mut manager = AmmsConfigsManager {
            authority: Pubkey::default(),
            head_authority: Pubkey::default(),
            configs_count: 0,
            bump: 0,
        };

        let authority = Pubkey::new_unique();
        let head_authority = Pubkey::new_unique();
        let bump = 42u8;

        manager.initialize(authority, head_authority, bump);

        assert_eq!(manager.authority, authority);
        assert_eq!(manager.head_authority, head_authority);
        assert_eq!(manager.configs_count, 0);
        assert_eq!(manager.bump, bump);
    }
    
    /// Tests the `update_authority` method of the `AmmsConfigsManager` struct.
    #[test]
    fn test_amms_configs_manager_update_authority() {
        let mut manager = AmmsConfigsManager {
            authority: Pubkey::default(),
            head_authority: Pubkey::new_unique(),
            configs_count: 10,
            bump: 42,
        };

        let new_authority = Pubkey::new_unique();
        manager.update_authority(new_authority);
        
        assert_eq!(manager.authority, new_authority);
    }

    /// Tests the `update_head_authority` method of the `AmmsConfigsManager` struct.
    #[test]
    fn test_amms_configs_manager_update_head_authority(){
        let mut manager = AmmsConfigsManager {
            authority: Pubkey::new_unique(),
            head_authority: Pubkey::default(),
            configs_count: 10,
            bump: 42,
        };

        let new_head_authority = Pubkey::new_unique();
        manager.update_head_authority(new_head_authority);

        assert_eq!(manager.head_authority, new_head_authority);
    }

    /// Tests the `increment_configs_count` method of the `AmmsConfigsManager` struct.
    #[test]
    fn test_amms_configs_manager_increment_configs_count(){
        let mut manager = AmmsConfigsManager {
            authority: Pubkey::new_unique(),
            head_authority: Pubkey::new_unique(),
            configs_count: 5,
            bump: 42,
        };

        manager.increment_configs_count();

        assert_eq!(manager.configs_count, 6);
    }
    
    /// Tests the `update_head_authority` method of the `AmmsConfigsManager` struct.
    #[test]
    fn test_amms_configs_manager_data_layout() {
        let authority = Pubkey::new_unique();
        let head_authority = Pubkey::new_unique();
        let configs_count = 42u64;
        let bump = 42u8;

        let mut data = [0u8; ANCHOR_DISCRIMINATOR + 73];
        let mut offset = 0;

        data[offset..offset + ANCHOR_DISCRIMINATOR].copy_from_slice(&AmmsConfigsManager::discriminator()); offset += ANCHOR_DISCRIMINATOR;
        data[offset..offset + 32].copy_from_slice(authority.as_ref()); offset += 32;
        data[offset..offset + 32].copy_from_slice(head_authority.as_ref()); offset += 32;
        data[offset..offset + 8].copy_from_slice(&configs_count.to_le_bytes()); offset += 8;
        data[offset] = bump; offset += 1;

        assert_eq!(offset, ANCHOR_DISCRIMINATOR + 73);
        
        let deserialized_manager = AmmsConfigsManager::try_deserialize(&mut data.as_ref()).unwrap();

        assert_eq!(deserialized_manager.authority, authority);
        assert_eq!(deserialized_manager.head_authority, head_authority);
        assert_eq!(deserialized_manager.configs_count, configs_count);
        assert_eq!(deserialized_manager.bump, bump);

        let mut serialized_data = Vec::new();
        deserialized_manager.try_serialize(&mut serialized_data).unwrap();
        assert_eq!(serialized_data.as_slice(), data.as_ref());
    }
}