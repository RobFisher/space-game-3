use space_game_ephemeris::{
    fetch_profile_assets, resolve_asset_path, resolved_asset_root, verify_profile_assets,
    AssetVerificationStatus, EphemerisAssetManifest, ASSET_ROOT_ENV,
};
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

const DEFAULT_MANIFEST_PATH: &str = "data/ephemeris/manifest.toml";
const DEFAULT_PROFILE: &str = "minimal";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {}", error);
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args = Args::parse(env::args().skip(1))?;
    let manifest = EphemerisAssetManifest::from_toml_path(&args.manifest)
        .map_err(|error| format!("failed to load {}: {}", args.manifest.display(), error))?;
    let root = args.asset_root.unwrap_or_else(resolved_asset_root);

    match args.command {
        Command::List => list_assets(&manifest, &args.profile, &root),
        Command::Verify => verify_assets(&manifest, &args.profile, &root),
        Command::Fetch { force } => fetch_assets(&manifest, &args.profile, &root, force),
    }
}

fn list_assets(
    manifest: &EphemerisAssetManifest,
    profile: &str,
    root: &PathBuf,
) -> Result<(), String> {
    println!("profile: {}", profile);
    println!("asset root: {}", root.display());
    for selected in manifest
        .profile_assets(profile)
        .map_err(|error| error.to_string())?
    {
        let path = resolve_asset_path(root, selected.asset);
        println!(
            "{}\n  kind: {:?}\n  required: {}\n  url: {}\n  path: {}\n  description: {}",
            selected.id,
            selected.asset.kind,
            selected.asset.required,
            selected.asset.url,
            path.display(),
            selected.asset.description.as_deref().unwrap_or("")
        );
    }
    Ok(())
}

fn verify_assets(
    manifest: &EphemerisAssetManifest,
    profile: &str,
    root: &PathBuf,
) -> Result<(), String> {
    match verify_profile_assets(manifest, profile, root) {
        Ok(results) => {
            println!("profile: {}", profile);
            println!("asset root: {}", root.display());
            for result in results {
                match result.status {
                    AssetVerificationStatus::Valid => {
                        println!("ok: {} at {}", result.id, result.path.display());
                    }
                    AssetVerificationStatus::OptionalMissing => {
                        println!("optional missing: {} at {}", result.id, result.path.display());
                    }
                }
            }
            Ok(())
        }
        Err(error) => Err(format!(
            "{}\nsuggestion: cargo run -p space-game-ephemeris --bin ephemeris-assets -- fetch --profile {}",
            error, profile
        )),
    }
}

fn fetch_assets(
    manifest: &EphemerisAssetManifest,
    profile: &str,
    root: &PathBuf,
    force: bool,
) -> Result<(), String> {
    println!("profile: {}", profile);
    println!("asset root: {}", root.display());
    let results =
        fetch_profile_assets(manifest, profile, root, force).map_err(|error| error.to_string())?;
    for result in results {
        println!("ok: {} at {}", result.id, result.path.display());
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct Args {
    command: Command,
    profile: String,
    manifest: PathBuf,
    asset_root: Option<PathBuf>,
}

#[derive(Debug, PartialEq, Eq)]
enum Command {
    List,
    Verify,
    Fetch { force: bool },
}

impl Args {
    fn parse<I>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = String>,
    {
        let mut args = args.into_iter();
        let command = match args.next().as_deref() {
            Some("list") => Command::List,
            Some("verify") => Command::Verify,
            Some("fetch") => Command::Fetch { force: false },
            Some("--help") | Some("-h") | None => return Err(usage()),
            Some(other) => return Err(format!("unknown command {}\n{}", other, usage())),
        };

        let mut parsed = Self {
            command,
            profile: DEFAULT_PROFILE.to_string(),
            manifest: PathBuf::from(DEFAULT_MANIFEST_PATH),
            asset_root: None,
        };

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--profile" => {
                    parsed.profile = args
                        .next()
                        .ok_or_else(|| "--profile requires a value".to_string())?;
                }
                "--manifest" => {
                    parsed.manifest = PathBuf::from(
                        args.next()
                            .ok_or_else(|| "--manifest requires a value".to_string())?,
                    );
                }
                "--asset-root" => {
                    parsed.asset_root =
                        Some(PathBuf::from(args.next().ok_or_else(|| {
                            "--asset-root requires a value".to_string()
                        })?));
                }
                "--force" => match parsed.command {
                    Command::Fetch { ref mut force } => *force = true,
                    _ => return Err("--force is only valid for fetch".to_string()),
                },
                other => return Err(format!("unknown option {}\n{}", other, usage())),
            }
        }

        Ok(parsed)
    }
}

fn usage() -> String {
    format!(
        "usage: ephemeris-assets <list|verify|fetch> [--profile NAME] [--manifest PATH] [--asset-root PATH] [--force]\n\
         default manifest: {}\n\
         default profile: {}\n\
         asset root override env: {}",
        DEFAULT_MANIFEST_PATH, DEFAULT_PROFILE, ASSET_ROOT_ENV
    )
}
