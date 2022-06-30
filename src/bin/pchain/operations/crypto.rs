use std::fs;
use std::path;
use std::process;
use ed25519_dalek::Signer;
use rand::rngs::OsRng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha20Rng;
use base64;
use serde::{Serialize, Deserialize};
use sha2::digest::Key;
use crate::setup;

const KEYPAIR_LENGTH: usize = 64;
const PRIVATEKEY_LENGTH: usize = 32;
const PUBLICKEY_LENGTH: usize = 32;

#[derive(Serialize, Deserialize)]
struct KeypairJSON {
    secret_key: String,
    public_key: String,
    keypair: String,
}

// generate keys using OS random number genrator. 
// Chacha20 do a further randomization before feeding as a seed to ed25519_dalek key generator.
// Thus the seed is cryptographically secure pseudorandom.
pub(crate) fn generate_keypair_and_save_as_json() {
    const KEYPAIR_FILENAME: &str = "keypair.json"; 

    let mut osrng = OsRng{};
    let mut chacha20_rng = ChaCha20Rng::from_rng(&mut osrng).unwrap(); 
   
    let keypair = ed25519_dalek::Keypair::generate(&mut chacha20_rng).to_bytes();
    let secret_key =  base64::encode(&keypair[0 .. PRIVATEKEY_LENGTH]);
    let public_key =  base64::encode(&keypair[PUBLICKEY_LENGTH .. KEYPAIR_LENGTH]);
    let keypair = base64::encode(keypair);

    let keypair_json = KeypairJSON {
        secret_key,
        public_key,
        keypair,
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
        let keypair_bs = base64::decode(json.keypair)
            .expect(E_MSG_JSON_INVALID_FORMAT);
        ed25519_dalek::Keypair::from_bytes(&keypair_bs)
            .expect(E_MSG_JSON_INVALID_FORMAT)
    };
    let ciphertext = keypair.sign(message.as_bytes()).to_bytes();
    let ciphertext = base64::encode(ciphertext);

    println!("Message: {}", message);
    println!("Ciphertext: {}", ciphertext);
}
