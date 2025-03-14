use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use penumbra_chain::component::StateReadExt as _;
use penumbra_chain::TransactionContext;
use penumbra_component::ActionHandler;

use penumbra_proof_params::SWAPCLAIM_PROOF_VERIFICATION_KEY;
use penumbra_shielded_pool::component::{NoteManager, StateReadExt as _};
use penumbra_storage::{StateRead, StateWrite};

use crate::{component::StateReadExt, event, swap_claim::SwapClaim};

#[async_trait]
impl ActionHandler for SwapClaim {
    type CheckStatelessContext = TransactionContext;
    async fn check_stateless(&self, context: TransactionContext) -> Result<()> {
        self.proof
            .verify(
                &SWAPCLAIM_PROOF_VERIFICATION_KEY,
                context.anchor,
                self.body.nullifier,
                self.body.fee.clone(),
                self.body.output_data,
                self.body.output_1_commitment,
                self.body.output_2_commitment,
            )
            .context("a swap claim proof did not verify")?;

        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
        let swap_claim = self;

        // 1. Validate the epoch duration passed in the swap claim matches
        // what we know.
        let epoch_duration = state.get_epoch_duration().await?;
        let provided_epoch_duration = swap_claim.epoch_duration;
        if epoch_duration != provided_epoch_duration {
            return Err(anyhow::anyhow!(
                "provided epoch duration does not match chain epoch duration"
            ));
        }

        // 2. The stateful check *must* validate that the clearing
        // prices used in the proof are valid.
        let provided_output_height = swap_claim.body.output_data.height;
        let provided_trading_pair = swap_claim.body.output_data.trading_pair;
        let output_data = state
            .output_data(provided_output_height, provided_trading_pair)
            .await?
            // This check also ensures that the height for the swap is in the past, otherwise
            // the output data would not be present in the JMT.
            .ok_or_else(|| anyhow::anyhow!("output data not found"))?;

        if output_data != swap_claim.body.output_data {
            return Err(anyhow::anyhow!(
                "provided output data does not match chain output data"
            ));
        }

        // 3. Check that the nullifier hasn't been spent before.
        let spent_nullifier = self.body.nullifier;
        state.check_nullifier_unspent(spent_nullifier).await?;

        Ok(())
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        // Record the output notes in the state.
        let source = state.object_get("source").unwrap_or_default();

        state
            .add_rolled_up_payload(self.body.output_1_commitment)
            .await;
        state
            .add_rolled_up_payload(self.body.output_2_commitment)
            .await;

        state.spend_nullifier(self.body.nullifier, source).await;

        state.record(event::swap_claim(&self));

        Ok(())
    }
}
