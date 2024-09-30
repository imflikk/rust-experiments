use std::io;
use std::env;



pub mod users;
pub mod process;

// Reference: https://crates.io/crates/tasklist


fn main() -> io::Result<()> {

    let mut user_choice = String::new();

    println!("Select an option:");
    println!("\t1. Get local process list");
    println!("\t2. Get local users");
    println!("\t3. Get current user's groups");

    io::stdin().read_line(&mut user_choice)?;

    user_choice = user_choice.to_string();

    match user_choice.as_str().trim() {
        "1" => {
            let local_processes = process::get_local_processes();
            println!("{}", local_processes);
        }
        "2" => {
            match crate::users::get_local_users() {
                Ok(users) => {
                    println!("Local users:");
                    for user in users {
                        // Need to fix this, it panics at the moment.
                        // Likely something to do with checking for the admin group
                        let local_groups = crate::users::get_user_groups(&user).unwrap();
                        if local_groups.iter().any(|g| g.contains("Administrators")) {
                            println!("\t{} (Admin)", user);
                        } else {
                            println!("\t{}", user);
                        }
                        
                    }
                }
                Err(err) => eprintln!("Error: {}", err),
            }
        }
        "3" => {
            let current_user = env::var("USERNAME").unwrap();
            match crate::users::get_user_groups(&current_user) {
                Ok(groups) => {
                    println!("Groups for user {}:", current_user);
                    for group in groups {
                        println!("\t{}", group);
                    }
                }
                Err(err) => eprintln!("Error: {}", err),
            }
        }
        _ => println!("Invalid choice"),
    }

    Ok(())
}



