// Copyright 2021-2022 Farcaster Devs
//
// This library is free software; you can redistribute it and/or
// modify it under the terms of the GNU Lesser General Public
// License as published by the Free Software Foundation; either
// version 3 of the License, or (at your option) any later version.
//
// This library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public
// License along with this library; if not, write to the Free Software
// Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301, USA

//! Protocol messages exchanged between swap daemons at each step of the swap protocol. These
//! messages are untrusted and must be validated uppon reception by each swap participant.

use std::fmt;
use std::io;

use crate::consensus::{self, CanonicalBytes, Decodable, Encodable};
use crate::crypto::{Commit, SharedKeyId, TaggedElement};
use crate::protocol::Parameters;
use crate::protocol::{verify_vec_of_commitments, CoreArbitratingTransactions};
use crate::swap::SwapId;
use crate::Error;

/// Forces Alice to commit to the result of her cryptographic setup before receiving Bob's setup.
/// This is done to remove adaptive behavior in the cryptographic parameters.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitAliceParameters<C> {
    /// The swap identifier related to this message.
    pub swap_id: SwapId,
    /// Commitment to the buy public key.
    pub buy: C,
    /// Commitment to the cancel public key.
    pub cancel: C,
    /// Commitment to the refund public key.
    pub refund: C,
    /// Commitment to the punish public key.
    pub punish: C,
    /// Commitment to the adaptor public key.
    pub adaptor: C,
    /// Commitments to the extra arbitrating public keys.
    pub extra_arbitrating_keys: Vec<TaggedElement<u16, C>>,
    /// Commitments to the arbitrating shared keys.
    pub arbitrating_shared_keys: Vec<TaggedElement<SharedKeyId, C>>,
    /// Commitment to the spend public key.
    pub spend: C,
    /// Commitments to the extra accordant public keys.
    pub extra_accordant_keys: Vec<TaggedElement<u16, C>>,
    /// Commitments to the accordant shared keys.
    pub accordant_shared_keys: Vec<TaggedElement<SharedKeyId, C>>,
}

impl<C> CommitAliceParameters<C>
where
    C: Eq + Clone + CanonicalBytes,
{
    pub fn verify_with_reveal<Pk, Qk, Rk, Sk, Addr>(
        &self,
        wallet: &impl Commit<C>,
        reveal: RevealAliceParameters<Pk, Qk, Rk, Sk, Addr>,
    ) -> Result<(), Error>
    where
        Pk: CanonicalBytes,
        Qk: CanonicalBytes,
        Rk: CanonicalBytes,
        Sk: CanonicalBytes,
        Addr: CanonicalBytes,
    {
        wallet.validate(reveal.buy.as_canonical_bytes(), self.buy.clone())?;
        wallet.validate(reveal.cancel.as_canonical_bytes(), self.cancel.clone())?;
        wallet.validate(reveal.refund.as_canonical_bytes(), self.refund.clone())?;
        wallet.validate(reveal.punish.as_canonical_bytes(), self.punish.clone())?;
        wallet.validate(reveal.adaptor.as_canonical_bytes(), self.adaptor.clone())?;
        verify_vec_of_commitments(
            wallet,
            reveal.extra_arbitrating_keys,
            &self.extra_arbitrating_keys,
        )?;
        verify_vec_of_commitments(
            wallet,
            reveal.arbitrating_shared_keys,
            &self.arbitrating_shared_keys,
        )?;
        wallet.validate(reveal.spend.as_canonical_bytes(), self.spend.clone())?;
        verify_vec_of_commitments(
            wallet,
            reveal.extra_accordant_keys,
            &self.extra_accordant_keys,
        )?;
        verify_vec_of_commitments(
            wallet,
            reveal.accordant_shared_keys,
            &self.accordant_shared_keys,
        )
    }
}

impl<C> fmt::Display for CommitAliceParameters<C>
where
    C: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<C> Encodable for CommitAliceParameters<C>
where
    C: CanonicalBytes,
{
    fn consensus_encode<W: io::Write>(&self, s: &mut W) -> Result<usize, io::Error> {
        let mut len = self.swap_id.consensus_encode(s)?;
        len += self.buy.as_canonical_bytes().consensus_encode(s)?;
        len += self.cancel.as_canonical_bytes().consensus_encode(s)?;
        len += self.refund.as_canonical_bytes().consensus_encode(s)?;
        len += self.punish.as_canonical_bytes().consensus_encode(s)?;
        len += self.adaptor.as_canonical_bytes().consensus_encode(s)?;
        len += self.extra_arbitrating_keys.consensus_encode(s)?;
        len += self.arbitrating_shared_keys.consensus_encode(s)?;
        len += self.spend.as_canonical_bytes().consensus_encode(s)?;
        len += self.extra_accordant_keys.consensus_encode(s)?;
        Ok(len + self.accordant_shared_keys.consensus_encode(s)?)
    }
}

impl<C> Decodable for CommitAliceParameters<C>
where
    C: CanonicalBytes,
{
    fn consensus_decode<D: io::Read>(d: &mut D) -> Result<Self, consensus::Error> {
        Ok(Self {
            swap_id: Decodable::consensus_decode(d)?,
            buy: C::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
            cancel: C::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
            refund: C::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
            punish: C::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
            adaptor: C::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
            extra_arbitrating_keys: Decodable::consensus_decode(d)?,
            arbitrating_shared_keys: Decodable::consensus_decode(d)?,
            spend: C::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
            extra_accordant_keys: Decodable::consensus_decode(d)?,
            accordant_shared_keys: Decodable::consensus_decode(d)?,
        })
    }
}

impl_strict_encoding!(CommitAliceParameters<C>, C: CanonicalBytes);

/// Forces Bob to commit to the result of his cryptographic setup before receiving Alice's setup.
/// This is done to remove adaptive behavior in the cryptographic parameters.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitBobParameters<C> {
    /// The swap identifier related to this message.
    pub swap_id: SwapId,
    /// Commitment to the buy public key.
    pub buy: C,
    /// Commitment to the cancel public key.
    pub cancel: C,
    /// Commitment to the refund public key.
    pub refund: C,
    /// Commitment to the adaptor public key.
    pub adaptor: C,
    /// Commitments to the extra arbitrating public keys.
    pub extra_arbitrating_keys: Vec<TaggedElement<u16, C>>,
    /// Commitments to the arbitrating shared keys.
    pub arbitrating_shared_keys: Vec<TaggedElement<SharedKeyId, C>>,
    /// Commitment to the spend public key.
    pub spend: C,
    /// Commitments to the extra accordant public keys.
    pub extra_accordant_keys: Vec<TaggedElement<u16, C>>,
    /// Commitments to the accordant shared keys.
    pub accordant_shared_keys: Vec<TaggedElement<SharedKeyId, C>>,
}

impl<C> CommitBobParameters<C>
where
    C: Eq + Clone + CanonicalBytes,
{
    pub fn verify_with_reveal<Pk, Qk, Rk, Sk, Addr>(
        &self,
        wallet: &impl Commit<C>,
        reveal: RevealBobParameters<Pk, Qk, Rk, Sk, Addr>,
    ) -> Result<(), Error>
    where
        Pk: CanonicalBytes,
        Qk: CanonicalBytes,
        Rk: CanonicalBytes,
        Sk: CanonicalBytes,
        Addr: CanonicalBytes,
    {
        wallet.validate(reveal.buy.as_canonical_bytes(), self.buy.clone())?;
        wallet.validate(reveal.cancel.as_canonical_bytes(), self.cancel.clone())?;
        wallet.validate(reveal.refund.as_canonical_bytes(), self.refund.clone())?;
        wallet.validate(reveal.adaptor.as_canonical_bytes(), self.adaptor.clone())?;
        verify_vec_of_commitments(
            wallet,
            reveal.extra_arbitrating_keys,
            &self.extra_arbitrating_keys,
        )?;
        verify_vec_of_commitments(
            wallet,
            reveal.arbitrating_shared_keys,
            &self.arbitrating_shared_keys,
        )?;
        wallet.validate(reveal.spend.as_canonical_bytes(), self.spend.clone())?;
        verify_vec_of_commitments(
            wallet,
            reveal.extra_accordant_keys,
            &self.extra_accordant_keys,
        )?;
        verify_vec_of_commitments(
            wallet,
            reveal.accordant_shared_keys,
            &self.accordant_shared_keys,
        )
    }
}

impl<C> fmt::Display for CommitBobParameters<C>
where
    C: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<C> Encodable for CommitBobParameters<C>
where
    C: CanonicalBytes,
{
    fn consensus_encode<W: io::Write>(&self, s: &mut W) -> Result<usize, io::Error> {
        let mut len = self.swap_id.consensus_encode(s)?;
        len += self.buy.as_canonical_bytes().consensus_encode(s)?;
        len += self.cancel.as_canonical_bytes().consensus_encode(s)?;
        len += self.refund.as_canonical_bytes().consensus_encode(s)?;
        len += self.adaptor.as_canonical_bytes().consensus_encode(s)?;
        len += self.extra_arbitrating_keys.consensus_encode(s)?;
        len += self.arbitrating_shared_keys.consensus_encode(s)?;
        len += self.spend.as_canonical_bytes().consensus_encode(s)?;
        len += self.extra_accordant_keys.consensus_encode(s)?;
        Ok(len + self.accordant_shared_keys.consensus_encode(s)?)
    }
}

impl<C> Decodable for CommitBobParameters<C>
where
    C: CanonicalBytes,
{
    fn consensus_decode<D: io::Read>(d: &mut D) -> Result<Self, consensus::Error> {
        Ok(Self {
            swap_id: Decodable::consensus_decode(d)?,
            buy: C::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
            cancel: C::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
            refund: C::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
            adaptor: C::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
            extra_arbitrating_keys: Decodable::consensus_decode(d)?,
            arbitrating_shared_keys: Decodable::consensus_decode(d)?,
            spend: C::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
            extra_accordant_keys: Decodable::consensus_decode(d)?,
            accordant_shared_keys: Decodable::consensus_decode(d)?,
        })
    }
}

impl_strict_encoding!(CommitBobParameters<C>, C: CanonicalBytes);

/// Reveals the zero-knowledge proof for the discrete logarithm across curves.
#[derive(Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevealProof<Pr> {
    /// The swap identifier related to this message.
    pub swap_id: SwapId,
    /// Reveal the cross-group discrete logarithm zero-knowledge proof.
    pub proof: Pr,
}

impl<Pr> fmt::Debug for RevealProof<Pr> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RevealProof")
            .field("swap_id", &self.swap_id)
            .field("proof", &"..")
            .finish()
    }
}

impl<Pr> fmt::Display for RevealProof<Pr>
where
    Pr: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RevealProof {{ swap_id: {}, proof: .. }}", self.swap_id)
    }
}

impl<Pr> Encodable for RevealProof<Pr>
where
    Pr: CanonicalBytes,
{
    fn consensus_encode<W: io::Write>(&self, s: &mut W) -> Result<usize, io::Error> {
        let len = self.swap_id.consensus_encode(s)?;
        Ok(len + self.proof.as_canonical_bytes().consensus_encode(s)?)
    }
}

impl<Pr> Decodable for RevealProof<Pr>
where
    Pr: CanonicalBytes,
{
    fn consensus_decode<D: io::Read>(d: &mut D) -> Result<Self, consensus::Error> {
        Ok(Self {
            swap_id: Decodable::consensus_decode(d)?,
            proof: Pr::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
        })
    }
}

impl_strict_encoding!(RevealProof<Pr>, Pr: CanonicalBytes);

/// Reveals the parameters commited by the [`CommitAliceParameters`] protocol message.
///
/// - `Addr` the arbitrating address type
/// - `Pk` the arbitrating Public Key type
/// - `Rk` the arbitrating Shared Secret Key type
/// - `Qk` the accordant Public Key type
/// - `Sk` the accordant Shared Secret Key type
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevealAliceParameters<Pk, Qk, Rk, Sk, Addr> {
    /// The swap identifier related to this message.
    pub swap_id: SwapId,
    /// Reveal the buy public key.
    pub buy: Pk,
    /// Reveal the cancel public key.
    pub cancel: Pk,
    /// Reveal the refund public key.
    pub refund: Pk,
    /// Reveal the punish public key.
    pub punish: Pk,
    /// Reveal the adaptor public key.
    pub adaptor: Pk,
    /// Reveal the vector of extra arbitrating public keys.
    pub extra_arbitrating_keys: Vec<TaggedElement<u16, Pk>>,
    /// Reveal the vector of extra arbitrating shared keys.
    pub arbitrating_shared_keys: Vec<TaggedElement<SharedKeyId, Rk>>,
    /// Reveal the spend public key.
    pub spend: Qk,
    /// Reveal the vector of extra accordant public keys.
    pub extra_accordant_keys: Vec<TaggedElement<u16, Qk>>,
    /// Reveal the vector of extra accordant shared keys.
    pub accordant_shared_keys: Vec<TaggedElement<SharedKeyId, Sk>>,
    /// Reveal the destination address.
    pub address: Addr,
}

impl<Pk, Qk, Rk, Sk, Addr> RevealAliceParameters<Pk, Qk, Rk, Sk, Addr> {
    pub fn into_parameters<Ti, F, Pr>(self) -> Parameters<Pk, Qk, Rk, Sk, Addr, Ti, F, Pr> {
        Parameters {
            buy: self.buy,
            cancel: self.cancel,
            refund: self.refund,
            punish: Some(self.punish),
            adaptor: self.adaptor,
            extra_arbitrating_keys: self.extra_arbitrating_keys,
            arbitrating_shared_keys: self.arbitrating_shared_keys,
            spend: self.spend,
            extra_accordant_keys: self.extra_accordant_keys,
            accordant_shared_keys: self.accordant_shared_keys,
            proof: None,
            destination_address: self.address,
            cancel_timelock: None,
            punish_timelock: None,
            fee_strategy: None,
        }
    }
}

impl<Pk, Qk, Rk, Sk, Addr> fmt::Display for RevealAliceParameters<Pk, Qk, Rk, Sk, Addr>
where
    Pk: fmt::Debug,
    Qk: fmt::Debug,
    Rk: fmt::Debug,
    Sk: fmt::Debug,
    Addr: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<Pk, Qk, Rk, Sk, Addr> Encodable for RevealAliceParameters<Pk, Qk, Rk, Sk, Addr>
where
    Pk: CanonicalBytes,
    Qk: CanonicalBytes,
    Rk: CanonicalBytes,
    Sk: CanonicalBytes,
    Addr: CanonicalBytes,
{
    fn consensus_encode<W: io::Write>(&self, s: &mut W) -> Result<usize, io::Error> {
        let mut len = self.swap_id.consensus_encode(s)?;
        len += self.buy.as_canonical_bytes().consensus_encode(s)?;
        len += self.cancel.as_canonical_bytes().consensus_encode(s)?;
        len += self.refund.as_canonical_bytes().consensus_encode(s)?;
        len += self.punish.as_canonical_bytes().consensus_encode(s)?;
        len += self.adaptor.as_canonical_bytes().consensus_encode(s)?;
        len += self.extra_arbitrating_keys.consensus_encode(s)?;
        len += self.arbitrating_shared_keys.consensus_encode(s)?;
        // this can go?
        len += self.spend.as_canonical_bytes().consensus_encode(s)?;
        len += self.extra_accordant_keys.consensus_encode(s)?;
        len += self.accordant_shared_keys.consensus_encode(s)?;
        Ok(len + self.address.as_canonical_bytes().consensus_encode(s)?)
    }
}

impl<Pk, Qk, Rk, Sk, Addr> Decodable for RevealAliceParameters<Pk, Qk, Rk, Sk, Addr>
where
    Pk: CanonicalBytes,
    Qk: CanonicalBytes,
    Rk: CanonicalBytes,
    Sk: CanonicalBytes,
    Addr: CanonicalBytes,
{
    fn consensus_decode<D: io::Read>(d: &mut D) -> Result<Self, consensus::Error> {
        Ok(Self {
            swap_id: Decodable::consensus_decode(d)?,
            buy: Pk::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
            cancel: Pk::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
            refund: Pk::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
            punish: Pk::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
            adaptor: Pk::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
            extra_arbitrating_keys: Decodable::consensus_decode(d)?,
            arbitrating_shared_keys: Decodable::consensus_decode(d)?,
            spend: Qk::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
            extra_accordant_keys: Decodable::consensus_decode(d)?,
            accordant_shared_keys: Decodable::consensus_decode(d)?,
            address: Addr::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
        })
    }
}

impl_strict_encoding!(RevealAliceParameters<Pk, Qk, Rk, Sk, Addr>, Pk: CanonicalBytes, Qk: CanonicalBytes, Rk: CanonicalBytes, Sk: CanonicalBytes, Addr: CanonicalBytes);

/// Reveals the parameters commited by the [`CommitBobParameters`] protocol message.
///
/// - `Addr` the arbitrating address type
/// - `Pk` the arbitrating Public Key type
/// - `Rk` the arbitrating Shared Secret Key type
/// - `Qk` the accordant Public Key type
/// - `Sk` the accordant Shared Secret Key type
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevealBobParameters<Pk, Qk, Rk, Sk, Addr> {
    /// The swap identifier related to this message.
    pub swap_id: SwapId,
    /// Reveal the buy public key.
    pub buy: Pk,
    /// Reveal the cancel public key.
    pub cancel: Pk,
    /// Reveal the refund public key.
    pub refund: Pk,
    /// Reveal the adaptor public key.
    pub adaptor: Pk,
    /// Reveal the vector of extra arbitrating public keys.
    pub extra_arbitrating_keys: Vec<TaggedElement<u16, Pk>>,
    /// Reveal the vector of extra arbitrating shared keys.
    pub arbitrating_shared_keys: Vec<TaggedElement<SharedKeyId, Rk>>,
    /// Reveal the spend public key.
    pub spend: Qk,
    /// Reveal the vector of extra accordant public keys.
    pub extra_accordant_keys: Vec<TaggedElement<u16, Qk>>,
    /// Reveal the vector of extra accordant shared keys.
    pub accordant_shared_keys: Vec<TaggedElement<SharedKeyId, Sk>>,
    /// The refund Bitcoin address.
    pub address: Addr,
}

impl<Pk, Qk, Rk, Sk, Addr> RevealBobParameters<Pk, Qk, Rk, Sk, Addr> {
    pub fn into_parameters<Ti, F, Pr>(self) -> Parameters<Pk, Qk, Rk, Sk, Addr, Ti, F, Pr> {
        Parameters {
            buy: self.buy,
            cancel: self.cancel,
            refund: self.refund,
            punish: None,
            adaptor: self.adaptor,
            extra_arbitrating_keys: self.extra_arbitrating_keys,
            arbitrating_shared_keys: self.arbitrating_shared_keys,
            spend: self.spend,
            extra_accordant_keys: self.extra_accordant_keys,
            accordant_shared_keys: self.accordant_shared_keys,
            proof: None,
            destination_address: self.address,
            cancel_timelock: None,
            punish_timelock: None,
            fee_strategy: None,
        }
    }
}

impl<Pk, Qk, Rk, Sk, Addr> fmt::Display for RevealBobParameters<Pk, Qk, Rk, Sk, Addr>
where
    Pk: fmt::Debug,
    Qk: fmt::Debug,
    Rk: fmt::Debug,
    Sk: fmt::Debug,
    Addr: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<Pk, Qk, Rk, Sk, Addr> Encodable for RevealBobParameters<Pk, Qk, Rk, Sk, Addr>
where
    Pk: CanonicalBytes,
    Qk: CanonicalBytes,
    Rk: CanonicalBytes,
    Sk: CanonicalBytes,
    Addr: CanonicalBytes,
{
    fn consensus_encode<W: io::Write>(&self, s: &mut W) -> Result<usize, io::Error> {
        let mut len = self.swap_id.consensus_encode(s)?;
        len += self.buy.as_canonical_bytes().consensus_encode(s)?;
        len += self.cancel.as_canonical_bytes().consensus_encode(s)?;
        len += self.refund.as_canonical_bytes().consensus_encode(s)?;
        len += self.adaptor.as_canonical_bytes().consensus_encode(s)?;
        len += self.extra_arbitrating_keys.consensus_encode(s)?;
        len += self.arbitrating_shared_keys.consensus_encode(s)?;
        len += self.spend.as_canonical_bytes().consensus_encode(s)?;
        len += self.extra_accordant_keys.consensus_encode(s)?;
        len += self.accordant_shared_keys.consensus_encode(s)?;
        Ok(len + self.address.as_canonical_bytes().consensus_encode(s)?)
    }
}

impl<Pk, Qk, Rk, Sk, Addr> Decodable for RevealBobParameters<Pk, Qk, Rk, Sk, Addr>
where
    Pk: CanonicalBytes,
    Qk: CanonicalBytes,
    Rk: CanonicalBytes,
    Sk: CanonicalBytes,
    Addr: CanonicalBytes,
{
    fn consensus_decode<D: io::Read>(d: &mut D) -> Result<Self, consensus::Error> {
        Ok(Self {
            swap_id: Decodable::consensus_decode(d)?,
            buy: Pk::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
            cancel: Pk::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
            refund: Pk::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
            adaptor: Pk::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
            extra_arbitrating_keys: Decodable::consensus_decode(d)?,
            arbitrating_shared_keys: Decodable::consensus_decode(d)?,
            spend: Qk::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
            extra_accordant_keys: Decodable::consensus_decode(d)?,
            accordant_shared_keys: Decodable::consensus_decode(d)?,
            address: Addr::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
        })
    }
}

impl_strict_encoding!(RevealBobParameters<Pk, Qk, Rk, Sk, Addr>, Pk: CanonicalBytes, Qk: CanonicalBytes, Rk: CanonicalBytes, Sk: CanonicalBytes, Addr: CanonicalBytes);

/// Sends the [`Lockable`], [`Cancelable`] and [`Refundable`] arbritrating transactions from
/// [`SwapRole::Bob`] to [`SwapRole::Alice`], as well as Bob's signature for the [`Cancelable`]
/// transaction.
///
/// [`SwapRole::Alice`]: crate::role::SwapRole::Alice
/// [`SwapRole::Bob`]: crate::role::SwapRole::Bob
/// [`Lockable`]: crate::transaction::Lockable
/// [`Cancelable`]: crate::transaction::Cancelable
/// [`Refundable`]: crate::transaction::Refundable
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreArbitratingSetup<Px, Sig> {
    /// The swap identifier related to this message.
    pub swap_id: SwapId,
    /// The arbitrating `lock (b)` transaction.
    pub lock: Px,
    /// The arbitrating `cancel (d)` transaction.
    pub cancel: Px,
    /// The arbitrating `refund (e)` transaction.
    pub refund: Px,
    /// The `Bc` `cancel (d)` signature.
    pub cancel_sig: Sig,
}

impl<Px, Sig> CoreArbitratingSetup<Px, Sig> {
    /// Transform the arbitrating setup into a core arbitrating transaction structure used in
    /// protocol methods on Alice and Bob.
    pub fn into_arbitrating_tx(self) -> CoreArbitratingTransactions<Px> {
        CoreArbitratingTransactions {
            lock: self.lock,
            cancel: self.cancel,
            refund: self.refund,
        }
    }
}

impl<Px, Sig> fmt::Display for CoreArbitratingSetup<Px, Sig>
where
    Px: fmt::Debug,
    Sig: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<Px, Sig> Encodable for CoreArbitratingSetup<Px, Sig>
where
    Px: CanonicalBytes,
    Sig: CanonicalBytes,
{
    fn consensus_encode<W: io::Write>(&self, s: &mut W) -> Result<usize, io::Error> {
        let mut len = self.swap_id.consensus_encode(s)?;
        len += self.lock.as_canonical_bytes().consensus_encode(s)?;
        len += self.cancel.as_canonical_bytes().consensus_encode(s)?;
        len += self.refund.as_canonical_bytes().consensus_encode(s)?;
        Ok(len + self.cancel_sig.as_canonical_bytes().consensus_encode(s)?)
    }
}

impl<Px, Sig> Decodable for CoreArbitratingSetup<Px, Sig>
where
    Px: CanonicalBytes,
    Sig: CanonicalBytes,
{
    fn consensus_decode<D: io::Read>(d: &mut D) -> Result<Self, consensus::Error> {
        Ok(Self {
            swap_id: Decodable::consensus_decode(d)?,
            lock: Px::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
            cancel: Px::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
            refund: Px::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
            cancel_sig: Sig::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
        })
    }
}

impl_strict_encoding!(CoreArbitratingSetup<Px, Sig>, Px: CanonicalBytes, Sig: CanonicalBytes);

/// Protocol message is intended to transmit [`SwapRole::Alice`]'s signature for the [`Cancelable`]
/// transaction and Alice's adaptor signature for the [`Refundable`] transaction. Uppon reception
/// [`SwapRole::Bob`] must validate the signatures.
///
/// [`SwapRole::Alice`]: crate::role::SwapRole::Alice
/// [`SwapRole::Bob`]: crate::role::SwapRole::Bob
/// [`Cancelable`]: crate::transaction::Cancelable
/// [`Refundable`]: crate::transaction::Refundable
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefundProcedureSignatures<Sig, EncSig> {
    /// The swap identifier related to this message.
    pub swap_id: SwapId,
    /// The `Ac` `cancel (d)` signature.
    pub cancel_sig: Sig,
    /// The `Ar(Tb)` `refund (e)` adaptor signature.
    pub refund_adaptor_sig: EncSig,
}

impl<Sig, EncSig> fmt::Display for RefundProcedureSignatures<Sig, EncSig>
where
    Sig: fmt::Debug,
    EncSig: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<Sig, EncSig> Encodable for RefundProcedureSignatures<Sig, EncSig>
where
    Sig: CanonicalBytes,
    EncSig: CanonicalBytes,
{
    fn consensus_encode<W: io::Write>(&self, s: &mut W) -> Result<usize, io::Error> {
        let mut len = self.swap_id.consensus_encode(s)?;
        len += self.cancel_sig.as_canonical_bytes().consensus_encode(s)?;
        Ok(len
            + self
                .refund_adaptor_sig
                .as_canonical_bytes()
                .consensus_encode(s)?)
    }
}

impl<Sig, EncSig> Decodable for RefundProcedureSignatures<Sig, EncSig>
where
    Sig: CanonicalBytes,
    EncSig: CanonicalBytes,
{
    fn consensus_decode<D: io::Read>(d: &mut D) -> Result<Self, consensus::Error> {
        Ok(Self {
            swap_id: Decodable::consensus_decode(d)?,
            cancel_sig: Sig::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
            refund_adaptor_sig: EncSig::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
        })
    }
}

impl_strict_encoding!(RefundProcedureSignatures<Sig, EncSig>, Sig: CanonicalBytes, EncSig: CanonicalBytes);

/// Protocol message intended to transmit [`SwapRole::Bob`]'s adaptor signature for the [`Buyable`]
/// transaction and the transaction itself. Uppon reception Alice must validate the transaction and
/// the adaptor signature.
///
/// [`SwapRole::Bob`]: crate::role::SwapRole::Bob
/// [`Buyable`]: crate::transaction::Buyable
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuyProcedureSignature<Px, EncSig> {
    /// The swap identifier related to this message.
    pub swap_id: SwapId,
    /// The arbitrating `buy (c)` transaction.
    pub buy: Px,
    /// The `Bb(Ta)` `buy (c)` adaptor signature.
    pub buy_adaptor_sig: EncSig,
}

impl<Px, EncSig> fmt::Display for BuyProcedureSignature<Px, EncSig>
where
    Px: fmt::Debug,
    EncSig: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<Px, EncSig> Encodable for BuyProcedureSignature<Px, EncSig>
where
    Px: CanonicalBytes,
    EncSig: CanonicalBytes,
{
    fn consensus_encode<W: io::Write>(&self, s: &mut W) -> Result<usize, io::Error> {
        let mut len = self.swap_id.consensus_encode(s)?;
        len += self.buy.as_canonical_bytes().consensus_encode(s)?;
        Ok(len
            + self
                .buy_adaptor_sig
                .as_canonical_bytes()
                .consensus_encode(s)?)
    }
}

impl<Px, EncSig> Decodable for BuyProcedureSignature<Px, EncSig>
where
    Px: CanonicalBytes,
    EncSig: CanonicalBytes,
{
    fn consensus_decode<D: io::Read>(d: &mut D) -> Result<Self, consensus::Error> {
        Ok(Self {
            swap_id: Decodable::consensus_decode(d)?,
            buy: Px::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
            buy_adaptor_sig: EncSig::from_canonical_bytes(unwrap_vec_ref!(d).as_ref())?,
        })
    }
}

impl_strict_encoding!(BuyProcedureSignature<Px, EncSig>, Px: consensus::CanonicalBytes, EncSig: consensus::CanonicalBytes);

/// Optional courtesy message from either [`SwapRole`] to inform the counterparty
/// that they have aborted the swap with an `OPTIONAL` message body to provide the reason.
///
/// [`SwapRole`]: crate::role::SwapRole
#[derive(Clone, Debug, Hash, Display, Serialize, Deserialize)]
#[display(Debug)]
pub struct Abort {
    /// The swap identifier related to this message.
    pub swap_id: SwapId,
    /// OPTIONAL `body`: error string.
    pub error_body: Option<String>,
}

impl Encodable for Abort {
    fn consensus_encode<W: io::Write>(&self, s: &mut W) -> Result<usize, io::Error> {
        let len = self.swap_id.consensus_encode(s)?;
        Ok(len + self.error_body.consensus_encode(s)?)
    }
}

impl Decodable for Abort {
    fn consensus_decode<D: io::Read>(d: &mut D) -> Result<Self, consensus::Error> {
        Ok(Self {
            swap_id: Decodable::consensus_decode(d)?,
            error_body: Option::<String>::consensus_decode(d)?,
        })
    }
}

impl_strict_encoding!(Abort);
