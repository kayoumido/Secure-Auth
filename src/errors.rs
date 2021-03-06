/*!
 * Here lays the definition of all the custom errors used in the system
 *
 * # Note
 * To simplify the definition of messages n' stuff, the crates `strum` & `strum_macros`
 * were used (thank you SEC Midterm :D)
 *
 * # Author
 * Doran Kayoumi <doran.kayoumi@heig-vd.ch>
 */

use std::error;
use std::fmt;
use strum::EnumMessage;
use strum_macros;

#[derive(PartialEq, Debug, strum_macros::EnumMessage)]
pub enum AuthError {
    #[strum(message = "Your login details are incorrect.")]
    LoginError,

    #[strum(message = "Something went wrong during registration.")]
    RegistrationError,

    #[strum(message = "Something went wrong during password reset.")]
    ResetError,

    #[strum(message = "The e-mail address you entered is invalid.")]
    InvalidEmail,

    #[strum(message = "Your password must be between 8 and 64 characters long.")]
    InvalidPassword,

    #[strum(message = "This e-mail address is already used for another account.")]
    EmailUsed,

    #[strum(message = "Reset token is expired.")]
    ExpiredToken,

    #[strum(message = "You've entered an ivalid token.")]
    TokenMismatch,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_message().unwrap())
    }
}

impl error::Error for AuthError {
    fn description(&self) -> &str {
        self.get_message().unwrap()
    }
}

#[derive(PartialEq, Debug, strum_macros::EnumMessage)]
pub enum UserDBError {
    #[strum(message = "Unable to create the user.")]
    CreateUserError,

    #[strum(message = "Unable to update the user.")]
    UpdateUserError,

    #[strum(message = "Unable to get the user.")]
    GetUserError,
}

impl fmt::Display for UserDBError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_message().unwrap())
    }
}

impl error::Error for UserDBError {
    fn description(&self) -> &str {
        self.get_message().unwrap()
    }
}
