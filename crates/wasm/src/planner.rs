use std::{
    collections::BTreeMap,
    fmt::{self, Debug, Formatter},
    mem,
};

use anyhow::{anyhow, Result};

use crate::note_record::SpendableNoteRecord;
use penumbra_asset::{asset::DenomMetadata, Balance, Value};
use penumbra_chain::params::{ChainParameters, FmdParameters};
use penumbra_dex::{swap::SwapPlaintext, swap::SwapPlan, swap_claim::SwapClaimPlan, TradingPair};
use penumbra_fee::Fee;
use penumbra_keys::{keys::AddressIndex, Address, FullViewingKey};
use penumbra_num::Amount;
use penumbra_shielded_pool::{Note, OutputPlan, SpendPlan};
use penumbra_stake::{IdentityKey, UndelegateClaimPlan};
use penumbra_tct as tct;
use penumbra_transaction::{
    action::{Proposal, ProposalSubmit, ProposalWithdraw, ValidatorVote, Vote},
    memo::MemoPlaintext,
    plan::{ActionPlan, DelegatorVotePlan, MemoPlan, TransactionPlan},
};

// use penumbra_view::{SpendableNoteRecord, ViewClient};
use rand_core::{CryptoRng, RngCore};
// use tracing::instrument;

/// A planner for a [`TransactionPlan`] that can fill in the required spends and change outputs upon
/// finalization to make a transaction balance.
pub struct Planner<R: RngCore + CryptoRng> {
    rng: R,
    balance: Balance,
    vote_intents: BTreeMap<u64, VoteIntent>,
    plan: TransactionPlan,
    // IMPORTANT: if you add more fields here, make sure to clear them when the planner is finished
}

#[derive(Debug, Clone)]
struct VoteIntent {
    start_block_height: u64,
    start_position: tct::Position,
    // rate_data: BTreeMap<IdentityKey, RateData>,
    vote: Vote,
}

impl<R: RngCore + CryptoRng> Debug for Planner<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Builder")
            .field("balance", &self.balance)
            .field("plan", &self.plan)
            .finish()
    }
}

impl<R: RngCore + CryptoRng> Planner<R> {
    /// Create a new planner.
    pub fn new(rng: R) -> Self {
        Self {
            rng,
            balance: Balance::default(),
            vote_intents: BTreeMap::default(),
            plan: TransactionPlan::default(),
        }
    }

    /// Get the current transaction balance of the planner.
    pub fn balance(&self) -> &Balance {
        &self.balance
    }

    // /// Get all the note requests necessary to fulfill the current [`Balance`].
    // pub fn notes_requests(
    //     &self,
    //     fvk: &FullViewingKey,
    //     source: AddressIndex,
    // ) -> (Vec<NotesRequest>, Vec<NotesForVotingRequest>) {
    //     (
    //         self.balance
    //             .required()
    //             .map(|Value { asset_id, amount }| NotesRequest {
    //                 account_id: Some(fvk.hash().into()),
    //                 asset_id: Some(asset_id.into()),
    //                 address_index: Some(source.into()),
    //                 amount_to_spend: amount.into(),
    //                 include_spent: false,
    //                 ..Default::default()
    //             })
    //             .collect(),
    //         self.vote_intents
    //             .iter()
    //             .map(
    //                 |(
    //                      _proposal, // The request only cares about the start block height
    //                      VoteIntent {
    //                          start_block_height, ..
    //                      },
    //                  )| NotesForVotingRequest {
    //                     account_id: Some(fvk.hash().into()),
    //                     votable_at_height: *start_block_height,
    //                     address_index: Some(source.into()),
    //                     ..Default::default()
    //                 },
    //             )
    //             .collect(),
    //     )
    // }

    /// Set the expiry height for the transaction plan.
    pub fn expiry_height(&mut self, expiry_height: u64) -> &mut Self {
        self.plan.expiry_height = expiry_height;
        self
    }

    /// Set a memo for this transaction plan.
    ///
    /// Errors if the memo is too long.
    pub fn memo(&mut self, memo: MemoPlaintext) -> anyhow::Result<&mut Self> {
        self.plan.memo_plan = Some(MemoPlan::new(&mut self.rng, memo)?);
        Ok(self)
    }

    /// Add a fee to the transaction plan.
    ///
    /// This function should be called once.
    pub fn fee(&mut self, fee: Fee) -> &mut Self {
        self.balance += fee.0;
        self.plan.fee = fee;
        self
    }

    /// Spend a specific positioned note in the transaction.
    ///
    /// If you don't use this method to specify spends, they will be filled in automatically from
    /// the view service when the plan is [`finish`](Planner::finish)ed.
    pub fn spend(&mut self, note: Note, position: tct::Position) -> &mut Self {
        let spend = SpendPlan::new(&mut self.rng, note, position).into();
        self.action(spend);
        self
    }

    /// Perform a swap claim based on an input swap NFT with a pre-paid fee.
    pub fn swap_claim(&mut self, plan: SwapClaimPlan) -> &mut Self {
        // Nothing needs to be spent, since the fee is pre-paid and the
        // swap NFT will be automatically consumed when the SwapClaim action
        // is processed by the validators.
        // TODO: need to set the intended fee so the tx actually balances,
        // otherwise the planner will create an output
        self.action(plan.into());
        self
    }

    /// Perform a swap based on input notes in the transaction.
    pub fn swap(
        &mut self,
        input_value: Value,
        into_denom: DenomMetadata,
        swap_claim_fee: Fee,
        claim_address: Address,
    ) -> Result<&mut Self> {
        // Determine the canonical order for the assets being swapped.
        // This will determine whether the input amount is assigned to delta_1 or delta_2.
        let trading_pair = TradingPair::new(input_value.asset_id, into_denom.id());

        // If `trading_pair.asset_1` is the input asset, then `delta_1` is the input amount,
        // and `delta_2` is 0.
        //
        // Otherwise, `delta_1` is 0, and `delta_2` is the input amount.
        let (delta_1, delta_2) = if trading_pair.asset_1() == input_value.asset_id {
            (input_value.amount, 0u64.into())
        } else {
            (0u64.into(), input_value.amount)
        };

        // If there is no input, then there is no swap.
        if delta_1 == Amount::zero() && delta_2 == Amount::zero() {
            return Err(anyhow!("No input value for swap"));
        }

        // Create the `SwapPlaintext` representing the swap to be performed:
        let swap_plaintext = SwapPlaintext::new(
            &mut self.rng,
            trading_pair,
            delta_1,
            delta_2,
            swap_claim_fee,
            claim_address,
        );

        let swap = SwapPlan::new(&mut self.rng, swap_plaintext).into();
        self.action(swap);

        Ok(self)
    }

    /// Add an output note from this transaction.
    ///
    /// Any unused output value will be redirected back to the originating address as change notes
    /// when the plan is [`finish`](Builder::finish)ed.
    pub fn output(&mut self, value: Value, address: Address) -> &mut Self {
        let output = OutputPlan::new(&mut self.rng, value, address).into();
        self.action(output);
        self
    }

    // /// Add a delegation to this transaction.
    // ///
    // /// If you don't specify spends or outputs as well, they will be filled in automatically.
    // pub fn delegate(&mut self, unbonded_amount: u64, rate_data: RateData) -> &mut Self {
    //     let delegation = rate_data.build_delegate(unbonded_amount).into();
    //     self.action(delegation);
    //     self
    // }

    // /// Add an undelegation to this transaction.
    // ///
    // /// TODO: can we put the chain parameters into the planner at the start, so we can compute end_epoch_index?
    // pub fn undelegate(
    //     &mut self,
    //     delegation_amount: Amount,
    //     rate_data: RateData,
    //     end_epoch_index: u64,
    // ) -> &mut Self {
    //     let undelegation = rate_data
    //         .build_undelegate(delegation_amount, end_epoch_index)
    //         .into();
    //     self.action(undelegation);
    //     self
    // }

    /// Add an undelegate claim to this transaction.
    pub fn undelegate_claim(&mut self, claim_plan: UndelegateClaimPlan) -> &mut Self {
        self.action(ActionPlan::UndelegateClaim(claim_plan));
        self
    }

    // /// Upload a validator definition in this transaction.
    // pub fn validator_definition(&mut self, new_validator: validator::Definition) -> &mut Self {
    //     self.action(ActionPlan::ValidatorDefinition(new_validator.into()));
    //     self
    // }

    /// Submit a new governance proposal in this transaction.
    pub fn proposal_submit(&mut self, proposal: Proposal, deposit_amount: Amount) -> &mut Self {
        self.action(ActionPlan::ProposalSubmit(ProposalSubmit {
            proposal,
            deposit_amount,
        }));
        self
    }

    /// Withdraw a governance proposal in this transaction.
    pub fn proposal_withdraw(&mut self, proposal: u64, reason: String) -> &mut Self {
        self.action(ActionPlan::ProposalWithdraw(ProposalWithdraw {
            proposal,
            reason,
        }));
        self
    }

    /// Claim a governance proposal deposit in this transaction.
    // pub fn proposal_deposit_claim(
    //     &mut self,
    //     proposal: u64,
    //     deposit_amount: Amount,
    //     outcome: Outcome<()>,
    // ) -> &mut Self {
    //     self.action(ActionPlan::ProposalDepositClaim(ProposalDepositClaim {
    //         proposal,
    //         deposit_amount,
    //         outcome,
    //     }));
    //     self
    // }

    /// Cast a validator vote in this transaction.
    pub fn validator_vote(&mut self, vote: ValidatorVote) -> &mut Self {
        self.action(ActionPlan::ValidatorVote(vote));
        self
    }

    // /// Vote with all possible vote weight on a given proposal.
    // pub fn delegator_vote(
    //     &mut self,
    //     proposal: u64,
    //     start_block_height: u64,
    //     start_position: tct::Position,
    //     start_rate_data: BTreeMap<IdentityKey, RateData>,
    //     vote: Vote,
    // ) -> &mut Self {
    //     self.vote_intents.insert(
    //         proposal,
    //         VoteIntent {
    //             start_position,
    //             start_block_height,
    //             vote,
    //             rate_data: start_rate_data,
    //         },
    //     );
    //     self
    // }

    /// Vote with a specific positioned note in the transaction.
    ///
    /// If you don't use this method to specify votes, they will be filled in automatically from the
    /// implied voting intent by [`vote`](Planner::vote) when the plan is
    /// [`finish`](Planner::finish)ed.
    pub fn delegator_vote_precise(
        &mut self,
        proposal: u64,
        start_position: tct::Position,
        vote: Vote,
        note: Note,
        position: tct::Position,
        unbonded_amount: Amount,
    ) -> &mut Self {
        let vote = DelegatorVotePlan::new(
            &mut self.rng,
            proposal,
            start_position,
            vote,
            note,
            position,
            unbonded_amount,
        )
        .into();
        self.action(vote);
        self
    }

    fn action(&mut self, action: ActionPlan) -> &mut Self {
        // Track the contribution of the action to the transaction's balance
        self.balance += action.balance();

        // Add the action to the plan
        self.plan.actions.push(action);
        self
    }

    /// Add spends and change outputs as required to balance the transaction, using the view service
    /// provided to supply the notes and other information.
    ///
    /// Clears the contents of the planner, which can be re-used.

    /// Add spends and change outputs as required to balance the transaction, using the spendable
    /// notes provided. It is the caller's responsibility to ensure that the notes are the result of
    /// collected responses to the requests generated by an immediately preceding call to
    /// [`Planner::note_requests`].
    ///
    /// Clears the contents of the planner, which can be re-used.
    pub fn plan_with_spendable_and_votable_notes(
        &mut self,
        chain_params: &ChainParameters,
        fmd_params: &FmdParameters,
        fvk: &FullViewingKey,
        source: AddressIndex,
        spendable_notes: Vec<SpendableNoteRecord>,
        _votable_notes: Vec<Vec<(SpendableNoteRecord, IdentityKey)>>,
    ) -> anyhow::Result<TransactionPlan> {
        // Fill in the chain id based on the view service
        self.plan.chain_id = chain_params.chain_id.clone();

        // Add the required spends to the planner
        for record in spendable_notes {
            self.spend(record.note, record.position);
        }

        // Add the required votes to the planner
        // for (
        //     records,
        //     (
        //         proposal,
        //         VoteIntent {
        //             start_position,
        //             vote,
        //             rate_data,
        //             ..
        //         },
        //     ),
        // ) in votable_notes
        //     .into_iter()
        //     .chain(std::iter::repeat(vec![])) // Chain with infinite repeating no notes, so the zip doesn't stop early
        //     .zip(mem::take(&mut self.vote_intents).into_iter())
        // {
        //     if records.is_empty() {
        //         // If there are no notes to vote with, return an error, because otherwise the user
        //         // would compose a transaction that would not satisfy their intention, and would
        //         // silently eat the fee.
        //         return Err(anyhow!(
        //             "can't vote on proposal {} because no delegation notes were staked when voting started",
        //             proposal
        //         ));
        //     }
        //
        //     for (record, identity_key) in records {
        //         // Vote with precisely this note on the proposal, computing the correct exchange
        //         // rate for self-minted vote receipt tokens using the exchange rate of the validator
        //         // at voting start time
        //         let unbonded_amount = rate_data
        //             .get(&identity_key)
        //             .ok_or_else(|| anyhow!("missing rate data for note"))?
        //             .unbonded_amount(record.note.amount().into())
        //             .into();
        //
        //         // If the delegation token is unspent, "roll it over" by spending it (this will
        //         // result in change sent back to us). This unlinks nullifiers used for voting on
        //         // multiple non-overlapping proposals, increasing privacy.
        //         if record.height_spent.is_none() {
        //             self.spend(record.note.clone(), record.position);
        //         }
        //
        //         self.delegator_vote_precise(
        //             proposal,
        //             start_position,
        //             vote,
        //             record.note,
        //             record.position,
        //             unbonded_amount,
        //         );
        //     }
        // }

        // For any remaining provided balance, make a single change note for each
        let self_address = fvk.incoming().payment_address(source).0;

        for value in self.balance.provided().collect::<Vec<_>>() {
            self.output(value, self_address);
        }

        // If there are outputs, we check that a memo has been added. If not, we add a default memo.
        if self.plan.num_outputs() > 0 && self.plan.memo_plan.is_none() {
            self.memo(MemoPlaintext::default())
                .expect("empty string is a valid memo");
        } else if self.plan.num_outputs() == 0 && self.plan.memo_plan.is_some() {
            anyhow::bail!("if no outputs, no memo should be added");
        }

        // Add clue plans for `Output`s.
        let precision_bits = fmd_params.precision_bits;
        self.plan
            .add_all_clue_plans(&mut self.rng, precision_bits.into());

        // Now the transaction should be fully balanced, unless we didn't have enough to spend
        if !self.balance.is_zero() {
            anyhow::bail!(
                "balance is non-zero after attempting to balance transaction: {:?}",
                self.balance
            );
        }

        // Clear the planner and pull out the plan to return
        self.balance = Balance::zero();
        self.vote_intents = BTreeMap::new();
        let plan = mem::take(&mut self.plan);

        Ok(plan)
    }
}
