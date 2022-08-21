use super::{StrError, PYTHON_HEADER};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;
use std::sync::{mpsc, Arc, Mutex};
use std::{env, fs, thread};

/// Writes a python file and call python3 on it
///
/// # Arguments
///
/// * `python_commands` - Python commands to be written to file
/// * `output_dir` - Output directory to be created
/// * `filename_py` - Filename with extension .py
///
/// # Note
///
/// The contents of [PYTHON_HEADER] are added at the beginning of the file.
pub(crate) fn call_python3(python_commands: &String, path: &Path) -> Result<String, StrError> {
    // create directory
    if let Some(p) = path.parent() {
        fs::create_dir_all(p).map_err(|_| "cannot create directory")?;
    }

    // combine header with commands
    let mut contents = String::new();
    contents.push_str(PYTHON_HEADER);
    contents.push_str(python_commands);

    // write file
    let mut file = File::create(path).map_err(|_| "cannot create file")?;
    file.write_all(contents.as_bytes()).map_err(|_| "cannot write file")?;

    // force sync
    file.sync_all().map_err(|_| "cannot sync file")?;

    // execute file
    let mut python = String::from("python3");
    if let Ok(v) = env::var("PLOTPY_PYTHON") {
        python = v;
    }
    let output = Command::new(python)
        .arg(path)
        .output()
        .map_err(|_| "cannot run python3")?;
    // results
    let out = String::from_utf8(output.stdout).unwrap();
    let err = String::from_utf8(output.stderr).unwrap();
    let mut results = String::new();
    if out.len() > 0 {
        results.push_str(&out);
    }
    if err.len() > 0 {
        results.push_str(&err)
    }

    // done
    Ok(results)
}

// 运行python且带退出信号
pub(crate) fn call_python3_signal<F>(python_commands: &String, path: &Path, call_signal: F) -> Result<String, StrError>
where
    F: FnOnce(mpsc::Sender<bool>) + Send + 'static,
{
    // create directory
    if let Some(p) = path.parent() {
        fs::create_dir_all(p).map_err(|_| "cannot create directory")?;
    }

    // combine header with commands
    let mut contents = String::new();
    contents.push_str(PYTHON_HEADER);
    contents.push_str(python_commands);
    contents.push_str("input(\"Press any key to close\")");

    // write file
    let mut file = File::create(path).map_err(|_| "cannot create file")?;
    file.write_all(contents.as_bytes()).map_err(|_| "cannot write file")?;

    // force sync
    file.sync_all().map_err(|_| "cannot sync file")?;

    // execute file
    let mut python = String::from("python3");
    if let Ok(v) = env::var("PLOTPY_PYTHON") {
        python = v;
    }
    let child = Command::new(python)
        .arg(path)
        .spawn()
        .map_err(|_| "cannot run python3")?;

    let child_arc = Arc::new(Mutex::new(child));
    let child_sub = child_arc.clone();
    let (send, recv) = mpsc::channel();
    let send_self = send.clone();
    thread::spawn(|| {
        call_signal(send);
    });
    thread::spawn(move || {
        let child = child_sub.clone();
        let mut child = child.lock().unwrap();
        let _ = child.wait();
        drop(child);
        let _ = send_self.send(true);
    });

    if let Ok(_) = recv.recv() {
        let child = child_arc.clone();
        let mut child = child.lock().unwrap();
        let _ = child.kill();
        drop(child);
    }

    let child = child_arc.clone();
    let mut child = child.lock().unwrap();
    let mut out = String::new();
    let mut err = String::new();
    if let Some(mut stdout) = child.stdout.take() {
        let _ = stdout.read_to_string(&mut out);
    }
    if let Some(mut stderr) = child.stderr.take() {
        let _ = stderr.read_to_string(&mut err);
    }
    let mut results = String::new();
    if out.len() > 0 {
        results.push_str(&out);
    }
    if err.len() > 0 {
        results.push_str(&err)
    }

    // done
    Ok(results)
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::{call_python3, call_python3_signal, StrError, PYTHON_HEADER};
    use std::path::Path;
    use std::process::Command;
    use std::time::Duration;
    use std::{fs, thread};

    const OUT_DIR: &str = "/tmp/plotpy/unit_tests";

    #[test]
    fn call_python3_() {
        let mut child = Command::new("python3.9").spawn().expect("failed to execute child");

        let ecode = child.wait().expect("failed to wait on child");
        println!("{:?}", ecode);
    }

    #[test]
    fn call_python3_signal_works() -> Result<(), StrError> {
        let commands = "print(\"Python says: Hello World!\")".to_string();
        let path = Path::new("call_python3_works.py");
        let output = call_python3_signal(&commands, &path, |send| {
            thread::sleep(Duration::from_secs(3));
            let _ = send.send(true);
        })?;
        let data = fs::read_to_string(&path).map_err(|_| "cannot read test file")?;
        let mut correct = String::from(PYTHON_HEADER);
        correct.push_str(&commands);
        assert_eq!(data, correct);
        println!("output: {}", output);
        // assert_eq!(output, "Python says: Hello World!\n");
        Ok(())
    }

    #[test]
    fn call_python3_works() -> Result<(), StrError> {
        let commands = "print(\"Python says: Hello World!\")".to_string();
        let path = Path::new("call_python3_works.py");
        let output = call_python3(&commands, &path)?;
        let data = fs::read_to_string(&path).map_err(|_| "cannot read test file")?;
        let mut correct = String::from(PYTHON_HEADER);
        correct.push_str(&commands);
        assert_eq!(data, correct);
        assert_eq!(output, "Python says: Hello World!\n");
        Ok(())
    }

    #[test]
    fn call_python3_create_dir_works() -> Result<(), StrError> {
        let commands = "print(\"Python says: Hello World!\")".to_string();
        let path = Path::new(OUT_DIR).join("call_python3_works.py");
        let output = call_python3(&commands, &path)?;
        let data = fs::read_to_string(&path).map_err(|_| "cannot read test file")?;
        let mut correct = String::from(PYTHON_HEADER);
        correct.push_str(&commands);
        assert_eq!(data, correct);
        assert_eq!(output, "Python says: Hello World!\n");
        Ok(())
    }

    #[test]
    fn call_python3_twice_works() -> Result<(), StrError> {
        let path = Path::new(OUT_DIR).join("call_python3_twice_works.py");
        // first
        let commands_first = "print(\"Python says: Hello World!\")".to_string();
        let output_first = call_python3(&commands_first, &path)?;
        let data_first = fs::read_to_string(&path).map_err(|_| "cannot read test file")?;
        let mut correct_first = String::from(PYTHON_HEADER);
        correct_first.push_str(&commands_first);
        assert_eq!(data_first, correct_first);
        assert_eq!(output_first, "Python says: Hello World!\n");
        // second
        let commands_second = "print(\"Python says: Hello World! again\")".to_string();
        let output_second = call_python3(&commands_second, &path)?;
        let data_second = fs::read_to_string(&path).map_err(|_| "cannot read test file")?;
        let mut correct_second = String::from(PYTHON_HEADER);
        correct_second.push_str(&commands_second);
        assert_eq!(data_second, correct_second);
        assert_eq!(output_second, "Python says: Hello World! again\n");
        Ok(())
    }
}
