/*
 *  This file is part of OBS Controller.
 *  Copyright (C) 2020 Beezig Team
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
#![allow(unused)]

use ed25519_dalek::{Keypair, PublicKey, Signature, Verifier};
use rand_core::OsRng;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Visitor;
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize)]
struct AppMetadata {
    uuid: u128,
    name: String,
    #[serde(serialize_with = "ser_pubkey", deserialize_with = "deser_pubkey")]
    pub_key: PublicKey
}

impl AppMetadata {
    pub fn register(uuid: u128, name: String) -> (AppMetadata, Keypair) {
        let pair = Keypair::generate(&mut OsRng);
        (AppMetadata {
            uuid, name,
            pub_key: pair.public
        }, pair)
    }

    pub fn validate_message(&self, message: &[u8], signature: [u8; 64]) -> bool {
        let mut hash = Sha256::new();
        hash.update(message);
        self.pub_key.verify(hash.finalize().as_slice(), &Signature::new(signature)).is_ok()
    }
}

fn ser_pubkey<S>(pub_key: &PublicKey, ser: S) -> Result<S::Ok, S::Error> where S: Serializer {
    ser.serialize_bytes(pub_key.as_bytes())
}

struct PubkeyVisitor;

impl<'de> Visitor<'de> for PubkeyVisitor {
    type Value = PublicKey;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("public key bytes")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E> where E: serde::de::Error {
        PublicKey::from_bytes(v).map_err(|_| serde::de::Error::custom("Invalid public key".to_string()))
    }
}

fn deser_pubkey<'de, D>(deser: D) -> Result<PublicKey, D::Error> where D: Deserializer<'de> {
    deser.deserialize_bytes(PubkeyVisitor)
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::{Signature, Signer};
    use super::AppMetadata;
    use sha2::{Sha256, Digest};

    #[test]
    pub fn validate() {
        let (app, key) = AppMetadata::register(0, "Test".to_string());
        // Calculate the payload hash (provided by the client in a real scenario)
        let mut hash = Sha256::new();
        hash.update(b"Test message signed");
        // Verify the payload with the signature (calculated here, provided by the client in a real scenario)
        let signature: Signature = key.sign(hash.finalize().as_slice());
        assert!(app.validate_message(b"Test message signed", signature.to_bytes()));
    }
}