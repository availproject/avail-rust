use primitive_types::H256;
use subxt::utils::MultiAddress;

use crate::{
	avail::{
		self,
		runtime_types::da_runtime::{impls::ProxyType, RuntimeCall},
	},
	client::Client,
	config::AccountId,
	SubmittableTransaction,
};

pub type ProxyCall = avail::proxy::calls::types::Proxy;
pub type AddProxyCall = avail::proxy::calls::types::AddProxy;
pub type RemoveProxyCall = avail::proxy::calls::types::RemoveProxy;
pub type RemoveProxiesCall = avail::proxy::calls::types::RemoveProxies;
pub type CreatePureCall = avail::proxy::calls::types::CreatePure;
pub type KillPureCall = avail::proxy::calls::types::KillPure;
pub type AnnounceCall = avail::proxy::calls::types::Announce;
pub type RemoveAnnouncementCall = avail::proxy::calls::types::RemoveAnnouncement;
pub type RejectAnnouncementCall = avail::proxy::calls::types::RejectAnnouncement;
pub type ProxyAnnouncedCall = avail::proxy::calls::types::ProxyAnnounced;

#[derive(Clone)]
pub struct Proxy {
	pub client: Client,
}

impl Proxy {
	/// Dispatch the given `call` from an account that the sender is authorised for through
	/// `add_proxy`.
	///
	/// The dispatch origin for this call must be _Signed_.
	///
	/// Parameters:
	/// - `real`: The account that the proxy will make a call on behalf of.
	/// - `force_proxy_type`: Specify the exact proxy type to be used and checked for this call.
	/// - `call`: The call to be made by the `real` account.
	pub fn proxy(
		&self,
		real: MultiAddress<AccountId, u32>,
		force_proxy_type: Option<ProxyType>,
		call: RuntimeCall,
	) -> SubmittableTransaction<ProxyCall> {
		let payload = avail::tx().proxy().proxy(real, force_proxy_type, call);
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	/// Register a proxy account for the sender that is able to make calls on its behalf.
	///
	/// The dispatch origin for this call must be _Signed_.
	///
	/// Parameters:
	/// - `delegate`: The account that the `caller` would like to make a proxy.
	/// - `proxy_type`: The permissions allowed for this proxy account.
	/// - `delay`: The announcement period required of the initial proxy. Will generally be
	/// zero.
	pub fn add_proxy(
		&self,
		delegate: MultiAddress<AccountId, u32>,
		proxy_type: ProxyType,
		delay: u32,
	) -> SubmittableTransaction<AddProxyCall> {
		let payload = avail::tx().proxy().add_proxy(delegate, proxy_type, delay);
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	/// Unregister a proxy account for the sender.
	///
	/// The dispatch origin for this call must be _Signed_.
	///
	/// Parameters:
	/// - `delegate`: The account that the `caller` would like to remove as a proxy.
	/// - `proxy_type`: The permissions currently enabled for the removed proxy account.
	/// - `delay`:  Will generally be zero.
	pub fn remove_proxy(
		&self,
		delegate: MultiAddress<AccountId, u32>,
		proxy_type: ProxyType,
		delay: u32,
	) -> SubmittableTransaction<RemoveProxyCall> {
		let payload = avail::tx().proxy().remove_proxy(delegate, proxy_type, delay);
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	/// Unregister all proxy accounts for the sender.
	///
	/// The dispatch origin for this call must be _Signed_.
	///
	/// WARNING: This may be called on accounts created by `pure`, however if done, then
	pub fn remove_proxies(&self) -> SubmittableTransaction<RemoveProxiesCall> {
		let payload = avail::tx().proxy().remove_proxies();
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	/// Spawn a fresh new account that is guaranteed to be otherwise inaccessible, and
	/// initialize it with a proxy of `proxy_type` for `origin` sender.
	///
	/// Requires a `Signed` origin.
	///
	///	- `proxy_type`: The type of the proxy that the sender will be registered as over the
	///	new account. This will almost always be the most permissive `ProxyType` possible to
	///	allow for maximum flexibility.
	///	- `index`: A disambiguation index, in case this is called multiple times in the same
	///	transaction (e.g. with `utility::batch`). Unless you're using `batch` you probably just
	///	want to use `0`.
	/// - `delay`: The announcement period required of the initial proxy. Will generally be
	///	zero.
	///
	/// Fails with `Duplicate` if this has already been called in this transaction, from the
	/// same sender, with the same parameters.
	///
	/// Fails if there are insufficient funds to pay for deposit.
	pub fn create_pure(&self, proxy_type: ProxyType, delay: u32, index: u16) -> SubmittableTransaction<CreatePureCall> {
		let payload = avail::tx().proxy().create_pure(proxy_type, delay, index);
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	/// Removes a previously spawned pure proxy.
	///
	/// WARNING: **All access to this account will be lost.** Any funds held in it will be
	/// inaccessible.
	///
	/// Requires a `Signed` origin, and the sender account must have been created by a call to
	/// `pure` with corresponding parameters.
	///
	/// - `spawner`: The account that originally called `pure` to create this account.
	/// - `index`: The disambiguation index originally passed to `pure`. Probably `0`.
	/// - `proxy_type`: The proxy type originally passed to `pure`.
	/// - `height`: The height of the chain when the call to `pure` was processed.
	/// - `ext_index`: The extrinsic index in which the call to `pure` was processed.
	///
	/// Fails with `NoPermission` in case the caller is not a previously created pure
	/// account whose `pure` call has corresponding parameters.
	pub fn kill_pure(
		&self,
		spawner: MultiAddress<AccountId, u32>,
		proxy_type: ProxyType,
		index: u16,
		height: u32,
		ext_index: u32,
	) -> SubmittableTransaction<KillPureCall> {
		let payload = avail::tx()
			.proxy()
			.kill_pure(spawner, proxy_type, index, height, ext_index);
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	/// Publish the hash of a proxy-call that will be made in the future.
	///
	/// This must be called some number of blocks before the corresponding `proxy` is attempted
	/// if the delay associated with the proxy relationship is greater than zero.
	///
	/// No more than `MaxPending` announcements may be made at any one time.
	///
	/// This will take a deposit of `AnnouncementDepositFactor` as well as
	/// `AnnouncementDepositBase` if there are no other pending announcements.
	///
	/// The dispatch origin for this call must be _Signed_ and a proxy of `real`.
	///
	/// Parameters:
	/// - `real`: The account that the proxy will make a call on behalf of.
	/// - `call_hash`: The hash of the call to be made by the `real` account.
	pub fn announce(
		&self,
		real: MultiAddress<AccountId, u32>,
		call_hash: H256,
	) -> SubmittableTransaction<AnnounceCall> {
		let payload = avail::tx().proxy().announce(real, call_hash);
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	/// Remove a given announcement.
	///
	/// May be called by a proxy account to remove a call they previously announced and return
	/// the deposit.
	///
	/// The dispatch origin for this call must be _Signed_.
	///
	/// Parameters:
	/// - `real`: The account that the proxy will make a call on behalf of.
	/// - `call_hash`: The hash of the call to be made by the `real` account.
	pub fn remove_announce(
		&self,
		real: MultiAddress<AccountId, u32>,
		call_hash: H256,
	) -> SubmittableTransaction<RemoveAnnouncementCall> {
		let payload = avail::tx().proxy().remove_announcement(real, call_hash);
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	/// Remove the given announcement of a delegate.
	///
	/// May be called by a target (proxied) account to remove a call that one of their delegates
	/// (`delegate`) has announced they want to execute. The deposit is returned.
	///
	/// The dispatch origin for this call must be _Signed_.
	///
	/// Parameters:
	/// - `delegate`: The account that previously announced the call.
	/// - `call_hash`: The hash of the call to be made.
	pub fn reject_announcement(
		&self,
		delegate: MultiAddress<AccountId, u32>,
		call_hash: H256,
	) -> SubmittableTransaction<RejectAnnouncementCall> {
		let payload = avail::tx().proxy().reject_announcement(delegate, call_hash);
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	/// Dispatch the given `call` from an account that the sender is authorized for through
	/// `add_proxy`.
	///
	/// Removes any corresponding announcement(s).
	///
	/// The dispatch origin for this call must be _Signed_.
	///
	/// Parameters:
	/// - `Real`: The account that the proxy will make a call on behalf of.
	/// - `ForceProxyType`: Specify the exact proxy type to be used and checked for this call.
	/// - `Call`: The call to be made by the `real` account.
	pub fn proxy_announced(
		&self,
		delegate: MultiAddress<AccountId, u32>,
		real: MultiAddress<AccountId, u32>,
		force_proxy_type: Option<ProxyType>,
		call: RuntimeCall,
	) -> SubmittableTransaction<ProxyAnnouncedCall> {
		let payload = avail::tx()
			.proxy()
			.proxy_announced(delegate, real, force_proxy_type, call);
		SubmittableTransaction::new(self.client.clone(), payload)
	}
}
