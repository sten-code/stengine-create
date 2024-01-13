use std::fs;
use std::io::{self, BufRead, Write};
use std::path;
use std::process;

fn main() -> io::Result<()> {
    let mut input = String::new();

    print!("What is your project name? ");
    io::stdout().flush().unwrap();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let name = input.trim();

    let mut test = name.to_string();
    if test
        .chars()
        .next()
        .expect("Name too short")
        .is_ascii_uppercase()
    {
        test = format!(
            "{}{}",
            test.chars()
                .next()
                .expect("Name too short")
                .to_ascii_lowercase(),
            &test[1..]
        );
    }
    // Convert to lowercase and replace spaces with hyphens (TestProject to test-project)
    let dir_name = &test
        .chars()
        .flat_map(|c| {
            if c.is_ascii_uppercase() {
                // Convert uppercase to lowercase
                vec!['-', c.to_ascii_lowercase()]
            } else if c.is_whitespace() {
                // Replace spaces with hyphens
                vec!['-']
            } else {
                // Keep other characters unchanged
                vec![c]
            }
        })
        .collect::<String>();

    if let Ok(_) = fs::metadata(dir_name) {
        eprintln!("A path with the name '{dir_name}' already exists.");
        process::exit(1);
    } else {
        // Clone the GitHub repository into the target directory
        let mut git_clone = process::Command::new("git");
        git_clone.arg("clone");
        git_clone.arg("https://github.com/sten-code/stengine.git");
        git_clone.arg(dir_name);
        git_clone.arg("--recursive");

        match git_clone.status() {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error cloning repository: {e}");
                process::exit(1);
            }
        }
    }

    // Create dir_name/project
    create_folder(format!("{}/{}", dir_name, name));
    create_folder(format!("{}/{}/assets", dir_name, name));
    create_folder(format!("{}/{}/src", dir_name, name));

    {
        let mut file = fs::File::create(format!("{}/{}/src/{}App.cpp", dir_name, name, name))?;
        writeln!(&mut file, "#include <Stengine.h>")?;
        writeln!(&mut file, "#include <Stengine/Core/EntryPoint.h>")?;
        writeln!(&mut file, "")?;
        writeln!(&mut file, "class {name} : public Sten::Application")?;
        writeln!(&mut file, "{{")?;
        writeln!(&mut file, "public:")?;
        writeln!(
            &mut file,
            "    {name}(Sten::ApplicationCommandLineArgs args)"
        )?;
        writeln!(
            &mut file,
            "        : Application(Sten::WindowProps(\"{name}\", 1280, 720), args)",
        )?;
        writeln!(&mut file, "    {{")?;
        writeln!(&mut file, "        ST_INFO(\"Hello, World!\");")?;
        writeln!(&mut file, "    }}")?;
        writeln!(&mut file, "")?;
        writeln!(&mut file, "    ~{name}()")?;
        writeln!(&mut file, "    {{")?;
        writeln!(&mut file, "    }}")?;
        writeln!(&mut file, "}};")?;
        writeln!(&mut file, "")?;
        writeln!(
            &mut file,
            "Sten::Application* Sten::CreateApplication(Sten::ApplicationCommandLineArgs args)"
        )?;
        writeln!(&mut file, "{{")?;
        writeln!(&mut file, "    return new {name}(args);")?;
        writeln!(&mut file, "}}")?;
    }

    // Move fonts into the assets folder of the new project
    match fs::rename(
        format!("{}/Steditor/assets/fonts", dir_name),
        format!("{}/{}/assets/fonts", dir_name, name),
    ) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Couldn't move fonts folder into assets: {e}");
            process::exit(1);
        }
    }

    // Move shaders into the assets folder of the new project
    match fs::rename(
        format!("{}/Steditor/assets/shaders", dir_name),
        format!("{}/{}/assets/shaders", dir_name, name),
    ) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Couldn't move shaders folder into assets: {e}");
            process::exit(1);
        }
    }

    // Move premake5.lua into the new project
    match fs::rename(
        format!("{}/Steditor/premake5.lua", dir_name),
        format!("{}/{}/premake5.lua", dir_name, name),
    ) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Couldn't move premake5.lua: {e}");
            process::exit(1);
        }
    }

    {
        let filepath = format!("{}/{}/premake5.lua", dir_name, name);
        let mut lines: Vec<String> = match read_lines(&filepath) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Couldn't read file premake5.lua: {e}");
                process::exit(1);
            }
        }
        .filter_map(|l| l.ok())
        .collect();

        if let Some(first_line) = lines.first_mut() {
            *first_line = format!("project \"{name}\"");
        }

        let mut new_file = fs::File::create(&filepath)?;
        for line in lines {
            writeln!(new_file, "{line}")?;
        }
    }

    // Remove default projects
    match fs::remove_dir_all(format!("{}/Sandbox", dir_name)) {
        Ok(_) => {}
        Err(e) => eprintln!("Error removing Sandbox: {e}"),
    }

    match fs::remove_dir_all(format!("{}/Steditor", dir_name)) {
        Ok(_) => {}
        Err(e) => eprintln!("Error removing Sandbox: {e}"),
    }

    {
        let filepath = format!("{}/premake5.lua", dir_name);

        let lines: Vec<String> = match read_lines(&filepath) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Couldn't read file premake5.lua: {e}");
                process::exit(1);
            }
        }
        .filter_map(|l| l.ok())
        .collect();

        // Remove the last 2 lines
        let mut shortened: Vec<&str> = lines
            .iter()
            .take(lines.len() - 2)
            .map(|s| s.as_str())
            .collect();

        let line = format!("include \"{name}\"");
        shortened.push(&line);

        let mut new_file = fs::File::create(&filepath)?;
        for line in shortened {
            writeln!(new_file, "{line}")?;
        }
    }

    Ok(())
}

fn create_folder<P>(path: P)
where
    P: AsRef<path::Path>,
{
    match fs::create_dir(path.as_ref()) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Couldn't create '{}': {e}", path.as_ref().display());
            process::exit(1);
        }
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<fs::File>>>
where
    P: AsRef<path::Path>,
{
    let file = fs::File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
