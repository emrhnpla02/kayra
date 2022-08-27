use std::convert::AsRef;
use std::default;
use std::fs;
use std::io;
use std::path;
use std::process;

#[derive(strum::AsRefStr, strum::EnumString)]
#[strum(serialize_all = "kebab_case")]
pub enum Manager {
  Npm,
  Yarn,
  Pnpm,
}

impl Default for Manager {
  fn default() -> Self {
    Manager::Npm
  }
}

#[derive(Default)]
pub struct PackageManager<'a> {
  manager: Manager,
  command: &'a str,
  path: Option<path::PathBuf>,
  packages: Vec<&'a str>,
  flags: Vec<&'a str>,
}

impl PackageManager<'_> {
  pub fn new(manager: Manager) -> Self {
    Self {
      manager,
      ..default::Default::default()
    }
  }
}

impl<'a> PackageManager<'a> {
  pub fn dir(mut self, relative_path: &str) -> Self {
    self.path = Some(path::PathBuf::from(relative_path));
    self
  }

  pub fn global(mut self) -> Self {
    self.flags.push("--global");
    self
  }

  pub fn flags(mut self, flags: &[&'a str]) -> Self {
    self.flags = flags.to_vec();
    self
  }

  pub fn install(mut self, packages: &[&'a str]) -> Self {
    self.packages = packages.to_vec();
    self.command = match self.manager.as_ref() {
      "npm" => "install",
      "pnpm" | "yarn" => "add",
      _ => "",
    };
    self
  }

  pub fn remove(mut self, packages: &[&'a str]) -> Self {
    self.packages = packages.to_vec();
    self.command = match self.manager.as_ref() {
      "npm" | "yarn" | "pnpm" => "remove",
      _ => "",
    };
    self
  }

  pub fn run(self) -> io::Result<process::Output> {
    let (package_manager, path, args) = self.collect()?;

    let result = process::Command::new(package_manager.as_ref())
      .stdout(process::Stdio::piped())
      .current_dir(path)
      .args(args)
      .output()?;

    Ok(result)
  }

  pub async fn async_run(self) -> io::Result<async_process::Child> {
    let (package_manager, path, args) = self.collect()?;

    let result = async_process::Command::new(package_manager.as_ref())
      .stdout(async_process::Stdio::piped())
      .current_dir(path)
      .args(args)
      .spawn()?;

    Ok(result)
  }
}

type PMOptions<'a> = (Manager, path::PathBuf, Vec<&'a str>);

impl<'a> PackageManager<'a> {
  fn collect(self) -> io::Result<PMOptions<'a>> {
    let Self {
      manager,
      command,
      path,
      packages,
      flags,
    } = self;

    if command.is_empty() {
      return Err(io::Error::new(
        io::ErrorKind::Other,
        "Unsupported package manager.",
      ));
    }

    let path = {
      let relative_path = path.unwrap_or_else(|| path::PathBuf::from("./"));

      match fs::canonicalize(&relative_path) {
        Ok(absolute_path) => absolute_path,
        Err(_err) => {
          fs::create_dir_all(&relative_path)?;
          fs::canonicalize(relative_path)?
        }
      }
    };

    if packages.is_empty() {
      match command {
        "install" | "add" | "remove" => {
          return Err(io::Error::new(io::ErrorKind::Other, "Missing parameter."))
        }
        _ => (),
      };
    };

    let mut args = vec![command];
    args.extend(packages);

    if !flags.is_empty() {
      args.extend(flags);
    }

    Ok((manager, path, args))
  }
}
