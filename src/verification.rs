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

use std::convert::TryInto;
use std::fs::{File, OpenOptions};
use std::io::{Cursor, Error, ErrorKind, Read, Seek, SeekFrom};
use std::str::FromStr;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use ed25519_dalek::{Keypair, PublicKey, Signature, Verifier};
use rand_core::OsRng;
use ring::aead::{Aad, AES_256_GCM, LessSafeKey, Nonce, UnboundKey};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Visitor;
use sha2::{Digest, Sha256};
use tiny_http::Request;
use x25519_dalek::EphemeralSecret;
use yauuid::Uuid;

use crate::verification::VerificationResult::{Body, JsonReject};

#[derive(Serialize, Deserialize)]
pub struct AppMetadata {
    uuid: u128,
    name: String,
    #[serde(serialize_with = "ser_pubkey", deserialize_with = "deser_pubkey")]
    pub_key: PublicKey,
}

pub fn find_app(uuid: u128) -> Result<Option<AppMetadata>, Error> {
    let file = File::open("obs-controller-apps.ock");
    let mut file = match file {
        Ok(file) => file,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e)
    };
    loop {
        let entry_size = file.read_u64::<LittleEndian>();
        return match entry_size {
            Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => Ok(None),
            Err(e) => Err(Error::new(ErrorKind::Other, e)),
            Ok(entry_size) => {
                let entry_uuid = file.read_u128::<LittleEndian>()?;
                file.seek(SeekFrom::Current(-(std::mem::size_of::<u128>() as i64)))?;
                if uuid != entry_uuid {
                    file.seek(SeekFrom::Current(entry_size as i64))?;
                    continue;
                }
                Ok(Some(bincode::deserialize_from(&file).expect("Couldn't parse binary")))
            }
        };
    }
}

pub enum VerificationResult {
    /// Verification successful, return the parsed body
    Body(String),
    /// Verification failed, respond instantly with a status code + message
    JsonReject(u16, &'static str)
}

/// Parses an HTTP request, checks if the app is authenticated and returns the body if so
pub fn middleware_auth(req: &mut Request) -> Result<VerificationResult, Error> {
    let app = req.headers().iter().filter(|h| h.field.as_str() == "X-OBSC-App")
        .map(|h| Uuid::from_str(h.value.as_str())).next();
    let signature = req.headers().iter().filter(|h| h.field.as_str() == "X-OBSC-Signature")
        .map(|h| base64::decode(h.value.as_str())).next();
    match (&app, &signature) {
        (Some(Err(_)), _) => {
            return Ok(JsonReject(400, r#"{"message": "Invalid UUID in X-OBSC-App"}"#));
        }
        (_, Some(Err(_))) => {
            return Ok(JsonReject(400, r#"{"message": "Invalid Base64 in X-OBSC-Signature"}"#));
        },
        (Some(_), Some(_)) => {}
        _ => {
            return Ok(JsonReject(400, r#"{"message": "Missing X-OBSC-App or X-OBSC-Signature"}"#));
        }
    }
    let mut body = String::with_capacity(1024.min(req.body_length().unwrap_or(1024)));
    req.as_reader().take(1024).read_to_string(&mut body).expect("Couldn't read body.");
    let msg = if body.is_empty() { "obs-controller" } else { body.as_str() };
    let app = uuid_to_u128(app.unwrap().unwrap())?;
    let app = match find_app(app)? {
        Some(app) => app,
        None => {
            return Ok(JsonReject(400, r#"{"message": "Unknown app"}"#));
        }
    };
    let signature = signature.unwrap().unwrap();
    if signature.len() != 64 {
        return Ok(JsonReject(400, r#"{"message": "Signature must be 64 bytes in length."}"#));
    }
    if app.validate_message(msg.as_bytes(), signature.try_into().unwrap()) {
        Ok(Body(body))
    } else {
        Ok(JsonReject(401, r#"{"message": "Not authenticated"}"#))
    }
}

#[allow(unused)]
/// Registers an app, returning the encrypted private key for Ed25519 message signing and the server's X25519 public key.
pub fn register_encrypt(uuid: Uuid, name: String, their_pubkey: x25519_dalek::PublicKey) -> Result<(String, String), Error> {
    let our_secret = EphemeralSecret::new(rand_core::OsRng);
    let our_pubkey = x25519_dalek::PublicKey::from(&our_secret);
    // Compute the shared secret from the app's public key and our generated secret
    let shared = our_secret.diffie_hellman(&their_pubkey);
    // Encrypt the app's Ed25519 private key for communication
    let (_, key) = AppMetadata::register(uuid_to_u128(uuid)?, name);
    let mut secret = [0u8; 32 + 16]; // tag length = 16
    secret[..32].copy_from_slice(&key.secret.to_bytes());
    let aes = LessSafeKey::new(UnboundKey::new(&AES_256_GCM, shared.as_bytes()).expect("Couldn't create key"));
    let tag = aes.seal_in_place_separate_tag(Nonce::assume_unique_for_key([121; 12]), Aad::empty(), &mut secret[..32]).expect("Couldn't create tag");
    secret[32..].copy_from_slice(tag.as_ref());
    Ok((base64::encode(&secret), base64::encode(&our_pubkey.to_bytes())))
}

fn uuid_to_u128(uuid: Uuid) -> Result<u128, Error> {
    let mut bytes = Cursor::new(uuid.as_bytes());
    bytes.read_u128::<LittleEndian>()
}

impl AppMetadata {
    pub fn register(uuid: u128, name: String) -> (AppMetadata, Keypair) {
        let pair = Keypair::generate(&mut OsRng);
        let app = AppMetadata {
            uuid,
            name,
            pub_key: pair.public,
        };
        let mut file = OpenOptions::new().create(true).write(true).append(true).open("obs-controller-apps.ock").unwrap();
        let size = bincode::serialized_size(&app).expect("Couldn't get serialized size");
        file.write_u64::<LittleEndian>(size).expect("Couldn't write size");
        bincode::serialize_into(&mut file, &app).expect("Couldn't serialize into file");
        (app, pair)
    }

    fn validate_message(&self, message: &[u8], signature: [u8; 64]) -> bool {
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
        PublicKey::from_bytes(v).map_err(|e| serde::de::Error::custom(format!("Invalid public key: {:?}", e)))
    }
}

fn deser_pubkey<'de, D>(deser: D) -> Result<PublicKey, D::Error> where D: Deserializer<'de> {
    deser.deserialize_bytes(PubkeyVisitor)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use ed25519_dalek::{SecretKey, Signature, Signer};
    use rand_core::OsRng;
    use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
    use sha2::{Digest, Sha256};
    use x25519_dalek::{EphemeralSecret, PublicKey};
    use yauuid::Uuid;

    use crate::verification::{register_encrypt, uuid_to_u128};

    use super::AppMetadata;

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

    #[test]
    pub fn parse() {
        AppMetadata::register(12, "Test Parse".to_string());
        assert_eq!(12, super::find_app(12).unwrap().unwrap().uuid);
    }

    #[test]
    pub fn uuid() {
        let uuid = Uuid::from_str("98704291-09e9-40f2-8476-064521fadaff").unwrap();
        assert_eq!(340090132878606694826081478218872942744_u128, uuid_to_u128(uuid).unwrap());
    }

    #[test]
    pub fn get_private_key() {
        let secret = EphemeralSecret::new(OsRng);
        let pub_key = PublicKey::from(&secret);
        let (private, public) = register_encrypt(Uuid::from_str("98704291-09e9-40f2-8476-064521fadaff").unwrap(), String::from("Test"), pub_key).unwrap();
        let mut server_pub = [0u8; 32];
        server_pub.copy_from_slice(&base64::decode(public).unwrap());
        let server_pub = PublicKey::from(server_pub);
        let shared = secret.diffie_hellman(&server_pub);
        let aes = LessSafeKey::new(UnboundKey::new(&AES_256_GCM, shared.as_bytes()).unwrap());
        let mut priv_bytes = [0u8; 48];
        priv_bytes.copy_from_slice(&base64::decode(private).unwrap());
        aes.open_in_place(Nonce::assume_unique_for_key([121; 12]), Aad::empty(), &mut priv_bytes).unwrap();
        SecretKey::from_bytes(&priv_bytes[..32]).unwrap();
    }
}