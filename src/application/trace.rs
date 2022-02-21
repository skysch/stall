////////////////////////////////////////////////////////////////////////////////
// Stall configuration management utility
////////////////////////////////////////////////////////////////////////////////
// This code is dual licensed using the MIT or Apache 2 license.
// See license-mit.md and license-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Crate-wide tracing infrastructure.
////////////////////////////////////////////////////////////////////////////////


// External library imports.
use anyhow::Context as _;
use anyhow::Error;
use serde::Deserialize;
use serde::Serialize;
use tracing::subscriber::set_global_default;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::Registry;

// Standard library imports.
use std::borrow::Cow;
use std::fs::OpenOptions;
use std::path::Path;
use std::path::PathBuf;


////////////////////////////////////////////////////////////////////////////////
// Constants
////////////////////////////////////////////////////////////////////////////////
/// Default value for the tracing environment variable.
const TRACE_ENV_VAR: &str = "STALL_TRACE";

/// Default setting for ANSI color usage.
const DEFAULT_ANSI_COLORS: bool = true;


////////////////////////////////////////////////////////////////////////////////
// TraceGuard
////////////////////////////////////////////////////////////////////////////////
/// An RAII guard for managing and flushing trace file writer state.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct TraceGuard {
    /// The worker thread guard for trace output to file.
    file_output_guard: Option<WorkerGuard>,
}


////////////////////////////////////////////////////////////////////////////////
// TraceConfig
////////////////////////////////////////////////////////////////////////////////
/// Tracing configuration parameters.
#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraceConfig {
    /// Trace level filters.
    #[serde(default = "TraceConfig::default_filters")]
    pub filters: Vec<Cow<'static, str>>,
    
    /// The trace output path. If None, the trace will not be written to file.
    #[serde(default = "TraceConfig::default_trace_output_path")]
    pub trace_output_path: Option<PathBuf>,

    /// Whether to write trace output to stdout.
    pub output_stdout: bool,

    /// Whether to use ANSI coloring in the output.
    #[serde(default = "TraceConfig::default_ansi_colors")]
    pub ansi_colors: bool,
}

impl Default for TraceConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl TraceConfig {
    /// Returns a new `TraceConfig` with the default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            filters: Self::default_filters(),
            trace_output_path: Self::default_trace_output_path(),
            output_stdout: true,
            ansi_colors: Self::default_ansi_colors(),
        }
    }

    /// Initializes the global default tracing subscriber for the using this
    /// configuration.
    pub fn init_global_default<L>(
        &self,
        default_level_filter: L)
        -> Result<TraceGuard, Error>
        where L: Into<LevelFilter>
    {
        let mut env_filter_layer = EnvFilter::from_env(TRACE_ENV_VAR)
            .add_directive(default_level_filter.into().into());
        for filter in &self.filters[..] {
            let directive = filter
                .parse()
                .with_context(|| format!(
                    "failed to parse trace filter directive \"{:?}\"",
                    filter))?;
            env_filter_layer = env_filter_layer.add_directive(directive);
        }

        let fmt_layer = match self.output_stdout {
            true => Some(Layer::new()
                .without_time()
                .with_ansi(self.ansi_colors)),
            false => None,
        };

        let (file_output_layer, file_output_guard) = match &self
            .trace_output_path
        {
            Some(trace_output_path) => {
                let path: &Path = trace_output_path.as_ref();
                let file = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(path)
                    .with_context(|| format!(
                        "Failed to create/open trace file for writing: {}",
                        path.display()))?;
                let (writer, guard) = tracing_appender::non_blocking(file);
                let layer = Layer::new()
                    .without_time()
                    .with_ansi(false)
                    .with_writer(writer);
                (Some(layer), Some(guard))
            }
            None => (None, None),
        };

        let subscriber = Registry::default()
            .with(env_filter_layer)
            .with(fmt_layer)
            .with(file_output_layer);

        set_global_default(subscriber)
            .context("failed to set global tracing subscriber")?;

        Ok(TraceGuard {
            file_output_guard,
        })
    }

    /// Returns the default value for filters.
    fn default_filters() -> Vec<Cow<'static, str>> {
        vec![
            "atma=WARN".into(),
        ]
    }

    /// Returns the default trace ouput path.
    const fn default_trace_output_path() -> Option<PathBuf> {
        None
    }

    /// Returns the default setting for ANSI color usage.
    const fn default_ansi_colors() -> bool {
        DEFAULT_ANSI_COLORS
    }
}


