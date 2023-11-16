use std::{io::Write, marker::PhantomData, num::NonZeroU32, string::FromUtf8Error};

use once_cell::sync::Lazy;
use ring::pbkdf2;
use serde::{Deserialize, Serialize};

use super::{
    store::{PasswordStore, PASS_ENTRY_STORE},
    util::password_input,
};

use crate::pass::util::{input_master_pass, password_hash, PASS_DIR_PATH, XDG_BASE};

pub static MASTER_PASS_STORE: Lazy<std::path::PathBuf> = Lazy::new(|| {
    XDG_BASE
        .place_state_file("master.dat")
        .expect("Unable to place master.dat file")
}); // $HOME/.local/state/.pass/Master.dat

#[derive(Debug, thiserror::Error)]
pub enum MasterPasswordError {
    #[error("The master password store file is not readable due to {0}")]
    UnableToRead(std::io::Error),

    #[error("Unable to create dirs for password storage")]
    UnableToCreateDirs(std::io::Error),

    #[error("Cannot read from console due to IO error")]
    UnableToReadFromConsole,

    #[error("Unable to write into master password store file: {0}")]
    UnableToWriteFile(std::io::Error),

    #[error("Master password not matched")]
    WrongMasterPassword,

    #[error("Unable to convert {0}")]
    UnableToConvert(#[source] FromUtf8Error),

    #[error("Bcrypt Error: {0}")]
    BcryptError(String),

    #[error("Unable to flush or use console IO: {0}")]
    IO(#[source] std::io::Error),

    #[error("Master password was not confirmed")]
    MasterPassConfirmFailed,

    #[error("Master password is not strong enough")]
    PassNotStrong,
}

/// Default state of [MasterPassword]
pub struct UnInit;

/// Initial state of [MasterPassword]
#[derive(Debug, Clone, Copy)]
pub struct Init;

/// Unverified state of [MasterPassword]
#[derive(Debug, Clone, Copy)]
pub struct UnVerified;

/// Verified state of [MasterPassword]
#[derive(Debug, Clone, Copy)]
pub struct Verified;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct MasterPassword<State = UnInit> {
    /// Master password
    pub master_pass: Option<Vec<u8>>,
    /// Master password hashed
    pub hash: Option<String>,
    /// [MasterPassword] state
    pub state: PhantomData<State>,
}

impl Default for MasterPassword<Init> {
    fn default() -> Self {
        Self {
            master_pass: Default::default(),
            hash: Default::default(),
            state: PhantomData,
        }
    }
}

impl MasterPassword<UnInit> {
    pub fn new() -> MasterPassword<Init> {
        MasterPassword::default()
    }
    pub fn create_pass_dirs() -> Result<(), MasterPasswordError> {
        std::fs::create_dir_all(PASS_DIR_PATH.to_owned())
            .map_err(MasterPasswordError::UnableToCreateDirs)
    }
}

impl MasterPassword<Init> {
    /// Convert initialised state to unverified state
    pub fn init(self) -> Result<MasterPassword<UnVerified>, MasterPasswordError> {
        // If master password file exists, then read hashed password and set to object
        if MASTER_PASS_STORE.exists() {
            return Ok(MasterPassword {
                hash: Some(self.get_hash_from_db()?),
                master_pass: None,
                state: PhantomData::<UnVerified>,
            });
        }

        self.generate_new_masterpass()
    }

    fn get_hash_from_db(&self) -> Result<String, MasterPasswordError> {
        String::from_utf8(
            std::fs::read(MASTER_PASS_STORE.to_path_buf())
                .map_err(MasterPasswordError::UnableToRead)?,
        )
        .map_err(MasterPasswordError::UnableToConvert)
    }

    fn generate_new_masterpass(&self) -> Result<MasterPassword<UnVerified>, MasterPasswordError> {
        MasterPassword::create_pass_dirs()?;

        let master_pass =
            input_master_pass().map_err(|_| MasterPasswordError::UnableToReadFromConsole)?;

        // Hashing prompted master password
        let hashed_password = password_hash(master_pass)
            .map_err(|_| MasterPasswordError::BcryptError("Unable to hash".to_string()))?;

        // Store hashed master password
        std::fs::write(MASTER_PASS_STORE.to_path_buf(), &hashed_password)
            .map_err(MasterPasswordError::UnableToWriteFile)?;

        colour::green_ln!("Pass initialised successfully");

        Ok(MasterPassword {
            master_pass: None,
            hash: Some(hashed_password),
            state: PhantomData::<UnVerified>,
        })
    }
}

impl MasterPassword<UnVerified> {
    /// Takes input master_password from user
    pub fn prompt(&mut self) -> Result<(), MasterPasswordError> {
        std::io::stdout().flush().map_err(MasterPasswordError::IO)?; // Flush the output to ensure prompt is displayed

        // Taking input master password
        let prompt_master_password = password_input("Enter your master password: ")
            .map_err(|_| MasterPasswordError::UnableToReadFromConsole)?;

        // Storing prompt password to object
        self.master_pass = Some(prompt_master_password);
        Ok(())
    }

    fn get_hash(&self) -> &String {
        assert!(self.hash.is_some());
        self.hash.as_ref().unwrap()
    }

    // Unlock the master password
    pub fn verify(&self) -> Result<MasterPassword<Verified>, MasterPasswordError> {
        std::io::stdout().flush().map_err(MasterPasswordError::IO)?; // Flush the output to ensure prompt is displayed

        // TODO: Improve code
        let password = self.master_pass.clone().expect("None is unrechable");

        // TODO: the last unwrap should be handled
        let hash = self.get_hash();

        match bcrypt::verify(password, hash) {
            Ok(true) => Ok(MasterPassword {
                master_pass: self.master_pass.clone(),
                hash: self.hash.clone(),
                state: PhantomData::<Verified>,
            }),
            Ok(false) => Err(MasterPasswordError::WrongMasterPassword),
            Err(e) => Err(MasterPasswordError::BcryptError(e.to_string())),
        }
    }
}

impl MasterPassword<Verified> {
    pub fn lock(self) -> MasterPassword<UnVerified> {
        MasterPassword {
            hash: self.hash,
            master_pass: None,
            state: PhantomData::<UnVerified>,
        }
    }

    // To change master password
    pub fn change(&mut self) -> Result<(), MasterPasswordError> {
        let prompt_new_master =
            input_master_pass().map_err(|_| MasterPasswordError::UnableToReadFromConsole)?;

        // Storing old master pass for later
        let old_master = self.clone();

        // Setting up new master in self
        let hash = password_hash(prompt_new_master.trim())
            .map_err(|_| MasterPasswordError::BcryptError("Unable to hash".to_string()))?;
        self.master_pass = Some(prompt_new_master.as_bytes().to_vec());
        self.hash = Some(hash.clone());

        // Re-encrypting contents over new master pass

        if PASS_ENTRY_STORE.exists() {
            self.clone()
                .re_encrypt_contents(old_master)
                .expect("Unable to re-encrypt entries");
        }

        // Store hash of changed master pass
        std::fs::write(MASTER_PASS_STORE.to_path_buf(), hash)
            .map_err(MasterPasswordError::UnableToWriteFile)?;
        colour::green_ln!("Master password changed successfully");

        Ok(())
    }

    ///
    pub fn re_encrypt_contents(&self, old_master: MasterPassword<Verified>) -> anyhow::Result<()> {
        // Load all entries form db by old master
        let mut storage = PasswordStore::load(PASS_ENTRY_STORE.to_path_buf(), old_master)?;

        // Changing master password
        storage.master_password = self.clone();

        // Again encrypt entries with new pass
        storage.dump(PASS_ENTRY_STORE.to_path_buf())?;

        Ok(())
    }

    /// Derive a encryption key from master password & salt
    pub fn derive_encryption_key(&self, salt: impl AsRef<[u8]>) -> [u8; 32] {
        let mut encryption_key = [0_u8; 32];

        // Deriving a encryption key using master pass
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            NonZeroU32::new(600_000).unwrap(),
            salt.as_ref(),
            self.master_pass.as_ref().unwrap(),
            &mut encryption_key,
        );

        encryption_key
    }
}

#[cfg(test)]
mod test {
    use super::MasterPassword;

    #[test]
    #[ignore = "unimplemented"]
    fn check_init() {
        let _master = MasterPassword::new();
        // let _unlocked = master.unwrap().verify();
    }
}
