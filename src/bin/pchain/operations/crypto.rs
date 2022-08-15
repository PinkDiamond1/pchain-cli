/*
 Copyright (c) 2022 ParallelChain Lab

 This program is free software: you can redistribute it and/or modify
 it under the terms of the GNU General Public License as published by
 the Free Software Foundation, either version 3 of the License, or
 (at your option) any later version.

 This program is distributed in the hope that it will be useful,
 but WITHOUT ANY WARRANTY; without even the implied warranty of
 MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 GNU General Public License for more details.

 You should have received a copy of the GNU General Public License
 along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
use std::fs;
use std::path;
use std::process;
use ed25519_dalek::Signer;
use rand::rngs::OsRng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha20Rng;
use serde::{Serialize, Deserialize};
use crate::setup;

const KEYPAIR_LENGTH: usize = 64;
const PRIVATEKEY_LENGTH: usize = 32;
const PUBLICKEY_LENGTH: usize = 32;

#[derive(Serialize, Deserialize)]
pub struct KeypairJSON {
    pub secret_key: String,
    pub public_key: String,
    pub keypair: String,
}

// generate keys using OS random number genrator. 
// Chacha20 do a further randomization before feeding as a seed to ed25519_dalek key generator.
// Thus the seed is cryptographically secure pseudorandom.
pub(crate) fn generate_keypair_and_save_as_json() {
    const KEYPAIR_FILENAME: &str = "keypair.json"; 

    let mut osrng = OsRng{};
    let mut chacha20_rng = ChaCha20Rng::from_rng(&mut osrng).unwrap(); 
   
    let keypair = ed25519_dalek::Keypair::generate(&mut chacha20_rng).to_bytes();
    let secret_key =  protocol_types::Base64URL::encode(&keypair[0 .. PRIVATEKEY_LENGTH]);
    let public_key =  protocol_types::Base64URL::encode(&keypair[PUBLICKEY_LENGTH .. KEYPAIR_LENGTH]);
    let keypair = protocol_types::Base64URL::encode(keypair);

    let keypair_json = KeypairJSON {
        secret_key: secret_key.to_string(),
        public_key: public_key.to_string(),
        keypair: keypair.to_string(),
    };

    if path::Path::new(KEYPAIR_FILENAME).exists() {
        println!("./keypair.json already exists. Rename this file (or delete it, but only if you are 100% sure you don't need it anymore).");
        process::exit(1);
    };

    fs::write(KEYPAIR_FILENAME, serde_json::to_string(&keypair_json).unwrap())
        .expect("Could not write keypair.json file. Your filesystem might be a bit wonky.");

    println!("keypair.json saved in current directory.")
}

pub(crate) fn sign(message: &str) { 
    const E_MSG_NOT_FOUND: &str = "keypair.json not found. Ensure that VeryLight configuration is complete using the Setup command";
    const E_MSG_JSON_INVALID_FORMAT: &str = "Registered keypair.json is of invalid format. Consult VeryLight's repository README for the correct format."; 

    let keypair = {
        let path = setup::read_config(setup::ConfigField::KeypairJSONPath);
        let file = fs::File::open(path)
            .expect(E_MSG_NOT_FOUND);
        let json = serde_json::from_reader::<_, KeypairJSON>(file)
            .expect(E_MSG_JSON_INVALID_FORMAT);
        let keypair_bs = protocol_types::Base64URL::decode(&json.keypair)
            .expect(E_MSG_JSON_INVALID_FORMAT);
        ed25519_dalek::Keypair::from_bytes(&keypair_bs)
            .expect(E_MSG_JSON_INVALID_FORMAT)
    };
    let serialized_credentials = protocol_types::Base64URL::decode(&message).unwrap();
    let ciphertext : ed25519_dalek::Signature = keypair.sign(&serialized_credentials[..]);
    let ciphertext = protocol_types::Base64URL::encode(ciphertext);

    println!("Message: {}", message);
    println!("Ciphertext: {}", ciphertext.to_string());
}
