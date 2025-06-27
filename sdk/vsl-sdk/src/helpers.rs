use alloy::consensus::Signed;
use alloy::primitives::{Address, SignatureError, eip191_hash_message};
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::{Error, Signature, SignerSync};
use alloy_rlp::Encodable;

pub trait HasSender {
    /// The sender of this transaction
    /// Commonly the sender is recovered from a signature on the transaction instead.
    fn sender(&self) -> Option<Address> {
        None
    }
}

impl<T> HasSender for Signed<T>
where
    T: Encodable,
{
    fn sender(&self) -> Option<Address> {
        let mut msg: Vec<u8> = Vec::new();
        self.tx().encode(&mut msg);
        self.signature().recover_address_from_msg(msg).ok()
    }
}

/// Trait to turn an [Encodable] object into a [Signed] one.
///
/// The object is encoded intro a stream of bytes, then prefixed according to
/// the [EIP-191](https://eips.ethereum.org/EIPS/eip-191) standard
/// (to allow using the [Signed] methods for extracting the address)
pub trait IntoSigned: Sized + Encodable + HasSender {
    fn into_signed(self, signer: &PrivateKeySigner) -> Result<Signed<Self>, Error> {
        let mut buf: Vec<u8> = Vec::new();
        self.encode(&mut buf);
        let hash = eip191_hash_message(buf);
        let sig = signer.sign_hash_sync(&hash)?;
        Ok(Signed::new_unchecked(self, sig, hash))
    }

    fn recover_address(&self, sig: &Signature) -> Result<Address, SignatureError> {
        let mut msg: Vec<u8> = Vec::new();
        self.encode(&mut msg);
        sig.recover_address_from_msg(&msg)
    }

    fn check(signed: &Signed<Self>) -> bool {
        let data = signed.tx();
        let signer = match data.recover_address(signed.signature()) {
            Ok(address) => address,
            Err(_) => return false,
        };
        let Some(address) = data.sender() else {
            return false;
        };
        return signer == address;
    }

    fn check_and_strip_signature(signed: Signed<Self>) -> Option<Self> {
        if !Self::check(&signed) {
            return None;
        };
        Some(signed.strip_signature())
    }
}

/// Implement the (undocumented) [RlpEcdsaDecodableTx] and [RlpEcdsaEncodableTx] traits
/// needed for some uses of [Signed], by making a dummy implementation of [Typed2718] with
/// a code of 0u8, and otherwise forwarding to [alloy_rlp::Decodable] and [alloy_rlp::Encodable].
#[macro_export]
macro_rules! impl_rlp_ecdsa_glue {
    ($type:ty) => {
        impl Typed2718 for $type {
            fn ty(&self) -> u8 {
                0
            }
        }
        impl RlpEcdsaEncodableTx for $type {
            fn rlp_encoded_fields_length(&self) -> usize {
                self.length()
            }

            fn rlp_encode_fields(&self, out: &mut dyn alloy_rlp::BufMut) {
                self.encode(out);
            }
        }

        impl RlpEcdsaDecodableTx for $type {
            const DEFAULT_TX_TYPE: u8 = 0;

            fn rlp_decode_fields(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
                Decodable::decode(buf)
            }
        }

        impl IntoSigned for $type {}
    };
}
