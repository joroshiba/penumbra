use std::str::FromStr;

use ark_groth16::r1cs_to_qap::LibsnarkReduction;
use ark_r1cs_std::{prelude::*, uint8::UInt8};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use decaf377::r1cs::ElementVar;
use decaf377::FieldExt;
use decaf377::{r1cs::FqVar, Bls12_377, Fq, Fr};

use ark_ff::ToConstraintField;
use ark_groth16::{Groth16, PreparedVerifyingKey, Proof, ProvingKey, VerifyingKey};
use ark_r1cs_std::prelude::AllocVar;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef};
use ark_snark::SNARK;
use decaf377_rdsa::{SpendAuth, VerificationKey};
use penumbra_proto::{core::crypto::v1alpha1 as pb, DomainType, TypeUrl};
use penumbra_tct as tct;
use penumbra_tct::r1cs::StateCommitmentVar;
use rand_core::OsRng;
use tct::r1cs::PositionVar;

use penumbra_asset::{balance, balance::commitment::BalanceCommitmentVar, Value};
use penumbra_keys::keys::{
    AuthorizationKeyVar, IncomingViewingKeyVar, NullifierKey, NullifierKeyVar,
    RandomizedVerificationKey, SeedPhrase, SpendAuthRandomizerVar, SpendKey,
};
use penumbra_proof_params::{ParameterSetup, VerifyingKeyExt, GROTH16_PROOF_LENGTH_BYTES};
use penumbra_sct::{Nullifier, NullifierVar};
use penumbra_shielded_pool::{note, Note, Rseed};

/// Groth16 proof for delegator voting.
#[derive(Clone, Debug)]
pub struct DelegatorVoteCircuit {
    // Witnesses
    /// Inclusion proof for the note commitment.
    state_commitment_proof: tct::Proof,
    /// The note being spent.
    note: Note,
    /// The blinding factor used for generating the value commitment.
    v_blinding: Fr,
    /// The randomizer used for generating the randomized spend auth key.
    spend_auth_randomizer: Fr,
    /// The spend authorization key.
    ak: VerificationKey<SpendAuth>,
    /// The nullifier deriving key.
    nk: NullifierKey,

    // Public inputs
    /// the merkle root of the state commitment tree.
    pub anchor: tct::Root,
    /// value commitment of the note to be spent.
    pub balance_commitment: balance::Commitment,
    /// nullifier of the note to be spent.
    pub nullifier: Nullifier,
    /// the randomized verification spend key.
    pub rk: VerificationKey<SpendAuth>,
    /// the start position of the proposal being voted on.
    pub start_position: tct::Position,
}

impl DelegatorVoteCircuit {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        state_commitment_proof: tct::Proof,
        note: Note,
        v_blinding: Fr,
        spend_auth_randomizer: Fr,
        ak: VerificationKey<SpendAuth>,
        nk: NullifierKey,
        anchor: tct::Root,
        balance_commitment: balance::Commitment,
        nullifier: Nullifier,
        rk: VerificationKey<SpendAuth>,
        start_position: tct::Position,
    ) -> Self {
        Self {
            state_commitment_proof,
            note,
            v_blinding,
            spend_auth_randomizer,
            ak,
            nk,
            anchor,
            balance_commitment,
            nullifier,
            rk,
            start_position,
        }
    }
}

impl ConstraintSynthesizer<Fq> for DelegatorVoteCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fq>) -> ark_relations::r1cs::Result<()> {
        // Witnesses
        let note_var = note::NoteVar::new_witness(cs.clone(), || Ok(self.note.clone()))?;
        let claimed_note_commitment = StateCommitmentVar::new_witness(cs.clone(), || {
            Ok(self.state_commitment_proof.commitment())
        })?;

        let delegator_position_var = tct::r1cs::PositionVar::new_witness(cs.clone(), || {
            Ok(self.state_commitment_proof.position())
        })?;
        let delegator_position_bits = delegator_position_var.to_bits_le()?;
        let merkle_path_var = tct::r1cs::MerkleAuthPathVar::new_witness(cs.clone(), || {
            Ok(self.state_commitment_proof)
        })?;

        let v_blinding_arr: [u8; 32] = self.v_blinding.to_bytes();
        let v_blinding_vars = UInt8::new_witness_vec(cs.clone(), &v_blinding_arr)?;

        let spend_auth_randomizer_var =
            SpendAuthRandomizerVar::new_witness(cs.clone(), || Ok(self.spend_auth_randomizer))?;
        let ak_element_var: AuthorizationKeyVar =
            AuthorizationKeyVar::new_witness(cs.clone(), || Ok(self.ak))?;
        let nk_var = NullifierKeyVar::new_witness(cs.clone(), || Ok(self.nk))?;

        // Public inputs
        let anchor_var = FqVar::new_input(cs.clone(), || Ok(Fq::from(self.anchor)))?;
        let claimed_balance_commitment_var =
            BalanceCommitmentVar::new_input(cs.clone(), || Ok(self.balance_commitment))?;
        let claimed_nullifier_var = NullifierVar::new_input(cs.clone(), || Ok(self.nullifier))?;
        let rk_var = RandomizedVerificationKey::new_input(cs.clone(), || Ok(self.rk.clone()))?;
        let start_position = PositionVar::new_input(cs.clone(), || Ok(self.start_position))?;

        // Note commitment integrity.
        let note_commitment_var = note_var.commit()?;
        note_commitment_var.enforce_equal(&claimed_note_commitment)?;

        // Nullifier integrity.
        let nullifier_var =
            NullifierVar::derive(&nk_var, &delegator_position_var, &claimed_note_commitment)?;
        nullifier_var.enforce_equal(&claimed_nullifier_var)?;

        // Merkle auth path verification against the provided anchor.
        merkle_path_var.verify(
            cs.clone(),
            &Boolean::TRUE,
            &delegator_position_bits,
            anchor_var,
            claimed_note_commitment.inner(),
        )?;

        // Check integrity of randomized verification key.
        let computed_rk_var = ak_element_var.randomize(&spend_auth_randomizer_var)?;
        computed_rk_var.enforce_equal(&rk_var)?;

        // Check integrity of diversified address.
        let ivk = IncomingViewingKeyVar::derive(&nk_var, &ak_element_var)?;
        let computed_transmission_key =
            ivk.diversified_public(&note_var.diversified_generator())?;
        computed_transmission_key.enforce_equal(&note_var.transmission_key())?;

        // Check integrity of balance commitment.
        let balance_commitment = note_var.value().commit(v_blinding_vars)?;
        balance_commitment.enforce_equal(&claimed_balance_commitment_var)?;

        // Check elements were not identity.
        let identity = ElementVar::new_constant(cs, decaf377::Element::default())?;
        identity.enforce_not_equal(&note_var.diversified_generator())?;
        identity.enforce_not_equal(&ak_element_var.inner)?;

        // Additionally, check that the start position has a zero commitment index, since this is
        // the only sensible start time for a vote.
        let zero_constant = FqVar::constant(Fq::from(0u64));
        let commitment = start_position.commitment()?;
        commitment.enforce_equal(&zero_constant)?;

        // Additionally, check that the position of the spend proof is before the start
        // start_height, which ensures that the note being voted with was created before voting
        // started.
        //
        // Also note that `FpVar::enforce_cmp` requires that the field elements have size
        // (p-1)/2, which is true for positions as they are 64 bits at most.
        //
        // This MUST be strict inequality (hence passing false to `should_also_check_equality`)
        // because you could delegate and vote on the proposal in the same block.
        delegator_position_var.position.enforce_cmp(
            &start_position.position,
            core::cmp::Ordering::Less,
            false,
        )?;

        Ok(())
    }
}

impl ParameterSetup for DelegatorVoteCircuit {
    fn generate_test_parameters() -> (ProvingKey<Bls12_377>, VerifyingKey<Bls12_377>) {
        let seed_phrase = SeedPhrase::from_randomness([b'f'; 32]);
        let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (address, _dtk_d) = ivk_sender.payment_address(0u32.into());

        let spend_auth_randomizer = Fr::from(1);
        let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
        let nk = *sk_sender.nullifier_key();
        let ak = sk_sender.spend_auth_key().into();
        let note = Note::from_parts(
            address,
            Value::from_str("1upenumbra").expect("valid value"),
            Rseed([1u8; 32]),
        )
        .expect("can make a note");
        let v_blinding = Fr::from(1);
        let rk: VerificationKey<SpendAuth> = rsk.into();
        let nullifier = Nullifier(Fq::from(1));
        let mut sct = tct::Tree::new();
        let note_commitment = note.commit();
        sct.insert(tct::Witness::Keep, note_commitment).unwrap();
        let anchor = sct.root();
        let state_commitment_proof = sct.witness(note_commitment).unwrap();
        let start_position = state_commitment_proof.position();

        let circuit = DelegatorVoteCircuit {
            state_commitment_proof,
            note,
            v_blinding,
            spend_auth_randomizer,
            ak,
            nk,
            anchor,
            balance_commitment: balance::Commitment(decaf377::basepoint()),
            nullifier,
            rk,
            start_position,
        };
        let (pk, vk) =
            Groth16::<Bls12_377, LibsnarkReduction>::circuit_specific_setup(circuit, &mut OsRng)
                .expect("can perform circuit specific setup");
        (pk, vk)
    }
}

#[derive(Clone, Debug)]
pub struct DelegatorVoteProof([u8; GROTH16_PROOF_LENGTH_BYTES]);

impl DelegatorVoteProof {
    #![allow(clippy::too_many_arguments)]
    pub fn prove(
        blinding_r: Fq,
        blinding_s: Fq,
        pk: &ProvingKey<Bls12_377>,
        state_commitment_proof: tct::Proof,
        note: Note,
        spend_auth_randomizer: Fr,
        ak: VerificationKey<SpendAuth>,
        nk: NullifierKey,
        anchor: tct::Root,
        balance_commitment: balance::Commitment,
        nullifier: Nullifier,
        rk: VerificationKey<SpendAuth>,
        start_position: tct::Position,
    ) -> anyhow::Result<Self> {
        // The blinding factor for the value commitment is zero since it
        // is not blinded.
        let zero_blinding = Fr::from(0);
        let circuit = DelegatorVoteCircuit {
            state_commitment_proof,
            note,
            v_blinding: zero_blinding,
            spend_auth_randomizer,
            ak,
            nk,
            anchor,
            balance_commitment,
            nullifier,
            rk,
            start_position,
        };
        let proof = Groth16::<Bls12_377, LibsnarkReduction>::create_proof_with_reduction(
            circuit, pk, blinding_r, blinding_s,
        )
        .map_err(|err| anyhow::anyhow!(err))?;
        let mut proof_bytes = [0u8; GROTH16_PROOF_LENGTH_BYTES];
        Proof::serialize_compressed(&proof, &mut proof_bytes[..]).expect("can serialize Proof");
        Ok(Self(proof_bytes))
    }

    /// Called to verify the proof using the provided public inputs.
    // For debugging proof verification failures,
    // to check that the proof data and verification keys are consistent.
    #[tracing::instrument(level="debug", skip(self, vk), fields(self = ?base64::encode(&self.clone().encode_to_vec()), vk = ?vk.debug_id()))]
    pub fn verify(
        &self,
        vk: &PreparedVerifyingKey<Bls12_377>,
        anchor: tct::Root,
        balance_commitment: balance::Commitment,
        nullifier: Nullifier,
        rk: VerificationKey<SpendAuth>,
        start_position: tct::Position,
    ) -> anyhow::Result<()> {
        let proof =
            Proof::deserialize_compressed_unchecked(&self.0[..]).map_err(|e| anyhow::anyhow!(e))?;

        let mut public_inputs = Vec::new();
        public_inputs.extend(Fq::from(anchor.0).to_field_elements().unwrap());
        public_inputs.extend(balance_commitment.0.to_field_elements().unwrap());
        public_inputs.extend(nullifier.0.to_field_elements().unwrap());
        let element_rk = decaf377::Encoding(rk.to_bytes())
            .vartime_decompress()
            .expect("expect only valid element points");
        public_inputs.extend(element_rk.to_field_elements().unwrap());
        public_inputs.extend(start_position.to_field_elements().unwrap());

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
            .ok_or_else(|| anyhow::anyhow!("delegator vote proof did not verify"))
    }
}

impl TypeUrl for DelegatorVoteProof {
    const TYPE_URL: &'static str = "/penumbra.core.crypto.v1alpha1.ZkDelegatorVoteProof";
}

impl DomainType for DelegatorVoteProof {
    type Proto = pb::ZkDelegatorVoteProof;
}

impl From<DelegatorVoteProof> for pb::ZkDelegatorVoteProof {
    fn from(proof: DelegatorVoteProof) -> Self {
        pb::ZkDelegatorVoteProof {
            inner: proof.0.to_vec(),
        }
    }
}

impl TryFrom<pb::ZkDelegatorVoteProof> for DelegatorVoteProof {
    type Error = anyhow::Error;

    fn try_from(proto: pb::ZkDelegatorVoteProof) -> Result<Self, Self::Error> {
        Ok(DelegatorVoteProof(proto.inner[..].try_into()?))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use ark_ff::{PrimeField, UniformRand};

    use decaf377::{Fq, Fr};
    use penumbra_asset::{asset, Value};
    use penumbra_keys::keys::{SeedPhrase, SpendKey};
    use penumbra_sct::Nullifier;
    use proptest::prelude::*;

    fn fr_strategy() -> BoxedStrategy<Fr> {
        any::<[u8; 32]>()
            .prop_map(|bytes| Fr::from_le_bytes_mod_order(&bytes[..]))
            .boxed()
    }

    proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]
    #[test]
    fn delegator_vote_happy_path(seed_phrase_randomness in any::<[u8; 32]>(), spend_auth_randomizer in fr_strategy(), value_amount in 1..2000000000u64, num_commitments in 0..2000u64) {
        let (pk, vk) = DelegatorVoteCircuit::generate_prepared_test_parameters();
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::from_randomness(seed_phrase_randomness);
        let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (sender, _dtk_d) = ivk_sender.payment_address(0u32.into());

        let value_to_send = Value {
            amount: value_amount.into(),
            asset_id: asset::Cache::with_known_assets().get_unit("upenumbra").unwrap().id(),
        };

        let note = Note::generate(&mut rng, &sender, value_to_send);
        let note_commitment = note.commit();
        let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
        let nk = *sk_sender.nullifier_key();
        let ak: VerificationKey<SpendAuth> = sk_sender.spend_auth_key().into();
        let mut sct = tct::Tree::new();

        // Next, we simulate the case where the SCT is not empty by adding `num_commitments`
        // unrelated items in the SCT.
        for _ in 0..num_commitments {
            let random_note_commitment = Note::generate(&mut rng, &sender, value_to_send).commit();
            sct.insert(tct::Witness::Keep, random_note_commitment).unwrap();
        }

        sct.insert(tct::Witness::Keep, note_commitment).unwrap();
        let anchor = sct.root();
        let state_commitment_proof = sct.witness(note_commitment).unwrap();
        sct.end_epoch().unwrap();

        let first_note_commitment = Note::generate(&mut rng, &sender, value_to_send).commit();
        sct.insert(tct::Witness::Keep, first_note_commitment).unwrap();
        let start_position = sct.witness(first_note_commitment).unwrap().position();

        let balance_commitment = value_to_send.commit(Fr::from(0u64));
        let rk: VerificationKey<SpendAuth> = rsk.into();
        let nf = Nullifier::derive(&nk, state_commitment_proof.position(), &note_commitment);

        let blinding_r = Fq::rand(&mut OsRng);
        let blinding_s = Fq::rand(&mut OsRng);

        let proof = DelegatorVoteProof::prove(
            blinding_r,
            blinding_s,
            &pk,
            state_commitment_proof,
            note,
            spend_auth_randomizer,
            ak,
            nk,
            anchor,
            balance_commitment,
            nf,
            rk,
            start_position,
        )
        .expect("can create proof");

        let proof_result = proof.verify(&vk, anchor, balance_commitment, nf, rk, start_position);
        assert!(proof_result.is_ok());
        }
    }

    proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]
    #[test]
    #[should_panic]
    fn delegator_vote_invalid_start_position(seed_phrase_randomness in any::<[u8; 32]>(), spend_auth_randomizer in fr_strategy(), value_amount in 1..2000000000u64, num_commitments in 1000..2000u64) {
        let (pk, vk) = DelegatorVoteCircuit::generate_prepared_test_parameters();
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::from_randomness(seed_phrase_randomness);
        let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (sender, _dtk_d) = ivk_sender.payment_address(0u32.into());

        let value_to_send = Value {
            amount: value_amount.into(),
            asset_id: asset::Cache::with_known_assets().get_unit("upenumbra").unwrap().id(),
        };

        let note = Note::generate(&mut rng, &sender, value_to_send);
        let note_commitment = note.commit();
        let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
        let nk = *sk_sender.nullifier_key();
        let ak: VerificationKey<SpendAuth> = sk_sender.spend_auth_key().into();
        let mut sct = tct::Tree::new();

        // Next, we simulate the case where the SCT is not empty by adding `num_commitments`
        // unrelated items in the SCT.
        for _ in 0..num_commitments {
            let random_note_commitment = Note::generate(&mut rng, &sender, value_to_send).commit();
            sct.insert(tct::Witness::Keep, random_note_commitment).unwrap();
        }

        sct.insert(tct::Witness::Keep, note_commitment).unwrap();
        let anchor = sct.root();
        let state_commitment_proof = sct.witness(note_commitment).unwrap();

        let not_first_note_commitment = Note::generate(&mut rng, &sender, value_to_send).commit();
        sct.insert(tct::Witness::Keep, not_first_note_commitment).unwrap();
        let start_position = sct.witness(not_first_note_commitment).unwrap().position();

        let balance_commitment = value_to_send.commit(Fr::from(0u64));
        let rk: VerificationKey<SpendAuth> = rsk.into();
        let nf = Nullifier::derive(&nk, state_commitment_proof.position(), &note_commitment);

        let blinding_r = Fq::rand(&mut OsRng);
        let blinding_s = Fq::rand(&mut OsRng);

        let proof = DelegatorVoteProof::prove(
            blinding_r,
            blinding_s,
            &pk,
            state_commitment_proof,
            note,
            spend_auth_randomizer,
            ak,
            nk,
            anchor,
            balance_commitment,
            nf,
            rk,
            start_position,
        ).expect("can form proof in release mode, but it should not verify");

        // In debug mode, we won't be able to construct a valid proof if the start position
        // commitment index is non-zero. However, in release mode, the proof will be constructed
        // but not verify. This is due to the fact there is an assertion during constraint
        // generation (upstream) where we panic in debug mode if the circuit is not satisifiable,
        // but not in release mode. To ensure the same behavior in this test for both modes,
        // we panic if we get here and the proof does not verify (expected).
        let proof_result = proof.verify(&vk, anchor, balance_commitment, nf, rk, start_position);
        proof_result.expect("we expect this proof _not_ to verify, so this will cause a panic");
    }
    }
}
