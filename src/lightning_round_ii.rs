
use derivative::Derivative;

#[derive(Derivative)]
#[derivative(Debug)]
struct Auth {
    pub api_key: String,
    #[derivative(Debug="ignore")]
    pub api_secret: String,
}


pub fn main() {
    let auth = Auth {
        api_key: "username".to_string(),
        api_secret: "password".to_string(),
    };

    dbg!(auth);
}