/// Plan specifies information about a planned upgrade and when it should occur.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Plan {
    /// Sets the name for the upgrade. This name will be used by the upgraded
    /// version of the software to apply any special "on-upgrade" commands during
    /// the first BeginBlock method after the upgrade is applied. It is also used
    /// to detect whether a software version can handle a given upgrade. If no
    /// upgrade handler with this name has been set in the software, it will be
    /// assumed that the software is out-of-date when the upgrade Time or Height is
    /// reached and the software will exit.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// Deprecated: Time based upgrades have been deprecated. Time based upgrade logic
    /// has been removed from the SDK.
    /// If this field is not empty, an error will be thrown.
    #[deprecated]
    #[prost(message, optional, tag = "2")]
    pub time: ::core::option::Option<::pbjson_types::Timestamp>,
    /// The height at which the upgrade must be performed.
    #[prost(int64, tag = "3")]
    pub height: i64,
    /// Any application specific upgrade info to be included on-chain
    /// such as a git commit that validators could automatically upgrade to
    #[prost(string, tag = "4")]
    pub info: ::prost::alloc::string::String,
    /// Deprecated: UpgradedClientState field has been deprecated. IBC upgrade logic has been
    /// moved to the IBC module in the sub module 02-client.
    /// If this field is not empty, an error will be thrown.
    #[deprecated]
    #[prost(message, optional, tag = "5")]
    pub upgraded_client_state: ::core::option::Option<::pbjson_types::Any>,
}
/// SoftwareUpgradeProposal is a gov Content type for initiating a software
/// upgrade.
/// Deprecated: This legacy proposal is deprecated in favor of Msg-based gov
/// proposals, see MsgSoftwareUpgrade.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SoftwareUpgradeProposal {
    /// title of the proposal
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    /// description of the proposal
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    /// plan of the proposal
    #[prost(message, optional, tag = "3")]
    pub plan: ::core::option::Option<Plan>,
}
/// CancelSoftwareUpgradeProposal is a gov Content type for cancelling a software
/// upgrade.
/// Deprecated: This legacy proposal is deprecated in favor of Msg-based gov
/// proposals, see MsgCancelUpgrade.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CancelSoftwareUpgradeProposal {
    /// title of the proposal
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    /// description of the proposal
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
}
/// ModuleVersion specifies a module and its consensus version.
///
/// Since: cosmos-sdk 0.43
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ModuleVersion {
    /// name of the app module
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// consensus version of the app module
    #[prost(uint64, tag = "2")]
    pub version: u64,
}
