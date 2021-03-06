/*!
 * Functions related to the password reset
 *
 * # Author
 * Doran Kayoumi <doran.kayoumi@heig-vd.ch>
 */

use chrono::prelude::*;

use crate::db::repository::{SQliteUserRepository, UserRepository};
use crate::errors::AuthError;
use crate::utils;

const CODE_VALIDITY_MIN: i64 = 15;

/// Public function for the reset token generation
/// See `_generate_reset_token` for more info
///
pub fn generate_reset_token(email: &str) -> Result<(), AuthError> {
    let repository = SQliteUserRepository {};
    _generate_reset_token(email, &repository)
}

/// Public function for changing the password
/// See `_change_password` for more info
///
pub fn change_password(email: &str, new_passwd: &str) -> Result<(), AuthError> {
    let repository = SQliteUserRepository {};
    _change_password(email, new_passwd, &repository)
}

/// Public function for the reset token check
/// See `_check_token` for more info
///
pub fn check_token(email: &str, token: &str) -> Result<(), AuthError> {
    let repository = SQliteUserRepository {};
    _check_token(email, token, &repository)
}

/// Public function for the sending of the reset token
/// See `_send_reset_token` for more info
///
pub fn send_reset_token(email: &str) {
    let repository = SQliteUserRepository {};
    _send_reset_token(email, &repository)
}

/// Generate a new reset token
///
/// # Arguments
///
/// * `email` - the email of the user that needs a reset token
///
/// * `repository` - the user repository to interact with
///
fn _generate_reset_token(email: &str, repository: &dyn UserRepository) -> Result<(), AuthError> {
    // generate the reset token
    // note: A token is generated even though the user doesn't exists
    //       this is done to not leak the info that the user doesn't exist.
    let token = utils::gen_token();

    // try and find the user in the db
    let u = repository.get_user(email);
    if let Err(_) = u {
        return Err(AuthError::ResetError);
    }

    // update the user with the reset token
    let mut u = u.unwrap();
    u.set_reset_token(&token);
    if let Err(_) = repository.update_user(&u) {
        return Err(AuthError::ResetError);
    }

    Ok(())
}

/// Change the users password
///
/// # Arguments
///
/// * `email` - the email of the user that needs a password change
///
/// * `new_passwd` - the new password
///
/// * `repository` - the user repository to interact with
///
fn _change_password(
    email: &str,
    new_passwd: &str,
    repository: &dyn UserRepository,
) -> Result<(), AuthError> {
    let u = repository.get_user(email);
    if let Err(_) = u {
        return Err(AuthError::ResetError);
    }
    let mut u = u.unwrap();

    // update the users password
    u.set_password(&utils::hash(new_passwd));

    if let Err(_) = repository.update_user(&u) {
        return Err(AuthError::ResetError);
    }

    Ok(())
}

/// Check if an inputed reset token is valid
///
/// # Arguments
///
/// * `email` - the email of the user that needs a password change
///
/// * `token` - the token to validate
///
/// * `repository` - the user repository to interact with
///
fn _check_token(
    email: &str,
    token: &str,
    repository: &dyn UserRepository,
) -> Result<(), AuthError> {
    let u = repository.get_user(email);
    if let Err(_) = u {
        return Err(AuthError::ResetError);
    }
    let u = u.unwrap();

    // check if the user has a reset token set
    // this should never happen but you never know
    if u.get_reset_token() == None {
        return Err(AuthError::ResetError);
    }

    let token_created_at =
        DateTime::parse_from_rfc3339(u.get_reset_token_created_at().unwrap().as_str()).unwrap();
    let now = DateTime::parse_from_rfc3339(Utc::now().to_rfc3339().as_str()).unwrap();

    if (now - token_created_at).num_minutes() > CODE_VALIDITY_MIN {
        Err(AuthError::ExpiredToken)
    } else if u.get_reset_token().unwrap() != token {
        Err(AuthError::TokenMismatch)
    } else {
        Ok(())
    }
}

/// Send the reset token to the user
///
/// # Arguments
///
/// * `email` - the email of the user that needs a password change
///
/// * `repository` - the user repository to interact with
///
fn _send_reset_token(email: &str, repository: &dyn UserRepository) {
    let u = repository.get_user(email).unwrap();

    println!();
    println!("from: lab02.auth@heig-vd.lo");
    println!("to: {}", email);
    println!("subject: Lab 02 - Auth Reset token");
    println!("message:");
    println!("Here is your reset token: {}", u.get_reset_token().unwrap());
    println!("Kind regards");
    println!();
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::db::models::User;
    use crate::db::repository::MockSQliteUserRepository;
    use crate::errors::UserDBError;

    #[test]
    fn test_token_generation_with_unknown_user() {
        let mut mock = MockSQliteUserRepository::new();

        mock.expect_get_user()
            .returning(|_| Err(UserDBError::GetUserError));

        let res = _generate_reset_token("email@email.test", &mock);

        assert_eq!(Err(AuthError::ResetError), res);
    }

    #[test]
    fn test_token_generation_with_known_user() {
        let mut mock = MockSQliteUserRepository::new();

        mock.expect_get_user()
            .returning(|e| Ok(User::new(e, "passwd_hash")));
        mock.expect_update_user().returning(|_| Ok(()));

        let res = _generate_reset_token("email@email.test", &mock);

        assert_eq!(Ok(()), res);
    }

    #[test]
    fn test_password_change_with_unknown_user() {
        let mut mock = MockSQliteUserRepository::new();

        mock.expect_get_user()
            .returning(|_| Err(UserDBError::GetUserError));

        let res = _change_password("email@email.test", "password", &mock);

        assert_eq!(Err(AuthError::ResetError), res);
    }

    #[test]
    fn test_password_change_with_known_user() {
        let mut mock = MockSQliteUserRepository::new();

        mock.expect_get_user()
            .returning(|e| Ok(User::new(e, "passwd_hash")));
        mock.expect_update_user().returning(|_| Ok(()));

        let res = _change_password("email@email.test", "password", &mock);

        assert_eq!(Ok(()), res);
    }

    #[test]
    fn test_check_token_with_unknown_user() {
        let mut mock = MockSQliteUserRepository::new();

        mock.expect_get_user()
            .returning(|_| Err(UserDBError::GetUserError));

        let res = _check_token("email@email.test", "token", &mock);

        assert_eq!(Err(AuthError::ResetError), res);
    }

    #[test]
    fn test_check_token_with_known_user_and_no_reset_token() {
        let mut mock = MockSQliteUserRepository::new();

        mock.expect_get_user()
            .returning(|e| Ok(User::new(e, "passwd_hash")));

        let res = _check_token("email@email.test", "token", &mock);

        assert_eq!(Err(AuthError::ResetError), res);
    }

    #[test]
    fn test_check_token_with_known_user_and_reset_token() {
        let mut mock = MockSQliteUserRepository::new();

        mock.expect_get_user().returning(|e| {
            let mut u = User::new(e, "passwd_hash");
            u.set_reset_token("token");
            Ok(u)
        });

        let res = _check_token("email@email.test", "token", &mock);

        assert_eq!(Ok(()), res);
    }

    #[test]
    fn test_check_token_with_known_user_and_wrong_reset_token() {
        let mut mock = MockSQliteUserRepository::new();

        mock.expect_get_user().returning(|e| {
            let mut u = User::new(e, "passwd_hash");
            u.set_reset_token("token");
            Ok(u)
        });

        let res = _check_token("email@email.test", "wrongtoken", &mock);

        assert_eq!(Err(AuthError::TokenMismatch), res);
    }
}
