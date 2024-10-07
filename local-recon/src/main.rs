use std::io;
use std::env;



pub mod users;
pub mod process;

// Reference: https://crates.io/crates/tasklist
// Test for git


fn main() -> io::Result<()> {

    let mut user_choice = String::new();

    println!("Select an option:");
    println!("\t1. Get local process list");
    println!("\t2. Get local users");
    println!("\t3. Get current user's groups");
    println!("\t4. Get a specific user's groups");
    println!("\t5. Get members of a specific group");

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
        "4" => {
            println!("Enter a username:");
            let mut username = String::new();
            io::stdin().read_line(&mut username)?;
            let username = username.trim();
            match crate::users::get_user_groups(username) {
                Ok(groups) => {
                    if groups.is_empty() {
                        println!("No groups found for user {}", username);
                        ()
                    } else {
                        println!("Groups for user {}:", username);
                        for group in groups {
                            println!("\t{}", group);
                        }
                    }
                }
                Err(err) => eprintln!("Error: {} (User likely doesn't exist)", err),
            }
        }
        "5" => {
            println!("Enter a group name:");
            let mut group_name = String::new();
            io::stdin().read_line(&mut group_name)?;
            let group_name = group_name.trim();
            match crate::users::get_members_of_group(group_name) {
                Ok(members) => {
                    if members.is_empty() {
                        println!("No members found for group {}", group_name);
                        ()
                    } else {
                        println!("Members of group {}:", group_name);
                        for member in members {
                            println!("\t{}", member);
                        }
                    }
                }
                Err(err) => eprintln!("Error: {} (Group likely doesn't exist)", err),
            }
        }
        _ => println!("Invalid choice"),
    }

    Ok(())
}



