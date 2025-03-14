use core::fmt;
use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
    str::FromStr,
};

use anyhow::Context;
use penumbra_num::Amount;
use penumbra_proto::client::v1alpha1 as pb_client;
use penumbra_proto::core::chain::v1alpha1 as pb_chain;

use penumbra_proto::view::v1alpha1 as pb_view;
use penumbra_proto::{DomainType, TypeUrl};
use serde::{Deserialize, Serialize};

pub mod change;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(
    try_from = "pb_chain::ChainParameters",
    into = "pb_chain::ChainParameters"
)]
pub struct ChainParameters {
    pub chain_id: String,
    pub epoch_duration: u64,

    pub unbonding_epochs: u64,
    /// The number of validators allowed in the consensus set (Active state).
    pub active_validator_limit: u64,
    /// The base reward rate, expressed in basis points of basis points
    pub base_reward_rate: u64,
    /// The penalty for slashing due to misbehavior, expressed in basis points squared (10^-8)
    pub slashing_penalty_misbehavior: u64,
    /// The penalty for slashing due to downtime, expressed in basis points squared (10^-8)
    pub slashing_penalty_downtime: u64,
    /// The number of blocks in the window to check for downtime.
    pub signed_blocks_window_len: u64,
    /// The maximum number of blocks in the window each validator can miss signing without slashing.
    pub missed_blocks_maximum: u64,

    /// Whether IBC (forming connections, processing IBC packets) is enabled.
    pub ibc_enabled: bool,
    /// Whether inbound ICS-20 transfers are enabled
    pub inbound_ics20_transfers_enabled: bool,
    /// Whether outbound ICS-20 transfers are enabled
    pub outbound_ics20_transfers_enabled: bool,

    /// The number of blocks during which a proposal is voted on.
    pub proposal_voting_blocks: u64,
    /// The deposit required to create a proposal.
    pub proposal_deposit_amount: Amount,
    /// The quorum required for a proposal to be considered valid, as a fraction of the total stake
    /// weight of the network.
    pub proposal_valid_quorum: Ratio,
    /// The threshold for a proposal to pass voting, as a ratio of "yes" votes over "no" votes.
    pub proposal_pass_threshold: Ratio,
    /// The threshold for a proposal to be slashed, as a ratio of "no" votes over all total votes.
    pub proposal_slash_threshold: Ratio,

    /// Whether DAO spend proposals are enabled.
    pub dao_spend_proposals_enabled: bool,
}

impl TypeUrl for ChainParameters {
    const TYPE_URL: &'static str = "/penumbra.core.chain.v1alpha1.ChainParameters";
}

impl DomainType for ChainParameters {
    type Proto = pb_chain::ChainParameters;
}

impl TryFrom<pb_chain::ChainParameters> for ChainParameters {
    type Error = anyhow::Error;

    fn try_from(msg: pb_chain::ChainParameters) -> anyhow::Result<Self> {
        Ok(ChainParameters {
            chain_id: msg.chain_id,
            epoch_duration: msg.epoch_duration,
            unbonding_epochs: msg.unbonding_epochs,
            active_validator_limit: msg.active_validator_limit,
            slashing_penalty_downtime: msg.slashing_penalty_downtime,
            slashing_penalty_misbehavior: msg.slashing_penalty_misbehavior,
            base_reward_rate: msg.base_reward_rate,
            missed_blocks_maximum: msg.missed_blocks_maximum,
            signed_blocks_window_len: msg.signed_blocks_window_len,
            ibc_enabled: msg.ibc_enabled,
            inbound_ics20_transfers_enabled: msg.inbound_ics20_transfers_enabled,
            outbound_ics20_transfers_enabled: msg.outbound_ics20_transfers_enabled,
            proposal_voting_blocks: msg.proposal_voting_blocks,
            proposal_deposit_amount: msg
                .proposal_deposit_amount
                .ok_or_else(|| anyhow::anyhow!("missing proposal_deposit_amount"))?
                .try_into()?,
            proposal_valid_quorum: msg
                .proposal_valid_quorum
                .parse()
                .context("couldn't parse proposal_valid_quorum")?,
            proposal_pass_threshold: msg
                .proposal_pass_threshold
                .parse()
                .context("couldn't parse proposal_pass_threshold")?,
            proposal_slash_threshold: msg
                .proposal_slash_threshold
                .parse()
                .context("couldn't parse proposal_slash_threshold")?,
            dao_spend_proposals_enabled: msg.dao_spend_proposals_enabled,
        })
    }
}

impl TryFrom<pb_view::ChainParametersResponse> for ChainParameters {
    type Error = anyhow::Error;

    fn try_from(response: pb_view::ChainParametersResponse) -> Result<Self, Self::Error> {
        response
            .parameters
            .ok_or_else(|| anyhow::anyhow!("empty ChainParametersResponse message"))?
            .try_into()
    }
}

impl TryFrom<pb_client::ChainParametersResponse> for ChainParameters {
    type Error = anyhow::Error;

    fn try_from(response: pb_client::ChainParametersResponse) -> Result<Self, Self::Error> {
        response
            .chain_parameters
            .ok_or_else(|| anyhow::anyhow!("empty ChainParametersResponse message"))?
            .try_into()
    }
}

impl From<ChainParameters> for pb_chain::ChainParameters {
    fn from(params: ChainParameters) -> Self {
        pb_chain::ChainParameters {
            chain_id: params.chain_id,
            epoch_duration: params.epoch_duration,
            unbonding_epochs: params.unbonding_epochs,
            active_validator_limit: params.active_validator_limit,
            signed_blocks_window_len: params.signed_blocks_window_len,
            missed_blocks_maximum: params.missed_blocks_maximum,
            slashing_penalty_downtime: params.slashing_penalty_downtime,
            slashing_penalty_misbehavior: params.slashing_penalty_misbehavior,
            base_reward_rate: params.base_reward_rate,
            ibc_enabled: params.ibc_enabled,
            inbound_ics20_transfers_enabled: params.inbound_ics20_transfers_enabled,
            outbound_ics20_transfers_enabled: params.outbound_ics20_transfers_enabled,
            proposal_voting_blocks: params.proposal_voting_blocks,
            proposal_deposit_amount: Some(params.proposal_deposit_amount.into()),
            proposal_valid_quorum: params.proposal_valid_quorum.to_string(),
            proposal_pass_threshold: params.proposal_pass_threshold.to_string(),
            proposal_slash_threshold: params.proposal_slash_threshold.to_string(),
            dao_spend_proposals_enabled: params.dao_spend_proposals_enabled,
        }
    }
}

// TODO: defaults are implemented here as well as in the
// `pd::main`
impl Default for ChainParameters {
    fn default() -> Self {
        Self {
            chain_id: String::new(),
            epoch_duration: 719,
            unbonding_epochs: 2,
            active_validator_limit: 80,
            // copied from cosmos hub
            signed_blocks_window_len: 10000,
            missed_blocks_maximum: 9500,
            // 1000 basis points = 10%
            slashing_penalty_misbehavior: 1000_0000,
            // 1 basis point = 0.01%
            slashing_penalty_downtime: 1_0000,
            // 3bps -> 11% return over 365 epochs
            base_reward_rate: 3_0000,
            ibc_enabled: true,
            inbound_ics20_transfers_enabled: true,
            outbound_ics20_transfers_enabled: true,
            // governance
            proposal_voting_blocks: 17_280, // 24 hours, at a 5 second block time
            proposal_deposit_amount: 10_000_000u64.into(), // 10,000,000 upenumbra = 10 penumbra
            // governance parameters copied from cosmos hub
            proposal_valid_quorum: Ratio::new(40, 100),
            proposal_pass_threshold: Ratio::new(50, 100),
            // slash threshold means if (no / no + yes + abstain) > slash_threshold, then proposal is slashed
            proposal_slash_threshold: Ratio::new(80, 100),
            dao_spend_proposals_enabled: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pb_chain::FmdParameters", into = "pb_chain::FmdParameters")]
pub struct FmdParameters {
    /// Bits of precision.
    pub precision_bits: u8,
    /// The block height at which these parameters became effective.
    pub as_of_block_height: u64,
}

impl TypeUrl for FmdParameters {
    const TYPE_URL: &'static str = "/penumbra.core.chain.v1alph1.FmdParameters";
}

impl DomainType for FmdParameters {
    type Proto = pb_chain::FmdParameters;
}

impl TryFrom<pb_chain::FmdParameters> for FmdParameters {
    type Error = anyhow::Error;

    fn try_from(msg: pb_chain::FmdParameters) -> Result<Self, Self::Error> {
        Ok(FmdParameters {
            precision_bits: msg.precision_bits.try_into()?,
            as_of_block_height: msg.as_of_block_height,
        })
    }
}

impl From<FmdParameters> for pb_chain::FmdParameters {
    fn from(params: FmdParameters) -> Self {
        pb_chain::FmdParameters {
            precision_bits: u32::from(params.precision_bits),
            as_of_block_height: params.as_of_block_height,
        }
    }
}

impl Default for FmdParameters {
    fn default() -> Self {
        Self {
            precision_bits: 0,
            as_of_block_height: 1,
        }
    }
}

/// This is a ratio of two `u64` values, intended to be used solely in governance parameters and
/// tallying. It only implements construction and comparison, not arithmetic, to reduce the trusted
/// codebase for governance.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pb_chain::Ratio", into = "pb_chain::Ratio")]
pub struct Ratio {
    numerator: u64,
    denominator: u64,
}

impl Display for Ratio {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}

impl FromStr for Ratio {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('/');
        let numerator = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("missing numerator"))?
            .parse()?;
        let denominator = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("missing denominator"))?
            .parse()?;
        if parts.next().is_some() {
            return Err(anyhow::anyhow!("too many parts"));
        }
        Ok(Ratio {
            numerator,
            denominator,
        })
    }
}

impl Ratio {
    pub fn new(numerator: u64, denominator: u64) -> Self {
        Self {
            numerator,
            denominator,
        }
    }
}

impl PartialEq for Ratio {
    fn eq(&self, other: &Self) -> bool {
        // Convert everything to `u128` to avoid overflow when multiplying
        u128::from(self.numerator) * u128::from(other.denominator)
            == u128::from(self.denominator) * u128::from(other.numerator)
    }
}

impl Eq for Ratio {}

impl PartialOrd for Ratio {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Ratio {
    fn cmp(&self, other: &Self) -> Ordering {
        // Convert everything to `u128` to avoid overflow when multiplying
        (u128::from(self.numerator) * u128::from(other.denominator))
            .cmp(&(u128::from(self.denominator) * u128::from(other.numerator)))
    }
}

impl From<Ratio> for pb_chain::Ratio {
    fn from(ratio: Ratio) -> Self {
        pb_chain::Ratio {
            numerator: ratio.numerator,
            denominator: ratio.denominator,
        }
    }
}

impl From<pb_chain::Ratio> for Ratio {
    fn from(msg: pb_chain::Ratio) -> Self {
        Ratio {
            numerator: msg.numerator,
            denominator: msg.denominator,
        }
    }
}
