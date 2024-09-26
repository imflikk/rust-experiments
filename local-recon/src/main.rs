use tasklist;

// Reference: https://crates.io/crates/tasklist


fn main() {

    unsafe {
        let tl = tasklist::Tasklist::new();

        // The formatting for the 2nd column is off no matter what I do, will try to fix later
        for i in tl {
            println!("{:<10}|{:<20}|{:<30.30}|", i.get_pid(), i.get_pname(), i.get_user());

        }
    }
    
}
