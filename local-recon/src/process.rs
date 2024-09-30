use tasklist;

pub fn get_local_processes() -> String {
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