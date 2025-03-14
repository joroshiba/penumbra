use std::convert::{From, TryFrom};

/// A marker type that captures the relationships between a domain type (`Self`) and a protobuf type (`Self::Proto`).
pub trait DomainType
where
    // Self: TypeUrl will be required once TypeUrl is implemented for all domain types.
    Self: TypeUrl + Clone + Sized + TryFrom<Self::Proto>,
    Self::Proto: prost::Message + Default + From<Self> + Send + Sync + 'static,
    anyhow::Error: From<<Self as TryFrom<Self::Proto>>::Error>,
{
    type Proto;

    /// Encode this domain type to a byte vector, via proto type `P`.
    fn encode_to_vec(&self) -> Vec<u8> {
        use prost::Message;
        self.to_proto().encode_to_vec()
    }

    /// Convert this domain type to the associated proto type.
    ///
    /// This uses the `From` impl internally, so it works exactly
    /// like `.into()`, but does not require type inference.
    fn to_proto(&self) -> Self::Proto {
        Self::Proto::from(self.clone())
    }

    /// Decode this domain type from a byte buffer, via proto type `P`.
    fn decode<B: bytes::Buf>(buf: B) -> Result<Self, anyhow::Error> {
        <Self::Proto as prost::Message>::decode(buf)?
            .try_into()
            .map_err(Into::into)
    }
}

/// A type that can be encoded to a protobuf `Any` message.
pub trait TypeUrl {
    const TYPE_URL: &'static str;
}

// Implementations on foreign types.
//
// This should only be done here in cases where the domain type lives in a crate
// that shouldn't depend on the Penumbra proto framework.

use crate::core::crypto::v1alpha1::{BindingSignature, SpendAuthSignature, SpendVerificationKey};
use crate::core::ibc::v1alpha1::IbcAction;
use decaf377_rdsa::{Binding, Signature, SpendAuth, VerificationKey};

impl TypeUrl for Signature<SpendAuth> {
    const TYPE_URL: &'static str = "/penumbra.core.crypto.v1alpha1.SpendAuthSignature";
}
impl TypeUrl for Signature<Binding> {
    const TYPE_URL: &'static str = "/penumbra.core.crypto.v1alpha1.BindingSignature";
}
impl TypeUrl for VerificationKey<SpendAuth> {
    const TYPE_URL: &'static str = "/penumbra.core.crypto.v1alpha1.SpendVerificationKey";
}

impl DomainType for Signature<SpendAuth> {
    type Proto = SpendAuthSignature;
}
impl DomainType for Signature<Binding> {
    type Proto = BindingSignature;
}
impl DomainType for VerificationKey<SpendAuth> {
    type Proto = SpendVerificationKey;
}

impl From<Signature<SpendAuth>> for SpendAuthSignature {
    fn from(sig: Signature<SpendAuth>) -> Self {
        Self {
            inner: sig.to_bytes().to_vec(),
        }
    }
}

impl From<Signature<Binding>> for BindingSignature {
    fn from(sig: Signature<Binding>) -> Self {
        Self {
            inner: sig.to_bytes().to_vec(),
        }
    }
}

impl From<VerificationKey<SpendAuth>> for SpendVerificationKey {
    fn from(key: VerificationKey<SpendAuth>) -> Self {
        Self {
            inner: key.to_bytes().to_vec(),
        }
    }
}

impl TryFrom<SpendAuthSignature> for Signature<SpendAuth> {
    type Error = anyhow::Error;
    fn try_from(value: SpendAuthSignature) -> Result<Self, Self::Error> {
        Ok(value.inner.as_slice().try_into()?)
    }
}

impl TryFrom<BindingSignature> for Signature<Binding> {
    type Error = anyhow::Error;
    fn try_from(value: BindingSignature) -> Result<Self, Self::Error> {
        Ok(value.inner.as_slice().try_into()?)
    }
}

impl TryFrom<SpendVerificationKey> for VerificationKey<SpendAuth> {
    type Error = anyhow::Error;
    fn try_from(value: SpendVerificationKey) -> Result<Self, Self::Error> {
        Ok(value.inner.as_slice().try_into()?)
    }
}

// Fuzzy Message Detection
use crate::core::crypto::v1alpha1::Clue as ProtoClue;
use decaf377_fmd::Clue;

impl TypeUrl for Clue {
    const TYPE_URL: &'static str = "/penumbra.core.crypto.v1alpha1.Clue";
}

impl DomainType for Clue {
    type Proto = ProtoClue;
}

impl From<Clue> for ProtoClue {
    fn from(msg: Clue) -> Self {
        ProtoClue {
            inner: bytes::Bytes::copy_from_slice(&msg.0).to_vec(),
        }
    }
}

impl TryFrom<ProtoClue> for Clue {
    type Error = anyhow::Error;

    fn try_from(proto: ProtoClue) -> anyhow::Result<Self, Self::Error> {
        let clue: [u8; 68] = proto.inner[..]
            .try_into()
            .map_err(|_| anyhow::anyhow!("expected 68-byte clue"))?;

        Ok(Clue(clue))
    }
}

// Consensus key
//
// The tendermint-rs PublicKey type already has a tendermint-proto type;
// this redefines its proto, because the encodings are consensus-critical
// and we don't vendor all of the tendermint protos.
use crate::core::crypto::v1alpha1::ConsensusKey;

impl TypeUrl for tendermint::PublicKey {
    const TYPE_URL: &'static str = "/penumbra.core.crypto.v1alpha1.ConsensusKey";
}

impl DomainType for tendermint::PublicKey {
    type Proto = ConsensusKey;
}

impl From<tendermint::PublicKey> for crate::core::crypto::v1alpha1::ConsensusKey {
    fn from(v: tendermint::PublicKey) -> Self {
        Self {
            inner: v.to_bytes(),
        }
    }
}

impl TryFrom<crate::core::crypto::v1alpha1::ConsensusKey> for tendermint::PublicKey {
    type Error = anyhow::Error;
    fn try_from(value: crate::core::crypto::v1alpha1::ConsensusKey) -> Result<Self, Self::Error> {
        Self::from_raw_ed25519(value.inner.as_slice())
            .ok_or_else(|| anyhow::anyhow!("invalid ed25519 key"))
    }
}

// IBC-rs impls
extern crate ibc_types;

use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::channel::v1::Channel as RawChannel;
use ibc_proto::ibc::core::client::v1::Height as RawHeight;
use ibc_proto::ibc::core::connection::v1::ConnectionEnd as RawConnectionEnd;

use ibc_types::core::channel::ChannelEnd;
use ibc_types::core::client::Height;
use ibc_types::core::connection::ConnectionEnd;
use ibc_types::lightclients::tendermint::client_state::ClientState;

impl TypeUrl for ConnectionEnd {
    const TYPE_URL: &'static str = "/ibc.core.connection.v1.ConnectionEnd";
}
impl TypeUrl for ChannelEnd {
    const TYPE_URL: &'static str = "/ibc.core.channel.v1.Channel";
}
impl TypeUrl for Height {
    const TYPE_URL: &'static str = "/ibc.core.client.v1.Height";
}
impl TypeUrl for ibc_types::lightclients::tendermint::client_state::ClientState {
    const TYPE_URL: &'static str = "/ibc.lightclients.tendermint.v1.ClientState";
}
impl TypeUrl for ibc_types::lightclients::tendermint::consensus_state::ConsensusState {
    const TYPE_URL: &'static str = "/ibc.lightclients.tendermint.v1.ConsensusState";
}

impl DomainType for ConnectionEnd {
    type Proto = RawConnectionEnd;
}
impl DomainType for ChannelEnd {
    type Proto = RawChannel;
}
impl DomainType for Height {
    type Proto = RawHeight;
}

impl DomainType for ClientState {
    type Proto = Any;
}
impl DomainType for ibc_types::lightclients::tendermint::consensus_state::ConsensusState {
    type Proto = Any;
}

impl<T> From<T> for IbcAction
where
    T: ibc_types::DomainType + ibc_types::TypeUrl + Send + Sync + 'static,
    <T as TryFrom<<T as ibc_types::DomainType>::Proto>>::Error: Send + Sync + std::error::Error,
{
    fn from(v: T) -> Self {
        let value_bytes = v.encode_to_vec();
        let any = pbjson_types::Any {
            type_url: T::TYPE_URL.to_string(),
            value: value_bytes.into(),
        };

        Self {
            raw_action: Some(any),
        }
    }
}
