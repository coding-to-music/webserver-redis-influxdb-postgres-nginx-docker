use lib::auth::TokenHandler;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let jwt_secret = &args[1];
    let token = &args[2];

    let token_handler = TokenHandler::new(jwt_secret.clone());

    let parsed = token_handler.parse_token(&token).unwrap();

    println!("{}", serde_json::to_string_pretty(&parsed).unwrap());
}
