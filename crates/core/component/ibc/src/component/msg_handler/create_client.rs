use anyhow::Result;
use async_trait::async_trait;
use ibc_types::{
    core::client::{events::CreateClient, msgs::MsgCreateClient, ClientId},
    lightclients::tendermint::client_type,
};
use penumbra_storage::StateWrite;

use crate::component::{
    client::{StateReadExt as _, StateWriteExt as _},
    client_counter::{ics02_validation, ClientCounter},
    MsgHandler,
};

#[async_trait]
impl MsgHandler for MsgCreateClient {
    async fn check_stateless(&self) -> Result<()> {
        client_state_is_tendermint(self)?;
        consensus_state_is_tendermint(self)?;

        Ok(())
    }

    // execute IBC CreateClient.
    //
    //  we compute the client's ID (a concatenation of a monotonically increasing integer, the
    //  number of clients on Penumbra, and the client type) and commit the following to our state:
    // - client type
    // - consensus state
    // - processed time and height
    async fn try_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        tracing::debug!(msg = ?self);
        let client_state =
            ics02_validation::get_tendermint_client_state(self.client_state.clone())?;

        // get the current client counter
        let id_counter = state.client_counter().await?;
        let client_id = ClientId::new(client_type(), id_counter.0)?;

        tracing::info!("creating client {:?}", client_id);

        let consensus_state =
            ics02_validation::get_tendermint_consensus_state(self.consensus_state.clone())?;

        // store the client data
        state.put_client(&client_id, client_state.clone());

        // store the genesis consensus state
        state
            .put_verified_consensus_state(
                client_state.latest_height(),
                client_id.clone(),
                consensus_state,
            )
            .await
            .unwrap();

        // increment client counter
        let counter = state.client_counter().await.unwrap_or(ClientCounter(0));
        state.put_client_counter(ClientCounter(counter.0 + 1));

        state.record(
            CreateClient {
                client_id: client_id.clone(),
                client_type: client_type(),
                consensus_height: client_state.latest_height(),
            }
            .into(),
        );
        Ok(())
    }
}
fn client_state_is_tendermint(msg: &MsgCreateClient) -> anyhow::Result<()> {
    if ics02_validation::is_tendermint_client_state(&msg.client_state) {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "MsgCreateClient: not a tendermint client state"
        ))
    }
}

fn consensus_state_is_tendermint(msg: &MsgCreateClient) -> anyhow::Result<()> {
    if ics02_validation::is_tendermint_consensus_state(&msg.consensus_state) {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "MsgCreateClient: not a tendermint consensus state"
        ))
    }
}
