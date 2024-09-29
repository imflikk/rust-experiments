use std::io;

use tasklist;

pub mod users;

// Reference: https://crates.io/crates/tasklist


fn main() -> io::Result<()> {

    let mut user_choice = String::new();

    println!("Select an option:");
    println!("\t1. Get local process list");
    println!("\t2. Get local users");

    io::stdin().read_line(&mut user_choice)?;

    user_choice = user_choice.to_string();

    match user_choice.as_str().trim() {
        "1" => {
            let local_processes = get_local_processes();
            println!("{}", local_processes);
        }
        "2" => {
            match crate::users::get_local_users() {
                Ok(users) => {
                    println!("Local users:");
                    for user in users {
                        println!("\t{}", user);
                    }
                }
                Err(err) => eprintln!("Error: {}", err),
            }
        }
        _ => println!("Invalid choice"),
    }

    Ok(())
}

fn get_local_processes() -> String {
    unsafe {
        let tl = tasklist::Tasklist::new();
        let mut formatted_task_string = String::new();

        // The formatting for the 2nd column is off no matter what I do, will try to fix later
        for i in tl {
            formatted_task_string += &format!("{:<10}|{:<20}|{:<30.30}|\n", i.get_pid(), i.get_pname(), i.get_user());
        }

        formatted_task_string
    }
}

