use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use std::fs::OpenOptions;
use std::io::{prelude::*, SeekFrom};

use serde_json;
use serde::{Serialize, Deserialize};

use crate::passwords::{hash_new, hash_old};

#[derive(Clone, Debug, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub password: String,
}

#[derive(Eq, Clone, Debug, Deserialize, Serialize)]
pub struct UserSettings {
    username: String,
    pub public: Vec<String>,
}

impl PartialEq for UserSettings {
    fn eq(&self, other: &UserSettings) -> bool {
        self.username == other.username
    }
}

impl UserSettings {
    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn from_string(string: &String) -> Result<UserSettings, ()> {
        match serde_json::from_str(string.as_str()) {
            Ok(n) => Ok(n),
            Err(_) => Err(()),
        }
    }

    pub fn new(username: &String) -> UserSettings {
        UserSettings { username: username.clone(), public: vec![] }
    }

}


#[derive(Eq, Clone, Debug, Deserialize, Serialize)]
pub struct User {
    username: String,
    hash: String,
    salt: String,
    pub settings: UserSettings,
}

impl User {
    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn from_string(string: &String) -> Result<User, ()> {
        match serde_json::from_str(string.as_str()) {
            Ok(n) => Ok(n),
            Err(_) => Err(()),
        }
    }

}

impl Hash for User {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.username.hash(state);
    }
}

impl PartialEq for User {
    fn eq(&self, other: &User) -> bool {
        self.username == other.username
    }
}

pub struct Users {
    path: String,
    users: HashSet<User>,
}

impl Users {
    pub fn new(path: String) -> Users {
        let users = match Users::get_users_file(&path) {
            Ok(n) => n,
            Err(_) => {HashSet::new()},
        };

        Users {path, users}
    }

    pub fn remove_user(&mut self, user: &User) {
        let mut index: usize = 0;

        self.users.remove(&user);
    }

    pub fn get_users(&self) -> Vec<User>{
        self.users.clone().into_iter().collect()
    }

    pub fn find_user(&self, username: &String) -> Option<User> {
        let dummy_user: User = User {
            username: username.clone(),
            hash: String::new(),
            salt: String::new(),
            settings: UserSettings::new(username),
        };

        match self.users.get(&dummy_user) {
            Some(n) => {return Some(n.to_owned())},
            None => {return None},
        };
    }

    pub fn update_user(&mut self, user: User) {
        self.users.remove(&user);
        self.users.insert(user);
    }


}

impl Users {
    fn write_users(path: &String, users: &Vec<User>) -> Result<(), ()> {
        let mut file = match OpenOptions::new()
            .write(true)
            .append(true)
            .open(path) {
                Ok(n) => n,
                Err(_) => {return Err(())}, 
            };
        
        for user in users {
            match writeln!(file, "{}", user.to_string()) {
                Ok(_) => {},
                Err(_) => {return Err(())},
            }
        }
    
        Ok(())
    }
    
    fn get_users_file(path: &String) -> Result<HashSet<User>, String> {
        let mut file = match OpenOptions::new()
        .read(true)
        .open(path) {
            Ok(n) => n,
            Err(n) => {return Err(format!("Error opening file"))},
        };
    
        let mut users: HashSet<User> = HashSet::new();
        
        let mut contenets = String::new();
        match file.read_to_string(&mut contenets) {
            Ok(_) => {},
            Err(_) => {return Err("Error reading from file".to_string())},
        };
    
        for line in contenets.lines() {
            let user = match User::from_string(&line.to_string()) {
                Ok(n) => n,
                Err(_) => {return Err("Error parsing user".to_string())},
            };
    
            users.insert(user);
        }
    
        
        Ok(users)
    }    

    pub fn update_file(&self) {
        let mut file = match OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.path) {
                Ok(n) => n,
                Err(_) => {
                    println!("Error opening file");
                    return;
                },
            };
            
        match file.set_len(0) {
            Ok(_) => {},
            Err(n) => {
                println!("{}", n);
                return;
            },
        };
        match file.seek(SeekFrom::Start(0)) {
            Ok(_) => {},
            Err(n) => {
                println!("{}", n);
                return;
            },
        };

        let _ = Users::write_users(&self.path, &self.get_users());

    }
}

impl Users {
    pub fn add_new_user(&mut self, user: &NewUser) -> Result<(), String> {
        if self.find_user(&user.username) != None {
            return Err("Username already in use".to_string());
        }

        let (hash, salt) = match hash_new(user.password.clone()) {
            Ok(n) => n,
            Err(n) => {return Err(format!("Error hashing password | {}", n));}
        };

        let settings = UserSettings::new(&user.username);

        let u = User {username: user.username.clone(), hash, salt, settings};

        self.users.insert(u);

        Ok(())
    }

    pub fn verify_user(&self, new_user: &NewUser) -> bool{
        let user = match self.find_user(&new_user.username) {
            Some(n) => n,
            None => {return false},
        };

        let hash = hash_old(new_user.password.clone(), user.salt).unwrap();

        hash == user.hash
    }
}


#[test]
fn test_users() {
    let mut users: Users = Users::new("users.txt".to_string());
    let new_user = NewUser {
        username: "user".to_string(),
        password: "password".to_string(),
    };

    users.add_new_user(&new_user).unwrap();

    match users.add_new_user(&new_user) {
        Ok(_) => panic!("cannot add duplicate user"),
        Err(_) => {},
    }

    assert!(users.verify_user(&new_user));

    let new_user2 = NewUser {
        username: "user".to_string(),
        password: "pass".to_string(),
    };

    assert!(!users.verify_user(&new_user2));


    users.update_file();

    let users2: Users = Users::new("users.txt".to_string());


    assert_eq!(users.get_users(), users2.get_users());

    let user = users.find_user(&new_user.username).unwrap();

    users.remove_user(&user);

    match users.find_user(&new_user.username) {
        Some(_) => panic!("user not removed"),
        None => {},
    }

    users.update_file();



}

