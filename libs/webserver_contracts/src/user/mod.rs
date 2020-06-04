pub use add_user::{AddUserParams, AddUserParamsInvalid, AddUserResult};
pub use change_password::{
    ChangePasswordParams, ChangePasswordParamsInvalid, ChangePasswordResult,
};
pub use delete_user::{DeleteUserParams, DeleteUserParamsInvalid, DeleteUserResult};
pub use set_role::{SetRoleParams, SetRoleParamsInvalid, SetRoleResult};
pub use validate_user::{ValidateUserParams, ValidateUserParamsInvalid, ValidateUserResult};

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct User {
    username: String,
    password: String,
}

impl User {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}

mod add_user {
    use super::User;
    use crate::Params;
    use std::convert::TryFrom;

    #[derive(serde::Serialize, Clone, Debug)]
    pub struct AddUserParams {
        user: User,
    }

    impl Params for AddUserParams {}

    impl AddUserParams {
        pub fn new(user: User) -> Self {
            Self { user }
        }

        pub fn user(&self) -> &User {
            &self.user
        }
    }

    #[derive(serde::Deserialize)]
    struct AddUserParamsBuilder {
        user: User,
    }

    impl AddUserParamsBuilder {
        fn build(self) -> Result<AddUserParams, AddUserParamsInvalid> {
            if self.user.password.len() < 10 {
                Err(AddUserParamsInvalid::PasswordTooShort)
            } else {
                Ok(AddUserParams { user: self.user })
            }
        }
    }

    pub enum AddUserParamsInvalid {
        InvalidFormat(serde_json::Error),
        PasswordTooShort,
    }

    impl TryFrom<crate::JsonRpcRequest> for AddUserParams {
        type Error = AddUserParamsInvalid;
        fn try_from(request: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
            let builder: AddUserParamsBuilder = serde_json::from_value(request.params)
                .map_err(AddUserParamsInvalid::InvalidFormat)?;

            builder.build()
        }
    }

    #[derive(serde::Serialize)]
    pub struct AddUserResult {
        success: bool,
    }

    impl AddUserResult {
        pub fn new(success: bool) -> Self {
            Self { success }
        }
    }
}

mod change_password {
    use super::User;
    use crate::Params;
    use std::convert::TryFrom;

    #[derive(serde::Serialize, Clone, Debug)]
    pub struct ChangePasswordParams {
        user: User,
        new_password: String,
    }

    impl Params for ChangePasswordParams {}

    impl ChangePasswordParams {
        pub fn user(&self) -> &User {
            &self.user
        }

        pub fn new_password(&self) -> &str {
            &self.new_password
        }
    }

    #[derive(serde::Deserialize)]
    struct ChangePasswordParamsBuilder {
        user: User,
        new_password: String,
    }

    impl ChangePasswordParamsBuilder {
        fn build(self) -> Result<ChangePasswordParams, ChangePasswordParamsInvalid> {
            Ok(ChangePasswordParams {
                user: self.user,
                new_password: self.new_password,
            })
        }
    }

    impl TryFrom<crate::JsonRpcRequest> for ChangePasswordParams {
        type Error = ChangePasswordParamsInvalid;
        fn try_from(request: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
            let builder: ChangePasswordParamsBuilder = serde_json::from_value(request.params)
                .map_err(ChangePasswordParamsInvalid::InvalidFormat)?;

            builder.build()
        }
    }

    pub enum ChangePasswordParamsInvalid {
        InvalidFormat(serde_json::Error),
    }

    #[derive(serde::Serialize)]
    pub struct ChangePasswordResult {
        success: bool,
    }

    impl ChangePasswordResult {
        pub fn new(success: bool) -> Self {
            Self { success }
        }
    }
}

mod validate_user {
    use super::User;
    use crate::Params;
    use std::convert::TryFrom;

    #[derive(serde::Serialize, Clone, Debug)]
    pub struct ValidateUserParams {
        user: User,
    }

    impl Params for ValidateUserParams {}

    impl ValidateUserParams {
        pub fn user(&self) -> &User {
            &self.user
        }
    }

    #[derive(serde::Deserialize)]
    struct ValidateUserParamsBuilder {
        user: User,
    }

    impl ValidateUserParamsBuilder {
        fn build(self) -> Result<ValidateUserParams, ValidateUserParamsInvalid> {
            Ok(ValidateUserParams { user: self.user })
        }
    }

    impl TryFrom<crate::JsonRpcRequest> for ValidateUserParams {
        type Error = ValidateUserParamsInvalid;
        fn try_from(request: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
            let builder: ValidateUserParamsBuilder = serde_json::from_value(request.params)
                .map_err(ValidateUserParamsInvalid::InvalidFormat)?;

            builder.build()
        }
    }

    pub enum ValidateUserParamsInvalid {
        InvalidFormat(serde_json::Error),
    }

    #[derive(serde::Serialize)]
    pub struct ValidateUserResult {
        valid: bool,
    }

    impl ValidateUserResult {
        pub fn new(valid: bool) -> Self {
            Self { valid }
        }
    }
}

mod set_role {
    use super::User;
    use crate::Params;
    use std::convert::TryFrom;

    #[derive(serde::Serialize, Clone, Debug)]
    pub struct SetRoleParams {
        user: User,
        username: String,
        role: String,
    }

    impl Params for SetRoleParams {}

    impl SetRoleParams {
        pub fn new(user: User, username: String, role: String) -> Self {
            Self {
                user,
                username,
                role,
            }
        }

        pub fn user(&self) -> &User {
            &self.user
        }

        pub fn username(&self) -> &str {
            &self.username
        }

        pub fn role(&self) -> &str {
            &self.role
        }
    }

    #[derive(serde::Deserialize)]
    struct SetRoleParamsBuilder {
        user: User,
        username: String,
        role: String,
    }

    impl SetRoleParamsBuilder {
        fn build(self) -> Result<SetRoleParams, SetRoleParamsInvalid> {
            Ok(SetRoleParams {
                user: self.user,
                username: self.username,
                role: self.role,
            })
        }
    }

    impl TryFrom<crate::JsonRpcRequest> for SetRoleParams {
        type Error = SetRoleParamsInvalid;
        fn try_from(request: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
            let builder: SetRoleParamsBuilder = serde_json::from_value(request.params)
                .map_err(SetRoleParamsInvalid::InvalidFormat)?;

            builder.build()
        }
    }

    pub enum SetRoleParamsInvalid {
        InvalidFormat(serde_json::Error),
        InvalidRole,
    }

    #[derive(serde::Serialize)]
    pub struct SetRoleResult {
        success: bool,
    }

    impl SetRoleResult {
        pub fn new(success: bool) -> Self {
            Self { success }
        }
    }
}

mod delete_user {
    use super::User;
    use std::convert::TryFrom;

    #[derive(serde::Serialize, Clone, Debug)]
    pub struct DeleteUserParams {
        user: User,
        username: String,
    }

    impl DeleteUserParams {
        pub fn new(user: User, username: String) -> Self {
            Self { user, username }
        }

        pub fn user(&self) -> &User {
            &self.user
        }

        pub fn username(&self) -> &str {
            &self.username
        }
    }

    #[derive(serde::Deserialize)]
    struct DeleteUserParamsBuilder {
        user: User,
        username: String,
    }

    impl DeleteUserParamsBuilder {
        fn build(self) -> Result<DeleteUserParams, DeleteUserParamsInvalid> {
            Ok(DeleteUserParams {
                user: self.user,
                username: self.username,
            })
        }
    }

    pub enum DeleteUserParamsInvalid {
        InvalidFormat(serde_json::Error),
    }

    impl TryFrom<crate::JsonRpcRequest> for DeleteUserParams {
        type Error = DeleteUserParamsInvalid;
        fn try_from(value: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
            let builder: DeleteUserParamsBuilder = serde_json::from_value(value.params)
                .map_err(DeleteUserParamsInvalid::InvalidFormat)?;

            builder.build()
        }
    }

    #[derive(serde::Serialize)]
    pub struct DeleteUserResult {
        success: bool,
    }

    impl DeleteUserResult {
        pub fn new(success: bool) -> Self {
            Self { success }
        }
    }
}
