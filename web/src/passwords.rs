use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2
};


pub fn hash_new(password: String) -> Result<(String, String), String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(n) => n,
        Err(n) => {return Err(format!("Error hashing password | {}", n))}
    };
    
    let hash: String = match password_hash.hash {
        Some(n) => n.to_string(),
        None => {return Err("Error getting hash from password".to_string());},
    };

    Ok((hash, salt.to_string()))
}

pub fn hash_old(password: String, salt: String) -> Result<String, String> {
    let salt = match SaltString::new(salt.as_str()) {
        Ok(n) => n,
        Err(n) => {
            return Err(format!("Error recognizing salt | {}", n));
        }
    };

    let argon2 = Argon2::default();

    let password_hash = match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(n) => n,
        Err(n) => {return Err(format!("Error hashing password | {}", n))}
    };
    
    let hash: String = match password_hash.hash {
        Some(n) => n.to_string(),
        None => {return Err("Error getting hash from password".to_string());},
    };

    Ok(hash)
}



#[test]
fn test_match() {
    let password: String = String::from("test password");

    let (hash, salt) = hash_new(password.clone()).expect("error creating hash");

    let hash2 = hash_old(password, salt).expect("error creating hash");

    println!("{}\n{}", hash, hash2);

    assert_eq!(hash, hash2);

}