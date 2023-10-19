use std::fs;
use std::io::{Read, Result as IoResult};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
  #[clap(help="envvar name")]
  envvar: String,
  #[clap(short='c', long="cmd", help="show command line")]
  show_cmd: bool,
}

#[derive(Eq, PartialEq, Hash, Debug, PartialOrd, Ord, Clone)]
enum EnvVal {
  Fail,
  Nothing,
  Value(String),
}

type Results = Vec<(EnvVal, u32, Option<String>)>;

fn get_envval(mut path: PathBuf, name: &str) -> IoResult<Option<String>> {
  path.push("environ");
  let mut buffer = vec![];
  let mut f = fs::File::open(&path)?;
  f.read_to_end(&mut buffer)?;
  let r = buffer.split(|c| *c == 0)
    .find(|v| v.starts_with(name.as_bytes()))
    .and_then(|v| {
      v.splitn(2, |c| *c == b'=')
        .nth(1)
        .map(|s| String::from_utf8_lossy(s).into_owned())
    });
  Ok(r)
}

fn get_cmdline(p: &Path) -> String {
  let cmdline_path = p.join("cmdline");
  let mut buf = String::new();
  let mut file = match fs::File::open(cmdline_path) {
    Ok(f) => f,
    Err(_) => return String::new(),
  };
  match file.read_to_string(&mut buf) {
    Ok(_) => (),
    Err(_) => return String::new(),
  };
  chop_null(buf)
}

fn chop_null(mut s: String) -> String {
  if s.is_empty() {
    return s;
  }
  let last = s.len() - 1;
  if !s.is_empty() && s.as_bytes()[last] == 0 {
    s.truncate(last);
  }
  s.replace('\0', " ")
}

fn main() -> IoResult<()> {
  let args = Cli::parse();
  let name_prefix = args.envvar + "=";
  let results: Results = fs::read_dir("/proc")?
    .filter_map(|entry| {
      if let Ok(entry) = entry {
        let path = entry.path();
        if let Ok(pid) = path.file_name().unwrap().to_str().unwrap().parse() {
          Some((path, pid))
        } else {
          None
        }
      } else {
        None
      }
    }).map(|(path, pid)| {
      let cmd = args.show_cmd.then(|| get_cmdline(&path));
      let v = get_envval(path, &name_prefix);
      let v = match v {
        Ok(Some(s)) => EnvVal::Value(s),
        Ok(None) => EnvVal::Nothing,
        Err(_) => EnvVal::Fail,
      };
      (v, pid, cmd)
    }).collect();

  if args.show_cmd {
    show_results_long(results);
  } else {
    show_results_short(results);
  }

  Ok(())
}

fn show_results_short(results: Results) {
  let mut map = HashMap::new();
  for (v, pid, _) in results {
    map.entry(v).or_insert_with(Vec::new).push(pid);
  }

  let mut r = map.into_iter().collect::<Vec<(EnvVal, Vec<u32>)>>();
  r.sort_unstable_by_key(|(_, pids)| pids.len());

  for (v, pids) in r {
    println!("{:5} {:?} ({:?})", pids.len(), v, pids);
  }
}

fn show_results_long(mut results: Results) {
  results.sort_by_cached_key(|(env, pid, _)| (env.clone(), *pid));

  for (v, pid, cmdline) in results {
    println!("{:7} {:20} {}", pid, v, cmdline.unwrap());
  }
}

use std::fmt::{Display, Formatter, Error};
impl Display for EnvVal {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
    match self {
      Self::Nothing => f.pad("Nothing"),
      Self::Fail => f.pad("Fail"),
      Self::Value(v) => f.pad(&format!("\"{}\"", v)),
    }
  }
}
