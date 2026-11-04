use clap::{Parser, Subcommand, ValueEnum};
use mathlang_verification::{hash_bytes, proof::FormalStatus};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Component, Path, PathBuf},
};
pub mod commands;
pub mod process;
pub const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/..");

#[derive(Clone, Copy, Debug, ValueEnum, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Text,
    Json,
}
#[derive(Parser, Debug)]
#[command(
    name = "mathlang",
    version,
    about = "Unified MathLang CLI; existing mathematical engines and immutable evidence"
)]
pub struct Cli {
    #[arg(long, global = true, value_enum)]
    pub format: Option<Format>,
    #[arg(long, global = true)]
    pub output: Option<PathBuf>,
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,
    #[arg(long, global = true)]
    pub quiet: bool,
    #[arg(long, global = true)]
    pub verbose: bool,
    #[command(subcommand)]
    pub command: Command,
}
#[derive(Subcommand, Debug)]
pub enum Command {
    Model {
        #[command(subcommand)]
        action: ModelAction,
    },
    Verify {
        input: String,
    },
    Proof {
        #[command(subcommand)]
        action: ProofAction,
    },
    Knowledge {
        #[command(subcommand)]
        action: KnowledgeAction,
    },
    Compare {
        left: PathBuf,
        right: PathBuf,
    },
    Evidence {
        #[command(subcommand)]
        action: EvidenceAction,
    },
    Solve {
        problem: String,
    },
    /// Exact expressions, statistics, geometry, classical physics and finite quantum states from JSON.
    Compute {
        input: PathBuf,
    },
    Doctor,
}
#[derive(Subcommand, Debug)]
pub enum ModelAction {
    Check { input: PathBuf },
    Inspect { input: PathBuf },
    Normalize { input: PathBuf },
}
#[derive(Subcommand, Debug)]
pub enum ProofAction {
    Plan { input: PathBuf },
    Counterexample { input: PathBuf },
}
#[derive(Subcommand, Debug)]
pub enum KnowledgeAction {
    Inspect {
        input: PathBuf,
    },
    Impact {
        input: PathBuf,
        #[arg(long)]
        revision: u64,
    },
    Update {
        input: PathBuf,
    },
}
#[derive(Subcommand, Debug)]
pub enum EvidenceAction {
    Show {
        input: PathBuf,
        #[arg(long)]
        obligation: Option<PathBuf>,
        #[arg(long)]
        registry: Option<PathBuf>,
    },
    Validate {
        input: PathBuf,
        #[arg(long)]
        obligation: Option<PathBuf>,
        #[arg(long)]
        registry: Option<PathBuf>,
    },
}
#[derive(Debug, Serialize)]
pub struct Error {
    pub error_code: String,
    pub category: &'static str,
    pub message: String,
    pub command: String,
    pub recoverable: bool,
    pub evidence_reference: Option<String>,
    #[serde(skip)]
    pub exit: i32,
}
impl Error {
    pub fn new(exit: i32, message: impl ToString) -> Self {
        let category = match exit {
            2 => "INPUT",
            3 => "MODEL",
            4 => "OUT_OF_SCOPE",
            5 => "ENVIRONMENT",
            6 => "RESOURCE_LIMIT",
            7 => "EVIDENCE",
            8 => "FILESYSTEM",
            _ => "INTERNAL",
        };
        Self {
            error_code: format!("MLCLI-{exit}"),
            category,
            message: message.to_string(),
            command: String::new(),
            recoverable: exit != 9,
            evidence_reference: None,
            exit,
        }
    }
}
pub type Result<T> = std::result::Result<T, Error>;
#[derive(Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub cli: CliConfig,
    pub runtime: RuntimeConfig,
    pub verification: VerificationConfig,
    pub evidence: EvidenceConfig,
}
#[derive(Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct CliConfig {
    pub output_format: Format,
}
impl Default for CliConfig {
    fn default() -> Self {
        Self {
            output_format: Format::Text,
        }
    }
}
#[derive(Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct RuntimeConfig {
    pub mode: String,
}
impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            mode: "local".into(),
        }
    }
}
#[derive(Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct VerificationConfig {
    pub max_rss_mib: u64,
    pub timeout_seconds: u64,
    pub parallelism: u64,
}
impl Default for VerificationConfig {
    fn default() -> Self {
        Self {
            max_rss_mib: 2048,
            timeout_seconds: 300,
            parallelism: 1,
        }
    }
}
#[derive(Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct EvidenceConfig {
    pub directory: PathBuf,
}
impl Default for EvidenceConfig {
    fn default() -> Self {
        Self {
            directory: "./evidence".into(),
        }
    }
}
impl Config {
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let path = path.or_else(|| {
            Path::new("mathlang.toml")
                .exists()
                .then_some(Path::new("mathlang.toml"))
        });
        let c: Self = match path {
            Some(p) => toml::from_str(&fs::read_to_string(p).map_err(|e| Error::new(8, e))?)
                .map_err(|e| Error::new(2, e))?,
            None => Self::default(),
        };
        if c.runtime.mode != "local" {
            return Err(Error::new(4, "only local runtime is supported"));
        }
        if c.verification.max_rss_mib != 2048
            || c.verification.timeout_seconds != 300
            || c.verification.parallelism != 1
        {
            return Err(Error::new(4,"existing runner fixes RSS=2048 MiB, timeout=300 seconds, parallelism=1; alternative limits are unsupported"));
        }
        Ok(c)
    }
}
pub fn read<T: DeserializeOwned + Serialize>(p: &Path) -> Result<T> {
    let raw: Value = serde_json::from_slice(&fs::read(p).map_err(|e| Error::new(8, e))?)
        .map_err(|e| Error::new(2, e))?;
    decode(raw)
}
pub fn decode<T: DeserializeOwned + Serialize>(raw: Value) -> Result<T> {
    let t: T = serde_json::from_value(raw.clone()).map_err(|e| {
        Error::new(
            if e.to_string().contains("unknown variant") {
                4
            } else {
                2
            },
            e,
        )
    })?;
    fn keys(a: &Value, b: &Value) -> Result<()> {
        match (a, b) {
            (Value::Object(a), Value::Object(b)) => {
                for (k, v) in a {
                    keys(
                        v,
                        b.get(k)
                            .ok_or_else(|| Error::new(2, format!("unknown field: {k}")))?,
                    )?;
                }
            }
            (Value::Array(a), Value::Array(b)) => {
                for (x, y) in a.iter().zip(b) {
                    keys(x, y)?;
                }
            }
            _ => (),
        }
        Ok(())
    }
    keys(
        &raw,
        &serde_json::to_value(&t).map_err(|e| Error::new(9, e))?,
    )?;
    Ok(t)
}
pub fn safe_path(path: &Path) -> Result<()> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| Error::new(8, e))?
            .join(path)
    };
    let mut p = PathBuf::new();
    for c in absolute.components() {
        if c == Component::ParentDir {
            return Err(Error::new(
                8,
                "parent traversal is not allowed for output/evidence paths",
            ));
        }
        p.push(c);
        match fs::symlink_metadata(&p) {
            Ok(m) if m.file_type().is_symlink() => {
                return Err(Error::new(
                    8,
                    format!("symbolic link rejected: {}", p.display()),
                ))
            }
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(Error::new(8, e)),
            _ => (),
        }
    }
    Ok(())
}
pub fn save(p: &Path, v: &impl Serialize) -> Result<()> {
    safe_path(p)?;
    let bytes = serde_json::to_vec_pretty(v).map_err(|e| Error::new(9, e))?;
    mathlang_verification::evidence::write_new(p, &bytes).map_err(|e| Error::new(8, e))
}
pub fn new_directory(p: &Path) -> Result<()> {
    safe_path(p)?;
    if let Some(parent) = p.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|e| Error::new(8, e))?;
    }
    fs::create_dir(p).map_err(|e| Error::new(8, e))
}
pub fn envelope(command: &str, run: &str) -> Value {
    json!({"schema_version":"mathlang.cli/0.1","command":command,"execution_status":"COMPLETED","formal_status":null,"run_id":run,"model_id":null,"model_revision":null,"result":{},"evidence":{"reference":null},"error":null})
}
pub fn failure(e: &mut Value, mut error: Error, command: &str) -> i32 {
    let code = error.exit;
    error.command = command.into();
    if e["formal_status"].is_null() && command == "verify" {
        e["formal_status"] = json!(FormalStatus::Error);
    }
    e["execution_status"] = json!(if code == 6 { "INTERRUPTED" } else { "ERROR" });
    error.evidence_reference = e["evidence"]["reference"].as_str().map(str::to_owned);
    e["error"] = json!(error);
    code
}
pub fn run_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    format!(
        "run-{}-{}-{}-{}",
        mathlang_verification::evidence::now(),
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed),
        hash_bytes(format!("{:?}", std::time::SystemTime::now()).as_bytes())[..8].to_owned()
    )
}
