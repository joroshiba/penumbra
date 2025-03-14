use anyhow::Result;
use penumbra_chain::component::StateReadExt as _;
use penumbra_chain::params::FmdParameters;
use penumbra_sct::component::StateReadExt as _;
use penumbra_storage::StateRead;
use penumbra_transaction::Transaction;

pub(super) async fn claimed_anchor_is_valid<S: StateRead>(
    state: S,
    transaction: &Transaction,
) -> Result<()> {
    state.check_claimed_anchor(transaction.anchor).await
}

pub(super) async fn fmd_parameters_valid<S: StateRead>(
    state: S,
    transaction: &Transaction,
) -> Result<()> {
    let previous_fmd_parameters = state
        .get_previous_fmd_parameters()
        .await
        .expect("chain params request must succeed");
    let current_fmd_parameters = state
        .get_current_fmd_parameters()
        .await
        .expect("chain params request must succeed");
    let height = state.get_block_height().await?;
    fmd_precision_within_grace_period(
        transaction,
        previous_fmd_parameters,
        current_fmd_parameters,
        height,
    )
}

const FMD_GRACE_PERIOD_BLOCKS: u64 = 10;

pub fn fmd_precision_within_grace_period(
    tx: &Transaction,
    previous_fmd_parameters: FmdParameters,
    current_fmd_parameters: FmdParameters,
    block_height: u64,
) -> anyhow::Result<()> {
    for clue in tx
        .transaction_body()
        .detection_data
        .unwrap_or_default()
        .fmd_clues
    {
        // Clue must be using the current `FmdParameters`, or be within
        // `FMD_GRACE_PERIOD_BLOCKS` of the previous `FmdParameters`.
        if clue.precision_bits() == current_fmd_parameters.precision_bits
            || (clue.precision_bits() == previous_fmd_parameters.precision_bits
                && block_height
                    < previous_fmd_parameters.as_of_block_height + FMD_GRACE_PERIOD_BLOCKS)
        {
            continue;
        } else {
            return Err(anyhow::anyhow!(
                "consensus rule violated: invalid clue precision",
            ));
        }
    }
    Ok(())
}
