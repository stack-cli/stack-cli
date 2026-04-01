use age::secrecy::SecretString;
use age::{Decryptor, Encryptor};
use anyhow::{anyhow, Context, Result};
use std::io::{Read, Write};
use std::str::FromStr;

pub const AGE_ENCRYPTED_PREFIX: &str = "ENC[AGE,";
pub const AGE_ENCRYPTED_SUFFIX: &str = "]";
pub const ENCRYPTED_KEY_PREFIX: &str = "AGE-SECRET-KEY-ENCRYPTED:";

pub fn is_encrypted(value: &str) -> bool {
    value.starts_with(AGE_ENCRYPTED_PREFIX) && value.ends_with(AGE_ENCRYPTED_SUFFIX)
}

pub fn wrap_encrypted(payload_b64: &str) -> String {
    format!("{AGE_ENCRYPTED_PREFIX}{payload_b64}{AGE_ENCRYPTED_SUFFIX}")
}

pub fn unwrap_encrypted(value: &str) -> Result<&str> {
    if !is_encrypted(value) {
        return Err(anyhow!("Value is not in ENC[AGE,...] format"));
    }
    Ok(&value[AGE_ENCRYPTED_PREFIX.len()..value.len() - AGE_ENCRYPTED_SUFFIX.len()])
}

pub fn encrypt_with_public_key(plaintext: &str, public_key: &str) -> Result<String> {
    let recipient = age::x25519::Recipient::from_str(public_key)
        .context("Invalid age public key in STACK_CLI_AGE_PUBLIC_KEY")?;

    let encryptor = Encryptor::with_recipients(vec![Box::new(recipient)])
        .context("Failed to initialize age encryptor")?;

    let mut encrypted = Vec::new();
    let mut writer = encryptor
        .wrap_output(&mut encrypted)
        .context("Failed to start age encryption stream")?;
    writer
        .write_all(plaintext.as_bytes())
        .context("Failed to write plaintext into age encryptor")?;
    writer
        .finish()
        .context("Failed to finalize age encryption")?;

    Ok(wrap_encrypted(&base64::encode(encrypted)))
}

pub fn decrypt_with_secret_key(ciphertext: &str, secret_key: &str) -> Result<String> {
    let payload = unwrap_encrypted(ciphertext)?;
    let encrypted = base64::decode(payload).context("Encrypted value is not valid base64")?;

    let identity = parse_identity(secret_key)?;

    let decryptor =
        Decryptor::new(&encrypted[..]).context("Encrypted value is not valid age data")?;

    match decryptor {
        Decryptor::Recipients(d) => {
            let mut reader = d
                .decrypt(std::iter::once(&identity as &dyn age::Identity))
                .context("Failed to decrypt value with provided age identity")?;
            let mut decrypted = String::new();
            reader
                .read_to_string(&mut decrypted)
                .context("Decrypted value is not valid UTF-8")?;
            Ok(decrypted)
        }
        Decryptor::Passphrase(_) => Err(anyhow!(
            "This value was encrypted with a passphrase, expected age recipient encryption"
        )),
    }
}

pub fn generate_key_pair(password: Option<&str>) -> Result<(String, String)> {
    let identity = age::x25519::Identity::generate();
    let public_key = identity.to_public().to_string();
    let private_key = identity.to_string();

    if let Some(password) = password {
        let encryptor = Encryptor::with_user_passphrase(SecretString::from(password.to_string()));
        let mut encrypted = Vec::new();
        let mut writer = encryptor
            .wrap_output(&mut encrypted)
            .context("Failed to start passphrase encryption for private key")?;
        writer
            .write_all(private_key.as_bytes())
            .context("Failed to encrypt private key")?;
        writer
            .finish()
            .context("Failed to finalize passphrase encryption for private key")?;
        Ok((
            public_key,
            format!("{ENCRYPTED_KEY_PREFIX}{}", base64::encode(encrypted)),
        ))
    } else {
        Ok((public_key, private_key))
    }
}

fn parse_identity(secret_key: &str) -> Result<age::x25519::Identity> {
    if let Some(encoded) = secret_key.strip_prefix(ENCRYPTED_KEY_PREFIX) {
        let payload =
            base64::decode(encoded).context("Encrypted key payload is not valid base64")?;
        let decryptor =
            Decryptor::new(&payload[..]).context("Encrypted key is not valid age data")?;

        match decryptor {
            Decryptor::Passphrase(d) => {
                let password = std::env::var("STACK_CLI_AGE_KEY_PASSWORD").context(
                    "STACK_CLI_AGE_KEY_PASSWORD is required for password-protected keys",
                )?;
                let mut reader = d
                    .decrypt(&SecretString::from(password), None)
                    .context("Failed to decrypt password-protected private key")?;
                let mut identity_text = String::new();
                reader
                    .read_to_string(&mut identity_text)
                    .context("Password-protected private key is not valid UTF-8")?;
                age::x25519::Identity::from_str(identity_text.trim())
                    .context("Decrypted private key is not a valid age x25519 identity")
            }
            Decryptor::Recipients(_) => Err(anyhow!(
                "Expected passphrase-protected private key, but got recipient-encrypted data"
            )),
        }
    } else {
        age::x25519::Identity::from_str(secret_key.trim())
            .context("Invalid age private key in STACK_CLI_AGE_SECRET_KEY")
    }
}
