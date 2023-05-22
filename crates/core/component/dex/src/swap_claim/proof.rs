use ark_ff::ToConstraintField;
use ark_groth16::{
    r1cs_to_qap::LibsnarkReduction, Groth16, PreparedVerifyingKey, Proof, ProvingKey, VerifyingKey,
};
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_snark::SNARK;
use decaf377::{r1cs::FqVar, Bls12_377};
use penumbra_proto::{core::crypto::v1alpha1 as pb, DomainType, TypeUrl};
use penumbra_tct as tct;
use rand_core::{CryptoRngCore, OsRng};

use penumbra_crypto::{
    asset::{self, AmountVar},
    keys::{NullifierKey, NullifierKeyVar, SeedPhrase, SpendKey},
    note::{self, NoteVar, StateCommitmentVar},
    Amount, Fee, Fq, Nullifier, NullifierVar, Rseed, Value, ValueVar,
};

use crate::{
    batch_swap_output_data::BatchSwapOutputDataVar,
    swap::{SwapPlaintext, SwapPlaintextVar},
    BatchSwapOutputData, TradingPair,
};

use penumbra_crypto::proofs::groth16::{ParameterSetup, GROTH16_PROOF_LENGTH_BYTES};

/// SwapClaim consumes an existing Swap NFT so they are most similar to Spend operations,
/// however the note commitment proof needs to be for a specific block due to clearing prices
/// only being valid for particular blocks (i.e. the exchange rates of assets change over time).
pub struct SwapClaimCircuit {
    /// The swap being claimed
    swap_plaintext: SwapPlaintext,
    /// Inclusion proof for the swap commitment
    state_commitment_proof: tct::Proof,
    // The nullifier deriving key for the Swap NFT note.
    nk: NullifierKey,
    /// Output amount 1
    lambda_1: Amount,
    /// Output amount 2
    lambda_2: Amount,
    /// Note commitment blinding factor for the first output note
    note_blinding_1: Fq,
    /// Note commitment blinding factor for the second output note
    note_blinding_2: Fq,
    /// Anchor
    pub anchor: tct::Root,
    /// Nullifier
    pub nullifier: Nullifier,
    /// Fee
    pub claim_fee: Fee,
    /// Batch swap output data
    pub output_data: BatchSwapOutputData,
    /// Note commitment of first output note
    pub note_commitment_1: note::Commitment,
    /// Note commitment of second output note
    pub note_commitment_2: note::Commitment,
}

impl SwapClaimCircuit {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        swap_plaintext: SwapPlaintext,
        state_commitment_proof: tct::Proof,
        nk: NullifierKey,
        lambda_1: Amount,
        lambda_2: Amount,
        note_blinding_1: Fq,
        note_blinding_2: Fq,
        anchor: tct::Root,
        nullifier: Nullifier,
        claim_fee: Fee,
        output_data: BatchSwapOutputData,
        note_commitment_1: note::Commitment,
        note_commitment_2: note::Commitment,
    ) -> Self {
        Self {
            swap_plaintext,
            state_commitment_proof,
            nk,
            lambda_1,
            lambda_2,
            note_blinding_1,
            note_blinding_2,
            anchor,
            nullifier,
            claim_fee,
            output_data,
            note_commitment_1,
            note_commitment_2,
        }
    }
}

impl ConstraintSynthesizer<Fq> for SwapClaimCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fq>) -> ark_relations::r1cs::Result<()> {
        // Witnesses
        let swap_plaintext_var =
            SwapPlaintextVar::new_witness(cs.clone(), || Ok(self.swap_plaintext.clone()))?;

        let claimed_swap_commitment = note::StateCommitmentVar::new_witness(cs.clone(), || {
            Ok(self.state_commitment_proof.commitment())
        })?;

        let position_var = tct::r1cs::PositionVar::new_witness(cs.clone(), || {
            Ok(self.state_commitment_proof.position())
        })?;
        let position_bits = tct::r1cs::PositionBitsVar::new_witness(cs.clone(), || {
            Ok(self.state_commitment_proof.position())
        })?;
        let merkle_path_var = tct::r1cs::MerkleAuthPathVar::new_witness(cs.clone(), || {
            Ok(self.state_commitment_proof)
        })?;
        let nk_var = NullifierKeyVar::new_witness(cs.clone(), || Ok(self.nk))?;
        let lambda_1_i_var = AmountVar::new_witness(cs.clone(), || Ok(self.lambda_1))?;
        let lambda_2_i_var = AmountVar::new_witness(cs.clone(), || Ok(self.lambda_2))?;
        let note_blinding_1 = FqVar::new_witness(cs.clone(), || Ok(self.note_blinding_1))?;
        let note_blinding_2 = FqVar::new_witness(cs.clone(), || Ok(self.note_blinding_2))?;

        // Inputs
        let anchor_var = FqVar::new_input(cs.clone(), || Ok(Fq::from(self.anchor)))?;
        let claimed_nullifier_var = NullifierVar::new_input(cs.clone(), || Ok(self.nullifier))?;
        let claimed_fee_var = ValueVar::new_input(cs.clone(), || Ok(self.claim_fee.0))?;
        let output_data_var =
            BatchSwapOutputDataVar::new_input(cs.clone(), || Ok(self.output_data))?;
        let claimed_note_commitment_1 =
            StateCommitmentVar::new_input(cs.clone(), || Ok(self.note_commitment_1))?;
        let claimed_note_commitment_2 =
            StateCommitmentVar::new_input(cs.clone(), || Ok(self.note_commitment_2))?;

        // Swap commitment integrity check.
        let swap_commitment = swap_plaintext_var.commit()?;
        claimed_swap_commitment.enforce_equal(&swap_commitment)?;

        // Merkle path integrity. Ensure the provided note commitment is in the TCT.
        merkle_path_var.verify(
            cs.clone(),
            &Boolean::TRUE,
            &position_bits.to_bits_le()?,
            anchor_var,
            claimed_swap_commitment.inner(),
        )?;

        // Nullifier integrity.
        let nullifier_var = nk_var.derive_nullifier(&position_var, &claimed_swap_commitment)?;
        nullifier_var.enforce_equal(&claimed_nullifier_var)?;

        // Fee consistency check.
        claimed_fee_var.enforce_equal(&swap_plaintext_var.claim_fee)?;

        // Validate the swap commitment's height matches the output data's height (i.e. the clearing price height).
        let block = position_bits.block()?;
        let note_commitment_block_height_var = output_data_var.epoch_height + block;
        output_data_var
            .height
            .enforce_equal(&note_commitment_block_height_var)?;

        // Validate that the output data's trading pair matches the note commitment's trading pair.
        output_data_var
            .trading_pair
            .enforce_equal(&swap_plaintext_var.trading_pair)?;

        // Output amounts integrity
        // let (computed_lambda_1_i, computed_lambda_2_i) = output_data_var
        //     .pro_rata_outputs(swap_plaintext_var.delta_1_i, swap_plaintext_var.delta_2_i)?;
        // computed_lambda_1_i.enforce_equal(&lambda_1_i_var)?;
        // computed_lambda_2_i.enforce_equal(&lambda_2_i_var)?;

        // Output note integrity
        let output_1_note = NoteVar {
            address: swap_plaintext_var.claim_address.clone(),
            value: ValueVar {
                amount: lambda_1_i_var,
                asset_id: swap_plaintext_var.trading_pair.asset_1,
            },
            note_blinding: note_blinding_1,
        };
        let output_1_commitment = output_1_note.commit()?;
        let output_2_note = NoteVar {
            address: swap_plaintext_var.claim_address,
            value: ValueVar {
                amount: lambda_2_i_var,
                asset_id: swap_plaintext_var.trading_pair.asset_2,
            },
            note_blinding: note_blinding_2,
        };
        let output_2_commitment = output_2_note.commit()?;

        claimed_note_commitment_1.enforce_equal(&output_1_commitment)?;
        claimed_note_commitment_2.enforce_equal(&output_2_commitment)?;

        Ok(())
    }
}

impl ParameterSetup for SwapClaimCircuit {
    fn generate_test_parameters() -> (ProvingKey<Bls12_377>, VerifyingKey<Bls12_377>) {
        let trading_pair = TradingPair {
            asset_1: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
            asset_2: asset::REGISTRY.parse_denom("nala").unwrap().id(),
        };

        let seed_phrase = SeedPhrase::from_randomness([b'f'; 32]);
        let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (address, _dtk_d) = ivk_sender.payment_address(0u32.into());
        let nk = *sk_sender.nullifier_key();

        let swap_plaintext = SwapPlaintext {
            trading_pair,
            delta_1_i: 100000u64.into(),
            delta_2_i: 1u64.into(),
            claim_fee: Fee(Value {
                amount: 3u64.into(),
                asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
            }),
            claim_address: address,
            rseed: Rseed([1u8; 32]),
        };
        let mut sct = tct::Tree::new();
        let swap_commitment = swap_plaintext.swap_commitment();
        sct.insert(tct::Witness::Keep, swap_commitment).unwrap();
        let anchor = sct.root();
        let state_commitment_proof = sct.witness(swap_commitment).unwrap();
        let nullifier = Nullifier(Fq::from(1));
        let claim_fee = Fee::default();
        let output_data = BatchSwapOutputData {
            delta_1: Amount::from(0u64),
            delta_2: Amount::from(0u64),
            lambda_1: Amount::from(0u64),
            lambda_2: Amount::from(0u64),
            unfilled_1: Amount::from(0u64),
            unfilled_2: Amount::from(0u64),
            height: 0,
            trading_pair: swap_plaintext.trading_pair,
            epoch_height: 0,
        };
        let note_blinding_1 = Fq::from(1);
        let note_blinding_2 = Fq::from(1);
        let note_commitment_1 = tct::Commitment(Fq::from(1));
        let note_commitment_2 = tct::Commitment(Fq::from(2));

        let circuit = SwapClaimCircuit {
            swap_plaintext,
            state_commitment_proof,
            anchor,
            nullifier,
            nk,
            claim_fee,
            output_data,
            lambda_1: Amount::from(1u64),
            lambda_2: Amount::from(1u64),
            note_blinding_1,
            note_blinding_2,
            note_commitment_1,
            note_commitment_2,
        };
        let (pk, vk) =
            Groth16::<Bls12_377, LibsnarkReduction>::circuit_specific_setup(circuit, &mut OsRng)
                .expect("can perform circuit specific setup");
        (pk, vk)
    }
}

#[derive(Clone, Debug)]
pub struct SwapClaimProof(pub [u8; GROTH16_PROOF_LENGTH_BYTES]);

impl SwapClaimProof {
    #![allow(clippy::too_many_arguments)]
    pub fn prove<R: CryptoRngCore>(
        rng: &mut R,
        pk: &ProvingKey<Bls12_377>,
        swap_plaintext: SwapPlaintext,
        state_commitment_proof: tct::Proof,
        nk: NullifierKey,
        anchor: tct::Root,
        nullifier: Nullifier,
        lambda_1: Amount,
        lambda_2: Amount,
        note_blinding_1: Fq,
        note_blinding_2: Fq,
        note_commitment_1: tct::Commitment,
        note_commitment_2: tct::Commitment,
        output_data: BatchSwapOutputData,
    ) -> anyhow::Result<Self> {
        let circuit = SwapClaimCircuit {
            swap_plaintext: swap_plaintext.clone(),
            state_commitment_proof,
            nk,
            anchor,
            nullifier,
            claim_fee: swap_plaintext.claim_fee,
            output_data,
            lambda_1,
            lambda_2,
            note_blinding_1,
            note_blinding_2,
            note_commitment_1,
            note_commitment_2,
        };
        let proof = Groth16::<Bls12_377, LibsnarkReduction>::prove(pk, circuit, rng)
            .map_err(|err| anyhow::anyhow!(err))?;

        let mut proof_bytes = [0u8; GROTH16_PROOF_LENGTH_BYTES];
        Proof::serialize_compressed(&proof, &mut proof_bytes[..]).expect("can serialize Proof");
        Ok(Self(proof_bytes))
    }

    /// Called to verify the proof using the provided public inputs.
    //#[tracing::instrument(skip(self, vk), fields(self = ?base64::encode(&self.clone().encode_to_vec()), vk = ?vk.debug_id()))]
    #[tracing::instrument(skip(self, vk))]
    pub fn verify(
        &self,
        vk: &PreparedVerifyingKey<Bls12_377>,
        anchor: tct::Root,
        nullifier: Nullifier,
        fee: Fee,
        output_data: BatchSwapOutputData,
        note_commitment_1: tct::Commitment,
        note_commitment_2: tct::Commitment,
    ) -> anyhow::Result<()> {
        let proof =
            Proof::deserialize_compressed_unchecked(&self.0[..]).map_err(|e| anyhow::anyhow!(e))?;

        let mut public_inputs = Vec::new();
        public_inputs.extend(Fq::from(anchor.0).to_field_elements().unwrap());
        public_inputs.extend(nullifier.0.to_field_elements().unwrap());
        public_inputs.extend(Fq::from(fee.0.amount).to_field_elements().unwrap());
        public_inputs.extend(fee.0.asset_id.0.to_field_elements().unwrap());
        public_inputs.extend(output_data.to_field_elements().unwrap());
        public_inputs.extend(note_commitment_1.0.to_field_elements().unwrap());
        public_inputs.extend(note_commitment_2.0.to_field_elements().unwrap());

        tracing::trace!(?public_inputs);
        let start = std::time::Instant::now();
        let proof_result = Groth16::<Bls12_377, LibsnarkReduction>::verify_with_processed_vk(
            vk,
            public_inputs.as_slice(),
            &proof,
        )
        .map_err(|err| anyhow::anyhow!(err))?;
        tracing::debug!(?proof_result, elapsed = ?start.elapsed());
        proof_result
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("swapclaim proof did not verify"))
    }
}

impl TypeUrl for SwapClaimProof {
    const TYPE_URL: &'static str = "/penumbra.core.crypto.v1alpha1.ZKSwapClaimProof";
}

impl DomainType for SwapClaimProof {
    type Proto = pb::ZkSwapClaimProof;
}

impl From<SwapClaimProof> for pb::ZkSwapClaimProof {
    fn from(proof: SwapClaimProof) -> Self {
        pb::ZkSwapClaimProof {
            inner: proof.0.to_vec(),
        }
    }
}

impl TryFrom<pb::ZkSwapClaimProof> for SwapClaimProof {
    type Error = anyhow::Error;

    fn try_from(proto: pb::ZkSwapClaimProof) -> Result<Self, Self::Error> {
        Ok(SwapClaimProof(proto.inner[..].try_into()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use penumbra_crypto::{
        keys::{SeedPhrase, SpendKey},
        Amount,
    };
    use proptest::prelude::*;

    proptest! {
    #![proptest_config(ProptestConfig::with_cases(2))]
    #[test]
    fn swap_claim_proof_happy_path(seed_phrase_randomness in any::<[u8; 32]>(), value1_amount in 2..200u64) {
        let (pk, vk) = SwapClaimCircuit::generate_prepared_test_parameters();

        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::from_randomness(seed_phrase_randomness);
        let sk_recipient = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (claim_address, _dtk_d) = ivk_recipient.payment_address(0u32.into());
        let nk = *sk_recipient.nullifier_key();

        let gm = asset::REGISTRY.parse_unit("gm");
        let gn = asset::REGISTRY.parse_unit("gn");
        let trading_pair = TradingPair::new(gm.id(), gn.id());

        let delta_1_i = Amount::from(value1_amount);
        let delta_2_i = Amount::from(0u64);
        let fee = Fee::default();

        let swap_plaintext =
        SwapPlaintext::new(&mut rng, trading_pair, delta_1_i, delta_2_i, fee, claim_address);
        let fee = swap_plaintext.clone().claim_fee;
        let mut sct = tct::Tree::new();
        let swap_commitment = swap_plaintext.swap_commitment();
        sct.insert(tct::Witness::Keep, swap_commitment).unwrap();
        let anchor = sct.root();
        let state_commitment_proof = sct.witness(swap_commitment).unwrap();
        let position = state_commitment_proof.position();
        let nullifier = nk.derive_nullifier(position, &swap_commitment);
        let epoch_duration = 20;
        let height = epoch_duration * position.epoch() + position.block();

        let output_data = BatchSwapOutputData {
            delta_1: Amount::from(100u64),
            delta_2: Amount::from(100u64),
            lambda_1: Amount::from(50u64),
            lambda_2: Amount::from(25u64),
            unfilled_1: Amount::from(23u64),
            unfilled_2: Amount::from(50u64),
            height: height.into(),
            trading_pair: swap_plaintext.trading_pair,
            epoch_height: position.epoch().into(),
        };
        let (lambda_1, lambda_2) = output_data.pro_rata_outputs(
            (delta_1_i.try_into().unwrap(),
            delta_2_i.try_into().unwrap())
        );

        let (output_rseed_1, output_rseed_2) = swap_plaintext.output_rseeds();
        let note_blinding_1 = output_rseed_1.derive_note_blinding();
        let note_blinding_2 = output_rseed_2.derive_note_blinding();
        let (output_1_note, output_2_note) = swap_plaintext.output_notes(&output_data);
        let note_commitment_1 = output_1_note.commit();
        let note_commitment_2 = output_2_note.commit();

        let proof = SwapClaimProof::prove(
            &mut rng,
            &pk,
            swap_plaintext,
            state_commitment_proof,
            nk,
            anchor,
            nullifier,
            lambda_1,
            lambda_2,
            note_blinding_1,
            note_blinding_2,
            note_commitment_1,
            note_commitment_2,
            output_data,
        )
        .expect("can create proof");

        let proof_result = proof.verify(&vk,
        anchor,
        nullifier,
        fee,
        output_data,
        note_commitment_1,
        note_commitment_2);

        assert!(proof_result.is_ok());
        }
    }
}
