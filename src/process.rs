use crate::auth::{login, register, reset, twofa};
use crate::db::models::User;
use crate::db::repository::{SQliteUserRepository, UserRepository};
use crate::errors::AuthError;
use crate::user_input;
use crate::utils;

pub fn login_process() -> User {
    println!("Login:");
    loop {
        let email = user_input::ask_for_email();
        let passwd = user_input::ask_for_password();

        let u = login::login(&email, &passwd);
        if let Err(e) = u {
            println!("{}", e);
            continue;
        }

        let u = u.unwrap();
        if u.is_2fa_enabled() {
            let secret = u.get_secret_2fa().unwrap();
            confirm_2fa_code(&secret);
        }

        return u;
    }
}

pub fn registration_process() {
    println!("Registration:");
    loop {
        let email = user_input::ask_for_email();
        let passwd = user_input::ask_for_password_with_policy_check();

        let u = register::register(&email, &passwd);
        if let Err(e) = u {
            println!("{}", e);
            continue;
        }

        break;
    }
}

pub fn reset_password_process() {
    let repository = SQliteUserRepository {};
    _reset_password_process(&repository)
}
pub fn enable_2fa_process(u: &mut User) {
    let repository = SQliteUserRepository {};
    _enable_2fa_process(u, &repository)
}
pub fn disable_2fa_process(u: &mut User) {
    let repository = SQliteUserRepository {};
    _disable_2fa_process(u, &repository)
}

fn _reset_password_process(repository: &dyn UserRepository) {
    println!("Password reset:");
    let email = user_input::ask_for_email();

    println!("In case a user with that data exists in our database, you'll recieve the token to reset your password");

    // try and generate a reset token for the given email
    if let Err(_) = reset::generate_reset_token(&email) {
        // exit the process without informing the user to avoid any forms of attacks
        return;
    }

    reset::send_reset_token(&email);

    // ideally all of the following would be handeled somewhere else
    // and the `send_reset_token` would send an email with a url that hte user needs to click to follow th reset instructions

    loop {
        let input_token = user_input::ask_for_reset_token();

        if let Err(e) = reset::check_token(&email, &input_token) {
            println!("{}", e);

            match e {
                AuthError::ExpiredToken => return,
                AuthError::TokenMismatch => continue,
                AuthError::ResetError => return,
                _ => panic!("Unexpected return value."),
            }
        }

        break;
    }

    // get the user from the db
    let u = repository.get_user(&email);
    if let Err(e) = u {
        // something bad happened (e.g. the db is down)
        // Note: The problem can't come from the non existance of the user
        //       because `generate_reset_token` generates a token only if the user exists.
        //       hence the panic.
        panic!(e);
    }
    let u = u.unwrap();

    if u.is_2fa_enabled() {
        println!("Confirm your identity:");
        // we can safely get the users 2FA secret
        let secret = u.get_secret_2fa().unwrap();
        confirm_2fa_code(&secret);
    }

    let passwd = user_input::ask_for_password_with_policy_check();
    if let Err(e) = reset::change_password(&email, &passwd) {
        println!("{}", e);
    }
}

fn _enable_2fa_process(u: &mut User, repository: &dyn UserRepository) {
    // quick check that the user doesn't already have 2fa activated
    // you never know...
    if u.is_2fa_enabled() {
        println!("Two-factor authentication already enabled");
        return;
    }

    // Before adding the 2FA, confirm the users identity
    // by asking for hes/his password
    println!("Confirm your identity:");
    confirm_identity_with_passwd(&u.get_password());

    // generate the 2FA secret & the QR code so the user can add the secret
    // to her/his 2FA authentication app
    let secret = twofa::generate_secret();
    let qr_url = twofa::generate_qr(&secret, &u.get_email(), "Lab 02 - Authentication");
    println!(
        "Scan the following QR code with your favorite Authentication app: {}\n",
        qr_url
    );

    // Ask the user to input a authentication code
    // to confirm she/he correctly setup the 2FA
    println!("Confirm 2FA setup:");
    confirm_2fa_code(&secret);

    // update the database with the new secret
    u.set_secret_2fa(Some(secret));
    if let Err(_) = repository.update_user(&u) {
        println!("Two-factor authentication failed.");

        // just to be safe, revert changes
        u.set_secret_2fa(None);
    }
}

fn _disable_2fa_process(u: &mut User, repository: &dyn UserRepository) {
    // quick check that the user doesn't already have 2fa activated
    // you never know...
    if !u.is_2fa_enabled() {
        println!("Two-factor authentication is already disabled");
        return;
    }

    // Before touching the 2FA, confirm the users identity
    // by asking for hers/his password
    println!("Confirm your identity:");
    confirm_identity_with_passwd(&u.get_password());

    // Ask the user to input a authentication code
    // to confirm she/he correctly setup the 2FA
    let secret = u.get_secret_2fa().unwrap(); // we can safely get the users 2FA secret
    confirm_2fa_code(&secret);

    // NOTE: For some reason this doesn't remove the secret from the DB
    // TODO: Fix
    // update the database with the changes
    u.set_secret_2fa(None);
    if let Err(_) = repository.update_user(&u) {
        println!("Two-factor authentication failed.");

        // just to be safe, revert changes
        u.set_secret_2fa(Some(secret));
    }
}

fn confirm_2fa_code(secret: &str) {
    loop {
        let auth_code = user_input::ask_for_authentication_code();
        if !twofa::check_code(secret, &auth_code) {
            println!("Incorrect authentication code.");
            continue;
        }
        break;
    }
}

fn confirm_identity_with_passwd(user_passwd: &str) {
    loop {
        let passwd = user_input::ask_for_password();
        if !utils::verify_hash(&passwd, user_passwd) {
            println!("Incorrect password.");
            continue;
        }
        return;
    }
}
