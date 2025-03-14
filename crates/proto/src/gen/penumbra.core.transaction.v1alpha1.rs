/// A Penumbra transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Transaction {
    #[prost(message, optional, tag = "1")]
    pub body: ::core::option::Option<TransactionBody>,
    /// The binding signature is stored separately from the transaction body that it signs.
    #[prost(bytes = "bytes", tag = "2")]
    pub binding_sig: ::prost::bytes::Bytes,
    /// The root of some previous state of the state commitment tree, used as an anchor for all
    /// ZK state transition proofs.
    #[prost(message, optional, tag = "3")]
    pub anchor: ::core::option::Option<super::super::crypto::v1alpha1::MerkleRoot>,
}
/// A transaction ID, the Sha256 hash of a transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Id {
    #[prost(bytes = "bytes", tag = "1")]
    pub hash: ::prost::bytes::Bytes,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EffectHash {
    #[prost(bytes = "bytes", tag = "1")]
    pub inner: ::prost::bytes::Bytes,
}
/// The body of a transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionBody {
    /// A list of actions (state changes) performed by this transaction.
    #[prost(message, repeated, tag = "1")]
    pub actions: ::prost::alloc::vec::Vec<Action>,
    /// Parameters determining if a transaction should be accepted by this chain.
    #[prost(message, optional, tag = "2")]
    pub transaction_parameters: ::core::option::Option<TransactionParameters>,
    /// The transaction fee.
    #[prost(message, optional, tag = "3")]
    pub fee: ::core::option::Option<super::super::crypto::v1alpha1::Fee>,
    /// Detection data for use with Fuzzy Message Detection
    #[prost(message, optional, tag = "4")]
    pub detection_data: ::core::option::Option<DetectionData>,
    /// Sub-message containing memo ciphertext if a memo was added to the transaction.
    #[prost(message, optional, tag = "5")]
    pub memo_data: ::core::option::Option<MemoData>,
}
/// Represents the encrypted memo data.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MemoData {
    /// The encrypted data. It will only be populated if there are
    /// outputs in the actions of the transaction. 528 bytes.
    #[prost(bytes = "bytes", optional, tag = "1")]
    pub encrypted_memo: ::core::option::Option<::prost::bytes::Bytes>,
}
/// The parameters determining if a transaction should be accepted by the chain.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionParameters {
    /// The maximum height that this transaction can be included in the chain.
    ///
    /// If zero, there is no maximum.
    #[prost(uint64, tag = "1")]
    pub expiry_height: u64,
    /// The chain this transaction is intended for.  Including this prevents
    /// replaying a transaction on one chain onto a different chain.
    #[prost(string, tag = "2")]
    pub chain_id: ::prost::alloc::string::String,
}
/// Detection data used by a detection server performing Fuzzy Message Detection.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DetectionData {
    /// A list of clues for use with Fuzzy Message Detection.
    #[prost(message, repeated, tag = "4")]
    pub fmd_clues: ::prost::alloc::vec::Vec<super::super::crypto::v1alpha1::Clue>,
}
/// A state change performed by a transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Action {
    #[prost(
        oneof = "action::Action",
        tags = "1, 2, 3, 4, 16, 17, 18, 19, 20, 21, 22, 30, 31, 32, 34, 40, 41, 42, 50, 51, 52, 200"
    )]
    pub action: ::core::option::Option<action::Action>,
}
/// Nested message and enum types in `Action`.
pub mod action {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Action {
        #[prost(message, tag = "1")]
        Spend(super::Spend),
        #[prost(message, tag = "2")]
        Output(super::Output),
        #[prost(message, tag = "3")]
        Swap(super::super::super::dex::v1alpha1::Swap),
        #[prost(message, tag = "4")]
        SwapClaim(super::super::super::dex::v1alpha1::SwapClaim),
        #[prost(message, tag = "16")]
        ValidatorDefinition(super::super::super::stake::v1alpha1::ValidatorDefinition),
        #[prost(message, tag = "17")]
        IbcAction(super::super::super::ibc::v1alpha1::IbcAction),
        /// Governance:
        #[prost(message, tag = "18")]
        ProposalSubmit(super::super::super::governance::v1alpha1::ProposalSubmit),
        #[prost(message, tag = "19")]
        ProposalWithdraw(super::super::super::governance::v1alpha1::ProposalWithdraw),
        #[prost(message, tag = "20")]
        ValidatorVote(super::super::super::governance::v1alpha1::ValidatorVote),
        #[prost(message, tag = "21")]
        DelegatorVote(super::super::super::governance::v1alpha1::DelegatorVote),
        #[prost(message, tag = "22")]
        ProposalDepositClaim(
            super::super::super::governance::v1alpha1::ProposalDepositClaim,
        ),
        /// Positions
        #[prost(message, tag = "30")]
        PositionOpen(super::super::super::dex::v1alpha1::PositionOpen),
        #[prost(message, tag = "31")]
        PositionClose(super::super::super::dex::v1alpha1::PositionClose),
        #[prost(message, tag = "32")]
        PositionWithdraw(super::super::super::dex::v1alpha1::PositionWithdraw),
        #[prost(message, tag = "34")]
        PositionRewardClaim(super::super::super::dex::v1alpha1::PositionRewardClaim),
        /// (un)delegation
        #[prost(message, tag = "40")]
        Delegate(super::super::super::stake::v1alpha1::Delegate),
        #[prost(message, tag = "41")]
        Undelegate(super::super::super::stake::v1alpha1::Undelegate),
        #[prost(message, tag = "42")]
        UndelegateClaim(super::super::super::stake::v1alpha1::UndelegateClaim),
        /// DAO
        #[prost(message, tag = "50")]
        DaoSpend(super::super::super::governance::v1alpha1::DaoSpend),
        #[prost(message, tag = "51")]
        DaoOutput(super::super::super::governance::v1alpha1::DaoOutput),
        #[prost(message, tag = "52")]
        DaoDeposit(super::super::super::governance::v1alpha1::DaoDeposit),
        #[prost(message, tag = "200")]
        Ics20Withdrawal(super::super::super::ibc::v1alpha1::Ics20Withdrawal),
    }
}
/// A transaction perspective is a bundle of key material and commitment openings
/// that allow generating a view of a transaction from that perspective.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionPerspective {
    #[prost(message, repeated, tag = "1")]
    pub payload_keys: ::prost::alloc::vec::Vec<PayloadKeyWithCommitment>,
    #[prost(message, repeated, tag = "2")]
    pub spend_nullifiers: ::prost::alloc::vec::Vec<NullifierWithNote>,
    /// The openings of note commitments referred to in the transaction
    /// but not included in the transaction.
    #[prost(message, repeated, tag = "3")]
    pub advice_notes: ::prost::alloc::vec::Vec<super::super::crypto::v1alpha1::Note>,
    /// Any relevant address views.
    #[prost(message, repeated, tag = "4")]
    pub address_views: ::prost::alloc::vec::Vec<
        super::super::crypto::v1alpha1::AddressView,
    >,
    /// Any relevant denoms for viewed assets.
    #[prost(message, repeated, tag = "5")]
    pub denoms: ::prost::alloc::vec::Vec<super::super::crypto::v1alpha1::DenomMetadata>,
    /// The transaction ID associated with this TransactionPerspective
    #[prost(message, optional, tag = "6")]
    pub transaction_id: ::core::option::Option<Id>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PayloadKey {
    #[prost(bytes = "bytes", tag = "1")]
    pub inner: ::prost::bytes::Bytes,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PayloadKeyWithCommitment {
    #[prost(message, optional, tag = "1")]
    pub payload_key: ::core::option::Option<PayloadKey>,
    #[prost(message, optional, tag = "2")]
    pub commitment: ::core::option::Option<
        super::super::crypto::v1alpha1::StateCommitment,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NullifierWithNote {
    #[prost(message, optional, tag = "1")]
    pub nullifier: ::core::option::Option<super::super::crypto::v1alpha1::Nullifier>,
    #[prost(message, optional, tag = "2")]
    pub note: ::core::option::Option<super::super::crypto::v1alpha1::Note>,
}
/// View of a Penumbra transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionView {
    /// View of the transaction body
    #[prost(message, optional, tag = "1")]
    pub body_view: ::core::option::Option<TransactionBodyView>,
    /// The binding signature is stored separately from the transaction body that it signs.
    #[prost(bytes = "bytes", tag = "2")]
    pub binding_sig: ::prost::bytes::Bytes,
    /// The root of some previous state of the state commitment tree, used as an anchor for all
    /// ZK state transition proofs.
    #[prost(message, optional, tag = "3")]
    pub anchor: ::core::option::Option<super::super::crypto::v1alpha1::MerkleRoot>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionBodyView {
    /// A list views into of actions (state changes) performed by this transaction.
    #[prost(message, repeated, tag = "1")]
    pub action_views: ::prost::alloc::vec::Vec<ActionView>,
    /// Transaction parameters.
    #[prost(message, optional, tag = "2")]
    pub transaction_parameters: ::core::option::Option<TransactionParameters>,
    /// The transaction fee.
    #[prost(message, optional, tag = "3")]
    pub fee: ::core::option::Option<super::super::crypto::v1alpha1::Fee>,
    /// The detection data in this transaction, only populated if
    /// there are outputs in the actions of this transaction.
    #[prost(message, optional, tag = "4")]
    pub detection_data: ::core::option::Option<DetectionData>,
    /// An optional view of a transaction memo. It will only be populated if there are
    /// outputs in the actions of this transaction.
    #[prost(message, optional, tag = "5")]
    pub memo_view: ::core::option::Option<MemoView>,
}
/// A view of a specific state change action performed by a transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ActionView {
    #[prost(
        oneof = "action_view::ActionView",
        tags = "1, 2, 3, 4, 16, 17, 18, 19, 20, 21, 22, 30, 31, 32, 34, 41, 42, 50, 51, 52, 43, 200"
    )]
    pub action_view: ::core::option::Option<action_view::ActionView>,
}
/// Nested message and enum types in `ActionView`.
pub mod action_view {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum ActionView {
        /// Action types with visible/opaque variants
        #[prost(message, tag = "1")]
        Spend(super::SpendView),
        #[prost(message, tag = "2")]
        Output(super::OutputView),
        #[prost(message, tag = "3")]
        Swap(super::super::super::dex::v1alpha1::SwapView),
        #[prost(message, tag = "4")]
        SwapClaim(super::super::super::dex::v1alpha1::SwapClaimView),
        /// Action types without visible/opaque variants
        #[prost(message, tag = "16")]
        ValidatorDefinition(super::super::super::stake::v1alpha1::ValidatorDefinition),
        #[prost(message, tag = "17")]
        IbcAction(super::super::super::ibc::v1alpha1::IbcAction),
        /// Governance:
        #[prost(message, tag = "18")]
        ProposalSubmit(super::super::super::governance::v1alpha1::ProposalSubmit),
        #[prost(message, tag = "19")]
        ProposalWithdraw(super::super::super::governance::v1alpha1::ProposalWithdraw),
        #[prost(message, tag = "20")]
        ValidatorVote(super::super::super::governance::v1alpha1::ValidatorVote),
        #[prost(message, tag = "21")]
        DelegatorVote(super::DelegatorVoteView),
        #[prost(message, tag = "22")]
        ProposalDepositClaim(
            super::super::super::governance::v1alpha1::ProposalDepositClaim,
        ),
        #[prost(message, tag = "30")]
        PositionOpen(super::super::super::dex::v1alpha1::PositionOpen),
        #[prost(message, tag = "31")]
        PositionClose(super::super::super::dex::v1alpha1::PositionClose),
        #[prost(message, tag = "32")]
        PositionWithdraw(super::super::super::dex::v1alpha1::PositionWithdraw),
        #[prost(message, tag = "34")]
        PositionRewardClaim(super::super::super::dex::v1alpha1::PositionRewardClaim),
        #[prost(message, tag = "41")]
        Delegate(super::super::super::stake::v1alpha1::Delegate),
        #[prost(message, tag = "42")]
        Undelegate(super::super::super::stake::v1alpha1::Undelegate),
        /// DAO
        #[prost(message, tag = "50")]
        DaoSpend(super::super::super::governance::v1alpha1::DaoSpend),
        #[prost(message, tag = "51")]
        DaoOutput(super::super::super::governance::v1alpha1::DaoOutput),
        #[prost(message, tag = "52")]
        DaoDeposit(super::super::super::governance::v1alpha1::DaoDeposit),
        /// TODO: we have no way to recover the opening of the undelegate_claim's
        /// balance commitment, and can only infer the value from looking at the rest
        /// of the transaction. is that fine?
        #[prost(message, tag = "43")]
        UndelegateClaim(super::super::super::stake::v1alpha1::UndelegateClaim),
        #[prost(message, tag = "200")]
        Ics20Withdrawal(super::super::super::ibc::v1alpha1::Ics20Withdrawal),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpendView {
    #[prost(oneof = "spend_view::SpendView", tags = "1, 2")]
    pub spend_view: ::core::option::Option<spend_view::SpendView>,
}
/// Nested message and enum types in `SpendView`.
pub mod spend_view {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Visible {
        #[prost(message, optional, tag = "1")]
        pub spend: ::core::option::Option<super::Spend>,
        #[prost(message, optional, tag = "2")]
        pub note: ::core::option::Option<
            super::super::super::crypto::v1alpha1::NoteView,
        >,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Opaque {
        #[prost(message, optional, tag = "1")]
        pub spend: ::core::option::Option<super::Spend>,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum SpendView {
        #[prost(message, tag = "1")]
        Visible(Visible),
        #[prost(message, tag = "2")]
        Opaque(Opaque),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelegatorVoteView {
    #[prost(oneof = "delegator_vote_view::DelegatorVote", tags = "1, 2")]
    pub delegator_vote: ::core::option::Option<delegator_vote_view::DelegatorVote>,
}
/// Nested message and enum types in `DelegatorVoteView`.
pub mod delegator_vote_view {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Visible {
        #[prost(message, optional, tag = "1")]
        pub delegator_vote: ::core::option::Option<
            super::super::super::governance::v1alpha1::DelegatorVote,
        >,
        #[prost(message, optional, tag = "2")]
        pub note: ::core::option::Option<
            super::super::super::crypto::v1alpha1::NoteView,
        >,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Opaque {
        #[prost(message, optional, tag = "1")]
        pub delegator_vote: ::core::option::Option<
            super::super::super::governance::v1alpha1::DelegatorVote,
        >,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum DelegatorVote {
        #[prost(message, tag = "1")]
        Visible(Visible),
        #[prost(message, tag = "2")]
        Opaque(Opaque),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OutputView {
    #[prost(oneof = "output_view::OutputView", tags = "1, 2")]
    pub output_view: ::core::option::Option<output_view::OutputView>,
}
/// Nested message and enum types in `OutputView`.
pub mod output_view {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Visible {
        #[prost(message, optional, tag = "1")]
        pub output: ::core::option::Option<super::Output>,
        #[prost(message, optional, tag = "2")]
        pub note: ::core::option::Option<
            super::super::super::crypto::v1alpha1::NoteView,
        >,
        #[prost(message, optional, tag = "3")]
        pub payload_key: ::core::option::Option<super::PayloadKey>,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Opaque {
        #[prost(message, optional, tag = "1")]
        pub output: ::core::option::Option<super::Output>,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum OutputView {
        #[prost(message, tag = "1")]
        Visible(Visible),
        #[prost(message, tag = "2")]
        Opaque(Opaque),
    }
}
/// Spends a shielded note.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Spend {
    /// The effecting data of the spend.
    #[prost(message, optional, tag = "1")]
    pub body: ::core::option::Option<SpendBody>,
    /// The authorizing signature for the spend.
    #[prost(message, optional, tag = "2")]
    pub auth_sig: ::core::option::Option<
        super::super::crypto::v1alpha1::SpendAuthSignature,
    >,
    /// The proof that the spend is well-formed is authorizing data.
    #[prost(message, optional, tag = "3")]
    pub proof: ::core::option::Option<super::super::crypto::v1alpha1::ZkSpendProof>,
}
/// The body of a spend description, containing only the effecting data
/// describing changes to the ledger, and not the authorizing data that allows
/// those changes to be performed.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpendBody {
    /// A commitment to the value of the input note.
    #[prost(message, optional, tag = "1")]
    pub balance_commitment: ::core::option::Option<
        super::super::crypto::v1alpha1::BalanceCommitment,
    >,
    /// The nullifier of the input note.
    #[prost(bytes = "bytes", tag = "3")]
    pub nullifier: ::prost::bytes::Bytes,
    /// The randomized validating key for the spend authorization signature.
    #[prost(bytes = "bytes", tag = "4")]
    pub rk: ::prost::bytes::Bytes,
}
/// Creates a new shielded note.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Output {
    /// The effecting data for the output.
    #[prost(message, optional, tag = "1")]
    pub body: ::core::option::Option<OutputBody>,
    /// The output proof is authorizing data.
    #[prost(message, optional, tag = "2")]
    pub proof: ::core::option::Option<super::super::crypto::v1alpha1::ZkOutputProof>,
}
/// The body of an output description, containing only the effecting data
/// describing changes to the ledger, and not the authorizing data that allows
/// those changes to be performed.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OutputBody {
    /// The minimal data required to scan and process the new output note.
    #[prost(message, optional, tag = "1")]
    pub note_payload: ::core::option::Option<
        super::super::crypto::v1alpha1::NotePayload,
    >,
    /// A commitment to the value of the output note. 32 bytes.
    #[prost(message, optional, tag = "2")]
    pub balance_commitment: ::core::option::Option<
        super::super::crypto::v1alpha1::BalanceCommitment,
    >,
    /// An encrypted key for decrypting the memo.
    #[prost(bytes = "bytes", tag = "3")]
    pub wrapped_memo_key: ::prost::bytes::Bytes,
    /// The key material used for note encryption, wrapped in encryption to the
    /// sender's outgoing viewing key. 80 bytes.
    #[prost(bytes = "bytes", tag = "4")]
    pub ovk_wrapped_key: ::prost::bytes::Bytes,
}
/// The data required to authorize a transaction plan.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AuthorizationData {
    /// The computed auth hash for the approved transaction plan.
    #[prost(message, optional, tag = "1")]
    pub effect_hash: ::core::option::Option<super::super::crypto::v1alpha1::EffectHash>,
    /// The required spend authorizations, returned in the same order as the
    /// Spend actions in the original request.
    #[prost(message, repeated, tag = "2")]
    pub spend_auths: ::prost::alloc::vec::Vec<
        super::super::crypto::v1alpha1::SpendAuthSignature,
    >,
    /// The required delegator vote authorizations, returned in the same order as the
    /// DelegatorVote actions in the original request.
    #[prost(message, repeated, tag = "3")]
    pub delegator_vote_auths: ::prost::alloc::vec::Vec<
        super::super::crypto::v1alpha1::SpendAuthSignature,
    >,
}
/// The data required for proving when building a transaction from a plan.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WitnessData {
    /// The anchor for the state transition proofs.
    #[prost(message, optional, tag = "1")]
    pub anchor: ::core::option::Option<super::super::crypto::v1alpha1::MerkleRoot>,
    /// The auth paths for the notes the transaction spends, in the
    /// same order as the spends in the transaction plan.
    #[prost(message, repeated, tag = "2")]
    pub state_commitment_proofs: ::prost::alloc::vec::Vec<
        super::super::crypto::v1alpha1::StateCommitmentProof,
    >,
}
/// Describes a planned transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionPlan {
    #[prost(message, repeated, tag = "1")]
    pub actions: ::prost::alloc::vec::Vec<ActionPlan>,
    #[prost(uint64, tag = "2")]
    pub expiry_height: u64,
    #[prost(string, tag = "3")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "4")]
    pub fee: ::core::option::Option<super::super::crypto::v1alpha1::Fee>,
    #[prost(message, repeated, tag = "5")]
    pub clue_plans: ::prost::alloc::vec::Vec<CluePlan>,
    #[prost(message, optional, tag = "6")]
    pub memo_plan: ::core::option::Option<MemoPlan>,
}
/// Describes a planned transaction action.
///
/// Some transaction Actions don't have any private data and are treated as being plans
/// themselves.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ActionPlan {
    #[prost(
        oneof = "action_plan::Action",
        tags = "1, 2, 3, 4, 16, 17, 18, 19, 20, 21, 22, 23, 30, 31, 32, 34, 40, 41, 42, 50, 51, 52"
    )]
    pub action: ::core::option::Option<action_plan::Action>,
}
/// Nested message and enum types in `ActionPlan`.
pub mod action_plan {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Action {
        #[prost(message, tag = "1")]
        Spend(super::SpendPlan),
        #[prost(message, tag = "2")]
        Output(super::OutputPlan),
        #[prost(message, tag = "3")]
        Swap(super::super::super::dex::v1alpha1::SwapPlan),
        #[prost(message, tag = "4")]
        SwapClaim(super::super::super::dex::v1alpha1::SwapClaimPlan),
        /// This is just a message relayed to the chain.
        #[prost(message, tag = "16")]
        ValidatorDefinition(super::super::super::stake::v1alpha1::ValidatorDefinition),
        /// This is just a message relayed to the chain.
        #[prost(message, tag = "17")]
        IbcAction(super::super::super::ibc::v1alpha1::IbcAction),
        /// Governance:
        #[prost(message, tag = "18")]
        ProposalSubmit(super::super::super::governance::v1alpha1::ProposalSubmit),
        #[prost(message, tag = "19")]
        ProposalWithdraw(super::super::super::governance::v1alpha1::ProposalWithdraw),
        #[prost(message, tag = "20")]
        ValidatorVote(super::super::super::governance::v1alpha1::ValidatorVote),
        #[prost(message, tag = "21")]
        DelegatorVote(super::super::super::governance::v1alpha1::DelegatorVotePlan),
        #[prost(message, tag = "22")]
        ProposalDepositClaim(
            super::super::super::governance::v1alpha1::ProposalDepositClaim,
        ),
        #[prost(message, tag = "23")]
        Withdrawal(super::super::super::ibc::v1alpha1::Ics20Withdrawal),
        #[prost(message, tag = "30")]
        PositionOpen(super::super::super::dex::v1alpha1::PositionOpen),
        #[prost(message, tag = "31")]
        PositionClose(super::super::super::dex::v1alpha1::PositionClose),
        /// The position withdraw/reward claim actions require balance information so they have Plan types.
        #[prost(message, tag = "32")]
        PositionWithdraw(super::super::super::dex::v1alpha1::PositionWithdrawPlan),
        #[prost(message, tag = "34")]
        PositionRewardClaim(super::super::super::dex::v1alpha1::PositionRewardClaimPlan),
        /// We don't need any extra information (yet) to understand delegations,
        /// because we don't yet use flow encryption.
        #[prost(message, tag = "40")]
        Delegate(super::super::super::stake::v1alpha1::Delegate),
        /// We don't need any extra information (yet) to understand undelegations,
        /// because we don't yet use flow encryption.
        #[prost(message, tag = "41")]
        Undelegate(super::super::super::stake::v1alpha1::Undelegate),
        #[prost(message, tag = "42")]
        UndelegateClaim(super::super::super::stake::v1alpha1::UndelegateClaimPlan),
        /// DAO
        #[prost(message, tag = "50")]
        DaoSpend(super::super::super::governance::v1alpha1::DaoSpend),
        #[prost(message, tag = "51")]
        DaoOutput(super::super::super::governance::v1alpha1::DaoOutput),
        #[prost(message, tag = "52")]
        DaoDeposit(super::super::super::governance::v1alpha1::DaoDeposit),
    }
}
/// Describes a plan for forming a `Clue`.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CluePlan {
    /// The address.
    #[prost(message, optional, tag = "1")]
    pub address: ::core::option::Option<super::super::crypto::v1alpha1::Address>,
    /// The random seed to use for the clue plan.
    #[prost(bytes = "bytes", tag = "2")]
    pub rseed: ::prost::bytes::Bytes,
    /// The bits of precision.
    #[prost(uint64, tag = "3")]
    pub precision_bits: u64,
}
/// Describes a plan for forming a `Memo`.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MemoPlan {
    /// The plaintext.
    #[prost(message, optional, tag = "1")]
    pub plaintext: ::core::option::Option<MemoPlaintext>,
    /// The key to use to encrypt the memo.
    #[prost(bytes = "bytes", tag = "2")]
    pub key: ::prost::bytes::Bytes,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MemoCiphertext {
    #[prost(bytes = "bytes", tag = "1")]
    pub inner: ::prost::bytes::Bytes,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MemoPlaintext {
    #[prost(message, optional, tag = "1")]
    pub sender: ::core::option::Option<super::super::crypto::v1alpha1::Address>,
    #[prost(string, tag = "2")]
    pub text: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MemoView {
    #[prost(oneof = "memo_view::MemoView", tags = "1, 2")]
    pub memo_view: ::core::option::Option<memo_view::MemoView>,
}
/// Nested message and enum types in `MemoView`.
pub mod memo_view {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Visible {
        #[prost(message, optional, tag = "1")]
        pub ciphertext: ::core::option::Option<super::MemoCiphertext>,
        #[prost(message, optional, tag = "2")]
        pub plaintext: ::core::option::Option<super::MemoPlaintext>,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Opaque {
        #[prost(message, optional, tag = "1")]
        pub ciphertext: ::core::option::Option<super::MemoCiphertext>,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum MemoView {
        #[prost(message, tag = "1")]
        Visible(Visible),
        #[prost(message, tag = "2")]
        Opaque(Opaque),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpendPlan {
    /// The plaintext note we plan to spend.
    #[prost(message, optional, tag = "1")]
    pub note: ::core::option::Option<super::super::crypto::v1alpha1::Note>,
    /// The position of the note we plan to spend.
    #[prost(uint64, tag = "2")]
    pub position: u64,
    /// The randomizer to use for the spend.
    #[prost(bytes = "bytes", tag = "3")]
    pub randomizer: ::prost::bytes::Bytes,
    /// The blinding factor to use for the value commitment.
    #[prost(bytes = "bytes", tag = "4")]
    pub value_blinding: ::prost::bytes::Bytes,
    /// The first blinding factor to use for the ZK spend proof.
    #[prost(bytes = "bytes", tag = "5")]
    pub proof_blinding_r: ::prost::bytes::Bytes,
    /// The second blinding factor to use for the ZK spend proof.
    #[prost(bytes = "bytes", tag = "6")]
    pub proof_blinding_s: ::prost::bytes::Bytes,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OutputPlan {
    /// The value to send to this output.
    #[prost(message, optional, tag = "1")]
    pub value: ::core::option::Option<super::super::crypto::v1alpha1::Value>,
    /// The destination address to send it to.
    #[prost(message, optional, tag = "2")]
    pub dest_address: ::core::option::Option<super::super::crypto::v1alpha1::Address>,
    /// The rseed to use for the new note.
    #[prost(bytes = "bytes", tag = "3")]
    pub rseed: ::prost::bytes::Bytes,
    /// The blinding factor to use for the value commitment.
    #[prost(bytes = "bytes", tag = "4")]
    pub value_blinding: ::prost::bytes::Bytes,
    /// The first blinding factor to use for the ZK output proof.
    #[prost(bytes = "bytes", tag = "5")]
    pub proof_blinding_r: ::prost::bytes::Bytes,
    /// The second blinding factor to use for the ZK output proof.
    #[prost(bytes = "bytes", tag = "6")]
    pub proof_blinding_s: ::prost::bytes::Bytes,
}
