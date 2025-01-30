use anchor_lang::{account, InitSpace};
use anchor_lang::prelude::*;
use crate::error::ErrorCode;

/// Represents a configuration object for managing fees and authorities in AMMs.
///
/// This structure contains details such as fee rates, the authority responsible for fee collection,
/// and a unique identifier to track the configuration within an `AmmsConfigsManager`.
#[account]
#[derive(InitSpace)]
pub struct AmmsConfig {
    /// Canonical bump seed for the account's PDA.
    bump: u8,   // 1 byte

    /// A unique identifier for this configuration in the `AmmsConfigsManager` collection.
    pub id: u64,    // 8 bytes

    /// The public key of the authority responsible for collecting protocol fees from pools.
    fee_authority: Pubkey,  // 32 bytes

    /// The fee rate for liquidity providers, measured in basis points (1 basis point = 0.01%).
    providers_fee_rate_basis_points: u16, // 2 bytes

    /// The protocol's fee rate, measured in basis points (1 basis point = 0.01%).
    protocol_fee_rate_basis_points: u16, // 2 bytes
}

impl AmmsConfig {
    /// The seed used to derive the account's PDA.
    pub const SEED: &'static [u8] = b"amms_config";

    /// Initializes the `AmmsConfig` with the provided parameters.
    ///
    /// # Parameters
    /// - `fee_authority`: The public key of the authority that will collect fees.
    /// - `protocol_fee_rate_basis_points`: The protocol fee rate, measured in basis points (1 = 0.01%).
    /// - `providers_fee_rate_basis_points`: The providers' fee rate, measured in basis points (1 = 0.01%).
    /// - `id`: A unique identifier for this configuration.
    /// - `bump`: The bump seed for the account's PDA.
    ///
    /// # Errors
    /// - Returns `ErrorCode::ConfigFeeRateExceeded` if the sum of `protocol_fee_rate_basis_points`
    ///   and `providers_fee_rate_basis_points` exceeds 10,000 (100%).
    pub(crate) fn initialize(&mut self, fee_authority: Pubkey, protocol_fee_rate_basis_points: u16, providers_fee_rate_basis_points: u16, id: u64, bump: u8) -> Result<()> {
        require!(providers_fee_rate_basis_points + protocol_fee_rate_basis_points <= 10000, ErrorCode::ConfigFeeRateExceeded);
        
        self.bump = bump;
        self.id = id;
        self.protocol_fee_rate_basis_points = protocol_fee_rate_basis_points;
        self.providers_fee_rate_basis_points = providers_fee_rate_basis_points;
        self.fee_authority = fee_authority;
        
        Ok(())
    }

    /// Updates the `fee_authority` field with a new authority public key.
    ///
    /// # Parameters
    /// - `fee_authority`: The new public key of the authority responsible for collecting fees.
    pub(crate) fn update_fee_authority(&mut self, fee_authority: Pubkey) {
        self.fee_authority = fee_authority;
    }


    /// Updates the fee rate for liquidity providers.
    ///
    /// Ensures that the sum of the updated `providers_fee_rate_basis_points` and
    /// the existing `protocol_fee_rate_basis_points` does not exceed 10,000 basis points (100%).
    ///
    /// # Parameters
    /// - `new_providers_fee_rate_basis_points`: The updated fee rate for liquidity providers,
    ///   measured in basis points.
    ///
    /// # Errors
    /// - Returns `ErrorCode::ConfigFeeRateExceeded` if the total fee rate exceeds 100%.
    pub(crate) fn update_providers_fee_rate(&mut self, new_providers_fee_rate_basis_points: u16) -> Result<()> {
        require!(
            new_providers_fee_rate_basis_points + self.protocol_fee_rate_basis_points <= 10000,
            ErrorCode::ConfigFeeRateExceeded
        );
        self.providers_fee_rate_basis_points = new_providers_fee_rate_basis_points;
        Ok(())
    }

    /// Updates the protocol fee rate.
    ///
    /// Ensures that the sum of the updated `protocol_fee_rate_basis_points` and
    /// the existing `providers_fee_rate_basis_points` does not exceed 10,000 basis points (100%).
    ///
    /// # Parameters
    /// - `new_protocol_fee_rate_basis_points`: The updated protocol fee rate, measured in basis points.
    ///
    /// # Errors
    /// - Returns `ErrorCode::ConfigFeeRateExceeded` if the total fee rate exceeds 100%.
    pub(crate) fn update_protocol_fee_rate(&mut self, new_protocol_fee_rate_basis_points: u16) -> Result<()> {
        require!(
            new_protocol_fee_rate_basis_points + self.providers_fee_rate_basis_points <= 10000,
            ErrorCode::ConfigFeeRateExceeded
        );
        self.protocol_fee_rate_basis_points = new_protocol_fee_rate_basis_points;
        Ok(())
    }

    /// Retrieves the public key of the current fee authority.
    ///
    /// # Returns
    /// - A reference to the `Pubkey` of the fee authority.
    #[inline]
    pub fn fee_authority(&self) -> &Pubkey {
        &self.fee_authority
    }

    /// Retrieves the PDA bump seed associated with this configuration.
    ///
    /// # Returns
    /// - The `u8` bump seed.
    #[inline]
    pub fn bump(&self) -> u8 {
        self.bump
    }

    /// Retrieves the fee rate for liquidity providers.
    ///
    /// # Returns
    /// - The `u16` fee rate for providers, measured in basis points.
    #[inline]
    pub fn providers_fee_rate_basis_points(&self) -> u16 {
        self.providers_fee_rate_basis_points
    }

    /// Retrieves the protocol fee rate.
    ///
    /// # Returns
    /// - The `u16` protocol fee rate, measured in basis points.
    #[inline]
    pub fn protocol_fee_rate_basis_points(&self) -> u16 {
        self.protocol_fee_rate_basis_points
    }
}

#[cfg(test)]
mod amms_config_tests {
    use anchor_lang::Discriminator;
    use crate::constants::ANCHOR_DISCRIMINATOR;
    use super::*;

    /// Tests the correct initialization of the `AmmsConfig` struct.
    #[test]
    fn test_amms_config_initialize() {
        let mut amms_config = AmmsConfig {
            bump: 0,
            id: 0,
            fee_authority: Pubkey::default(),
            providers_fee_rate_basis_points: 0,
            protocol_fee_rate_basis_points: 0,
        };

        let fee_authority = Pubkey::new_unique();
        let protocol_fee_rate_basis_points = 200;
        let providers_fee_rate_basis_points = 300;
        let id = 42u64;
        let bump = 42u8;

        amms_config.initialize(fee_authority, protocol_fee_rate_basis_points, providers_fee_rate_basis_points, id, bump).unwrap();

        assert_eq!(amms_config.bump, bump);
        assert_eq!(amms_config.id, id);
        assert_eq!(amms_config.fee_authority, fee_authority);
        assert_eq!(amms_config.protocol_fee_rate_basis_points, protocol_fee_rate_basis_points);
        assert_eq!(amms_config.providers_fee_rate_basis_points, providers_fee_rate_basis_points);

        assert_eq!(amms_config.bump(), bump);
        assert_eq!(amms_config.fee_authority().key(), fee_authority);
        assert_eq!(amms_config.protocol_fee_rate_basis_points(), protocol_fee_rate_basis_points);
        assert_eq!(amms_config.providers_fee_rate_basis_points(), providers_fee_rate_basis_points);

    }


    /// Tests the `update_fee_authority` method of the `AmmsConfig` struct.
    #[test]
    fn test_amms_config_update_fee_authority() {
        let mut amms_config = AmmsConfig {
            bump: 42,
            id: 42,
            fee_authority: Pubkey::default(),
            providers_fee_rate_basis_points: 300,
            protocol_fee_rate_basis_points: 200,
        };

        let new_fee_authority = Pubkey::new_unique();
        amms_config.update_fee_authority(new_fee_authority);
        assert_eq!(amms_config.fee_authority, new_fee_authority);
    }

    /// Tests the `update_providers_fee_rate` method of the `AmmsConfig` struct.
    #[test]
    fn test_amms_config_update_providers_fee_rate() {
        let mut amms_config = AmmsConfig {
            bump: 42,
            id: 42,
            fee_authority: Pubkey::default(),
            providers_fee_rate_basis_points: 300,
            protocol_fee_rate_basis_points: 200,
        };

        let new_providers_fee_rate = 234;
        amms_config.update_providers_fee_rate(new_providers_fee_rate).unwrap();
        assert_eq!(amms_config.providers_fee_rate_basis_points, new_providers_fee_rate);
        assert_eq!(amms_config.update_providers_fee_rate(9801).ok(), None);
    }

    /// Tests the `update_protocol_fee_rate` method of the `AmmsConfig` struct.
    #[test]
    fn test_amms_config_update_protocol_fee_rate() {
        let mut amms_config = AmmsConfig {
            bump: 42,
            id: 42,
            fee_authority: Pubkey::default(),
            providers_fee_rate_basis_points: 300,
            protocol_fee_rate_basis_points: 200,
        };

        let new_protocol_fee_rate = 234;
        amms_config.update_protocol_fee_rate(new_protocol_fee_rate).unwrap();
        assert_eq!(amms_config.protocol_fee_rate_basis_points, new_protocol_fee_rate);
        assert_eq!(amms_config.update_protocol_fee_rate(9701).ok(), None);
    }

    /// Tests `AmmsConfig` account data layout.
    #[test]
    fn test_amms_config_data_layout() {
        let fee_authority = Pubkey::new_unique();
        let bump = 42u8;
        let id = 42u64;
        let providers_fee_rate_basis_points: u16 = 200;
        let protocol_fee_rate_basis_points: u16 = 300;

        let mut data = [0u8; ANCHOR_DISCRIMINATOR + 45];
        let mut offset = 0;

        data[offset..offset + ANCHOR_DISCRIMINATOR].copy_from_slice(&AmmsConfig::discriminator()); offset += ANCHOR_DISCRIMINATOR;
        data[offset..offset + 1].copy_from_slice(&bump.to_le_bytes()); offset += 1;
        data[offset..offset + 8].copy_from_slice(&id.to_le_bytes()); offset += 8;
        data[offset..offset + 32].copy_from_slice(fee_authority.as_ref()); offset += 32;
        data[offset..offset + 2].copy_from_slice(&providers_fee_rate_basis_points.to_le_bytes()); offset += 2;
        data[offset..offset + 2].copy_from_slice(&protocol_fee_rate_basis_points.to_le_bytes()); offset += 2;

        assert_eq!(ANCHOR_DISCRIMINATOR + AmmsConfig::INIT_SPACE, offset);
        
        let deserialized_amms_config = AmmsConfig::try_deserialize(&mut data.as_slice()).unwrap();

        assert_eq!(deserialized_amms_config.bump, bump);
        assert_eq!(deserialized_amms_config.id, id);
        assert_eq!(deserialized_amms_config.fee_authority, fee_authority);
        assert_eq!(deserialized_amms_config.providers_fee_rate_basis_points, providers_fee_rate_basis_points);
        assert_eq!(deserialized_amms_config.protocol_fee_rate_basis_points, protocol_fee_rate_basis_points);

        let mut serialized_amms_config = Vec::new();
        deserialized_amms_config.try_serialize(&mut serialized_amms_config).unwrap();
        assert_eq!(serialized_amms_config.as_slice(), data.as_ref());
    }
}