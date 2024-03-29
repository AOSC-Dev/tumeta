use clap::{Parser, ArgAction};
use eyre::{bail, eyre, Report, Result};
use log::{debug, error, info};
use rayon::prelude::*;

use topic_manifest::{Manifest, ManifestCollection};

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::BufWriter;
use std::path::PathBuf;

static ENV_LOG: &str = "TUMETA_LOG";
static ENV_LOG_DEFAULT: &str = "info";

#[derive(Parser)]
#[command(author, version, about)]
pub struct Args {
    /// Path to source file(s) in TOML format
    #[arg(short, long)]
    src: PathBuf,

    /// Path to destination JSON file
    #[arg(short, long)]
    dst: PathBuf,

    /// Ignore errors
    #[arg(short, long, action = ArgAction::SetTrue, default_value_t = false)]
    ignore_error: bool,
}

fn main() -> Result<()> {
    // Setup logger
    if env::var(ENV_LOG).is_err() {
        env::set_var(ENV_LOG, ENV_LOG_DEFAULT);
    }
    pretty_env_logger::init_custom_env(ENV_LOG);

    // Parse arguments
    let args = Args::parse();

    // Check src and dst paths
    if !args.src.exists() {
        bail!("Source path {} does not exist", args.src.to_string_lossy());
    }
    if args.dst.is_dir() {
        bail!(
            "Invalid destination path {}: destination could not be an existing directory",
            args.dst.to_string_lossy()
        );
    }
    let dst_parent = args
        .dst
        .parent()
        .ok_or(eyre!("Failed to get parent path for dst path"))?;
    if !dst_parent.exists() {
        bail!(
            "Parent path of the destination {} does not exist",
            dst_parent.to_string_lossy()
        );
    }

    info!(
        "Searching for TOML manifests in {}",
        args.src.to_string_lossy()
    );
    let manifest: ManifestCollection = jwalk::WalkDir::new(args.src)
        .follow_links(true)
        .into_iter()
        .par_bridge()
        .filter_map(|res| {
            let entry = res.ok()?;
            if entry.file_type().is_file() {
                let path = entry.path();
                if path.extension()?.to_ascii_lowercase() == "toml" {
                    Some(path)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .map(|p| {
            debug!("Parsing {}", p.to_string_lossy());

            let parsed = toml::from_str(&fs::read_to_string(&p)?).map_err(|e| {
                Report::new(e).wrap_err(format!("Failed to parse {}", p.to_string_lossy()))
            })?;
            let name = p
                .file_stem()
                .ok_or(eyre!(
                    "Invalid topic manifest filename: {}",
                    p.to_string_lossy()
                ))?
                .to_string_lossy()
                .to_string();
            Ok((name, parsed))
        })
        .filter_map(|r: Result<(String, Manifest), Report>| {
            if let Err(e) = r {
                error!("{:#}", e);
                if ! args.ignore_error {
                    panic!("Failed to parse source file(s)");
                }
                None
            } else {
                r.ok()
            }
        })
        .collect::<BTreeMap<String, Manifest>>()
        .into();

    // Check consistency of the file
    let inconsistency = manifest.find_missing_topics();
    for (topic, missing) in &inconsistency {
        error!("Missing dependency for cumulative topic {}: {:?}", topic, missing);
    }
    if (! inconsistency.is_empty()) && (! args.ignore_error) {
        bail!("Topic manifests are inconsistent, abort");
    }

    // Write to dst file
    info!(
        "Writing {} entries to {}",
        manifest.len(),
        args.dst.to_string_lossy()
    );
    let out_file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(args.dst)?;
    let writer = BufWriter::new(out_file);
    serde_json::to_writer_pretty(writer, &manifest)?;
    info!("Done");

    Ok(())
}
