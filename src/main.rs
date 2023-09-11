// Clap ->  , Scrypt -> For Passsword Verification ,
// Make uuid whenever any new password added for

use bcrypt::{hash, verify, BcryptError, DEFAULT_COST};
use clap::{Args, Parser, Subcommand};
use passwords::PasswordGenerator;

#[derive(Parser)]
#[clap(
    name = "pass",
    version,
    author = "Ishan",
    about = "A easy-to-use CLI password manager and generator"
)]
struct Cli {
    /// Subcommand to do some operation like add, remove, etc.
    #[command(subcommand)]
    commands: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize the pass
    Init(InitArgs),

    /// Make a new password
    Add(AddArgs),

    /// Remove a password
    Remove(RemoveArgs),

    /// Update a password
    Update(UpdateArgs),

    /// List all made password
    List(ListArgs),

    /// Get a password
    Get(GetArgs),
    //
    // /// Login to the pass
    // Login(LoginArgs),
    //
    // /// Logout from the pass
    // Logout(LogoutArgs),
    //
    // /// Create a new user
    // CreateUser(CreateUserArgs),
    //
    // /// Remove a existing user
    // RemoveUser(RemoveUserArgs),
}

#[derive(Args)]
struct InitArgs {
    /// Master password for the pass
    #[clap(required = false, default_value = "")]
    master_password: String,
}

#[derive(Args)]
struct AddArgs {
    /// Username/email of the account
    #[clap(short = 'n', long = "name")]
    username: String,

    /// Password of the account (if not provided, a random password will be generated)
    #[clap(short = 'p', long = "pass")]
    password: Option<String>,

    /// URL of the site/app for which the password is
    #[clap(short = 'u', long = "url", default_value = "No URL provided")]
    url: String,

    /// Notes for the account
    #[clap(short = 'm', required = false, default_value = " ")]
    notes: String,
}

#[derive(Args)]
struct RemoveArgs {
    /// Username/email of the account
    // #[clap(short = 'n', long = "name")]
    username: String,
}

#[derive(Args)]
struct UpdateArgs {
    /// Username/email of the account
    // #[clap(short = 'n', long = "name")]
    username: String,
}

#[derive(Args)]
struct ListArgs {}

#[derive(Args)]
struct GetArgs {
    /// Username/email of the account
    // #[clap(short = 'n', long = "name")]
    username: String,
}
//
// #[derive(Args)]
// struct LoginArgs {}
//
// #[derive(Args)]
// struct LogoutArgs {}
//
// #[derive(Args)]
// struct CreateUserArgs {}
//
// #[derive(Args)]
// struct RemoveUserArgs {}

fn main() {
    // Making a safe & secure, easy to use password manager and generator using clap and bcrypt

    // Parsing the command line arguments into Cli struct
    let args = Cli::parse();

    match &args.commands.unwrap() {
        Commands::Init(_) => {
            // Taking the master password from the user
            println!("Enter Master Password: ");
            let mut master_password = String::new();
            std::io::stdin()
                .read_line(&mut master_password)
                .expect("Coudn't read Master Password");

            // Check if the password is strong enough
            if !is_strong_password(&mut master_password) {
                println!("Password is not strong enough!");
                return;
            }

            // Hashing the master Password
            let hashed_master_password = hash(&master_password, DEFAULT_COST).unwrap();

            // Storing the master password
            // let master_password_struct = InitArgs {
            //     master_password: hashed_master_password,
            // };
            println!("Initializing pass...");

            println!("\nMaster Password: {}", master_password);
            println!("Hashed Master password: {}", hashed_master_password);

            println!(
                "\nPassword Verify: {}",
                is_correct_master_password(&master_password, &hashed_master_password).unwrap()
            );
        }

        Commands::Add(args) => {
            println!("Username: {}", args.username);

            // Generating a password if not provided by user
            if args.password.is_none() {
                // args.password = Some(generate_password());
                // println!("Password: {:?}", args.password);
                println!("Password: {:?}", generate_password());
            } else {
                println!("Password: {:?}", args.password);
            }

            println!("URL: {}", args.url);
            println!("Notes: {}", args.notes);
        }

        Commands::Remove(args) => {
            println!("Username for remove password: {}", args.username);
        }

        Commands::Update(args) => {
            println!("Username for updating password: {}", args.username);
        }

        Commands::List(_) => {
            println!("Listing all passwords...");
        }

        Commands::Get(args) => {
            println!("Password for username {} is :", args.username);
        } //
          // Commands::Login(args) => {}
          //
          // Commands::Logout(args) => {}
          //
          // Commands::CreateUser(args) => {}
          //
          // Commands::RemoveUser(args) => {}
    }
    // TODO: Add clap for command line arguments
    // TODO: Add bcrypt for hashing passwords
    // TODO: Add file handling for storing passwords
}

fn generate_password() -> String {
    let generator = PasswordGenerator {
        length: 12,
        numbers: true,
        lowercase_letters: true,
        uppercase_letters: true,
        symbols: true,
        spaces: true,
        exclude_similar_characters: false,
        strict: true,
    };

    generator.generate_one().unwrap().to_string()
}

// Function to verify the master password is strong enough
fn is_strong_password(password: &mut String) -> bool {
    *password = password.trim().to_string();

    // Check if the password length is at least 8 characters
    if password.len() < 8 {
        return false;
    }

    let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password
        .chars()
        .any(|c| !c.is_alphanumeric() && !c.is_whitespace());

    return has_lowercase && has_uppercase && has_digit && has_special;
}

// Check if master password is correct
fn is_correct_master_password(
    master_password: &str,
    hashed_master_password: &str,
) -> Result<bool, BcryptError> {
    match verify(&master_password, &hashed_master_password) {
        Ok(is_correct) => Ok(true),
        Err(err) => Err(err),
    }
}
/* Imp Notes:
 * For custom name of project -> cargo install --path . && pass
 */
