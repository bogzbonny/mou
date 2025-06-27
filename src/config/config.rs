use {
    super::{
        api_key::ApiKey,
        defaults::{
            default_cache_dir, default_docker_context, default_dockerfile, default_log_dir,
            default_project_name,
        },
        mcp::McpServer,
        tools::Tools,
        CommandConfiguration, LLMConfiguration, LLMConfigurations,
    },
    crate::runtime_settings::RuntimeSettings,
    anyhow::{Context as _, Result},
    config::{Config as ConfigRs, Environment, File},
    serde::{Deserialize, Deserializer, Serialize},
    std::{
        path::{Path, PathBuf},
        str::FromStr,
        time::Duration,
    },
    swiftide::integrations::treesitter::SupportedLanguages,
};

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "default_project_name")]
    pub project_name: String,
    #[serde(alias = "language", deserialize_with = "deserialize_languages")]
    pub languages: Vec<SupportedLanguages>,
    pub llm: Box<LLMConfigurations>,
    pub commands: CommandConfiguration,
    #[serde(default = "default_cache_dir")]
    pub cache_dir: PathBuf,
    #[serde(default = "default_log_dir")]
    pub log_dir: PathBuf,

    /// The agent model to use by default in chats
    #[serde(default)]
    pub agent: SupportedAgentConfigurations,

    #[serde(default)]
    /// Concurrency for indexing
    /// By default for IO-bound LLMs, we assume 4x the number of CPUs
    /// For Ollama, it's the number of CPUs
    indexing_concurrency: Option<usize>,
    #[serde(default)]
    /// Batch size for indexing
    /// By default for IO-bound LLMs, we use a smaller batch size, as we can run it in parallel
    /// For local embeddings it's 256
    indexing_batch_size: Option<usize>,

    /// Opt-out of garbage collection. Useful when running benchmarks
    #[serde(default = "default_indexing_garbage_collection")]
    pub indexing_garbage_collection_enabled: bool,

    #[serde(default)]
    pub docker: DockerConfiguration,

    /// Optional: Backoff configuration for api calls
    /// this is currently only used for open-ai and open-router
    #[serde(default)]
    pub backoff: BackoffConfiguration,

    /// Optional: Use tavily as a search tool
    #[serde(default)]
    pub tavily_api_key: Option<ApiKey>,

    /// Required if using `OpenAI`
    #[serde(default)]
    pub openai_api_key: Option<ApiKey>,

    /// Required if using 'Anthropic'
    #[serde(default)]
    pub anthropic_api_key: Option<ApiKey>,

    /// Required if using `Open Router`
    #[serde(default)]
    pub open_router_api_key: Option<ApiKey>,

    /// Required if using `Azure OpenAI`
    #[serde(default)]
    pub azure_openai_api_key: Option<ApiKey>,

    #[serde(default)]
    pub tool_executor: SupportedToolExecutors,

    /// A list of tool name and whether it is enabled or disabled
    ///
    /// This allows the user to disable tools that are not needed for their workflow. Or enable
    /// tools that are disabled by default
    #[serde(default)]
    pub tools: Tools,

    /// When endless mode is enabled, the agent will keep running until it either cannot complete,
    /// did complete or was manually stopped.
    ///
    /// In addition, the agent is instructed that it cannot ask for feedback, but should try to
    /// complete its task instead.
    ///
    /// When running without a TUI, the agent will always run in endless mode.
    ///
    /// WARN: There currently is _no_ limit for endless mode
    #[serde(default)]
    pub endless_mode: bool,

    /// When `stop_on_empty_messages` is enabled, the agent will stop if the next completion does not
    /// contain new messages.
    ///
    /// Often, agents return a reasoning completion before calling any tools. Previously this would
    /// require the user to manually prompt to continue the task. This falls back to that
    /// behaviour.
    ///
    /// When endless mode is enabled this setting has no effect.
    #[serde(default)]
    pub stop_on_empty_messages: bool,

    /// OpenTelemetry tracing feature toggle
    #[serde(default = "default_otel_enabled")]
    pub otel_enabled: bool,

    /// How the agent will edit files, defaults to whole
    #[serde(default)]
    pub agent_edit_mode: AgentEditMode,

    /// Additional constraints / instructions for the agent
    ///
    /// These are passes to the agent in the system prompt and are rendered in a list. If you
    /// intend to use more complicated instructions, consider adding a file to read in the
    /// repository instead.
    #[serde(default)]
    pub agent_custom_constraints: Option<Vec<String>>,

    #[serde(default)]
    pub ui: UIConfig,

    /// Number of completions before the agent summarizes the conversation.
    /// This is used to steer the agent to focus on the current task. If this value is too small
    /// the agent will have clear loss of context when performing tasks. If this value is too large
    /// the agent will not have focus and not understand what is relevant and important.
    ///
    /// Additionally, summarizing the conversation will reduce the context window which can be
    /// beneficial for APIs with stringent limits on context tokens.
    ///
    /// Defaults to 10.
    #[serde(default = "default_num_completions_for_summary")]
    pub num_completions_for_summary: usize,

    /// Add tools from MCP servers to the agents
    #[serde(default)]
    pub mcp: Option<Vec<McpServer>>,
}

fn deserialize_languages<'de, D>(deserializer: D) -> Result<Vec<SupportedLanguages>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Helper {
        Single(SupportedLanguages),
        Multiple(Vec<SupportedLanguages>),
    }

    let helper = Helper::deserialize(deserializer)?;

    match helper {
        Helper::Single(lang) => Ok(vec![lang]),
        Helper::Multiple(langs) => Ok(langs),
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct UIConfig {
    pub hide_header: bool,
}

fn default_otel_enabled() -> bool {
    false
}

fn default_num_completions_for_summary() -> usize {
    10
}

fn default_indexing_garbage_collection() -> bool {
    true
}

/// Agent session configurations supported by Kwaak
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, Default)]
pub enum SupportedAgentConfigurations {
    /// Single looping agent that has all tools available
    #[default]
    #[serde(alias = "V1")]
    Coding,
    /// A two stage agent, starting with a planning agent that delegates to the coding
    /// agent
    PlanAct,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SupportedToolExecutors {
    #[default]
    Docker,
    Local,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, Default, strum_macros::EnumIs)]
#[serde(rename_all = "kebab-case")]
pub enum AgentEditMode {
    Whole,
    Line,
    #[default]
    Patch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerConfiguration {
    #[serde(default = "default_dockerfile")]
    pub dockerfile: PathBuf,
    #[serde(default = "default_docker_context")]
    pub context: PathBuf,
}

impl Default for DockerConfiguration {
    fn default() -> Self {
        Self {
            dockerfile: "Dockerfile".into(),
            context: ".".into(),
        }
    }
}

/// Backoff configuration for api calls.
/// Each time an api call fails backoff will wait an increasing period of time for each subsequent
/// retry attempt. see <https://docs.rs/backoff/latest/backoff/> for more details.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BackoffConfiguration {
    /// Initial interval in seconds between retries
    pub initial_interval_sec: u64,
    /// The factor by which the interval is multiplied on each retry attempt
    pub multiplier: f64,
    /// Introduces randomness to avoid retry storms
    pub randomization_factor: f64,
    /// Total time all attempts are allowed in seconds. Once a retry must wait longer than this,
    /// the request is considered to have failed.
    pub max_elapsed_time_sec: u64,
}

impl Default for BackoffConfiguration {
    fn default() -> Self {
        Self {
            initial_interval_sec: 15,
            multiplier: 2.0,
            randomization_factor: 0.05,
            max_elapsed_time_sec: 120,
        }
    }
}

impl From<BackoffConfiguration> for backoff::ExponentialBackoff {
    fn from(from_backoff: BackoffConfiguration) -> Self {
        backoff::ExponentialBackoffBuilder::default()
            .with_initial_interval(Duration::from_secs(from_backoff.initial_interval_sec))
            .with_multiplier(from_backoff.multiplier)
            .with_randomization_factor(from_backoff.randomization_factor)
            .with_max_elapsed_time(Some(Duration::from_secs(from_backoff.max_elapsed_time_sec)))
            .build()
    }
}

impl FromStr for Config {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        toml::from_str(s)
            .context("Failed to parse configuration")
            .and_then(Config::fill_llm_api_keys)
            .map(Config::add_project_name_to_paths)
    }
}

impl Config {
    #[must_use]
    pub fn runtime_settings(&self) -> RuntimeSettings {
        RuntimeSettings::from_config(self)
    }

    pub fn load(path: Option<&Path>) -> Result<Self> {
        let builder = match path {
            Some(path) => ConfigRs::builder().add_source(File::from(path)),
            None => {
                // Support both kwaak.toml and .config/kwaak.toml
                // Check if they exist and create the builder accordingly
                if std::fs::metadata(".config/mou.toml").is_ok() {
                    ConfigRs::builder().add_source(File::with_name(".config/mou.toml"))
                } else {
                    ConfigRs::builder().add_source(File::with_name("mou.toml"))
                }
            }
        };

        let config = builder
            .add_source(File::with_name("mou.local").required(false))
            .add_source(Environment::with_prefix("MOU").separator("__"))
            .build()
            .context("Failed to build configuration")?;

        config
            .try_deserialize()
            .map_err(Into::into)
            .and_then(Config::fill_llm_api_keys)
            .map(Config::add_project_name_to_paths)
    }

    #[must_use]
    pub fn language_extensions(&self) -> Vec<&str> {
        let mut extensions = vec![];

        for lang in &self.languages {
            extensions.extend(lang.file_extensions());
        }

        extensions
    }

    // Seeds the api keys into the LLM configurations
    fn fill_llm_api_keys(mut self) -> Result<Self> {
        let previous = self.clone();

        let LLMConfigurations {
            indexing,
            embedding,
            query,
        } = &mut *self.llm;

        for config in &mut [indexing, embedding, query] {
            let maybe_root_key = previous.root_provider_api_key_for(config);
            fill_llm(config, maybe_root_key)?;
        }
        Ok(self)
    }

    fn add_project_name_to_paths(mut self) -> Self {
        if self.cache_dir.ends_with("mou") {
            self.cache_dir.push(&self.project_name);
        }
        if self.log_dir.ends_with("mou/logs") {
            self.log_dir.push(&self.project_name);
        }

        self
    }

    #[must_use]
    fn root_provider_api_key_for(&self, provider: &LLMConfiguration) -> Option<&ApiKey> {
        match provider {
            LLMConfiguration::OpenAI { .. } => self.openai_api_key.as_ref(),
            LLMConfiguration::OpenRouter { .. } => self.open_router_api_key.as_ref(),
            LLMConfiguration::Anthropic { .. } => self.anthropic_api_key.as_ref(),
            LLMConfiguration::AzureOpenAI { .. } => self.azure_openai_api_key.as_ref(),
            _ => None,
        }
    }

    #[must_use]
    pub fn indexing_provider(&self) -> &LLMConfiguration {
        let LLMConfigurations { indexing, .. } = &*self.llm;
        indexing
    }

    #[must_use]
    pub fn embedding_provider(&self) -> &LLMConfiguration {
        let LLMConfigurations { embedding, .. } = &*self.llm;
        embedding
    }

    #[must_use]
    pub fn query_provider(&self) -> &LLMConfiguration {
        let LLMConfigurations { query, .. } = &*self.llm;
        query
    }

    #[must_use]
    pub fn cache_dir(&self) -> &Path {
        self.cache_dir.as_path()
    }

    #[must_use]
    pub fn log_dir(&self) -> &Path {
        self.log_dir.as_path()
    }

    #[must_use]
    pub fn indexing_concurrency(&self) -> usize {
        if let Some(concurrency) = self.indexing_concurrency {
            return concurrency;
        }

        match self.indexing_provider() {
            LLMConfiguration::OpenAI { .. } => num_cpus::get() * 4,
            LLMConfiguration::AzureOpenAI { .. } => num_cpus::get() * 4,
            LLMConfiguration::OpenRouter { .. } => num_cpus::get() * 4,
            LLMConfiguration::Ollama { .. } => num_cpus::get(),
            LLMConfiguration::FastEmbed { .. } => num_cpus::get(),
            LLMConfiguration::Anthropic { .. } => num_cpus::get() * 4,
        }
    }

    #[must_use]
    pub fn indexing_batch_size(&self) -> usize {
        if let Some(batch_size) = self.indexing_batch_size {
            return batch_size;
        }

        match self.indexing_provider() {
            LLMConfiguration::OpenAI { .. } => 12,
            LLMConfiguration::AzureOpenAI { .. } => 12,
            LLMConfiguration::Ollama { .. } => 256,
            LLMConfiguration::OpenRouter { .. } => 12,
            LLMConfiguration::FastEmbed { .. } => 256,
            LLMConfiguration::Anthropic { .. } => 12,
        }
    }

    /// Tools enabled by the user
    #[must_use]
    pub fn enabled_tools(&self) -> Vec<&str> {
        self.tools
            .iter()
            .filter(|(_, enabled)| **enabled)
            .map(|(key, _)| key.as_str())
            .collect()
    }

    /// Tools disabled by the user
    #[must_use]
    pub fn disabled_tools(&self) -> Vec<&str> {
        self.tools
            .iter()
            .filter(|(_, enabled)| !*enabled)
            .map(|(key, _)| key.as_str())
            .collect()
    }
}

fn fill_llm(llm: &mut LLMConfiguration, root_key: Option<&ApiKey>) -> Result<()> {
    match llm {
        LLMConfiguration::OpenAI { api_key, .. } => {
            // If the user omitted api_key in the config,
            // fill from the root-level openai_api_key if present.
            if api_key.is_none() {
                if let Some(root) = root_key {
                    *api_key = Some(root.clone());
                } else {
                    anyhow::bail!(
                        "OpenAI config requires an `api_key`, and none was provided or available in the root"
                    );
                }
            }
        }
        LLMConfiguration::AzureOpenAI { api_key, .. } => {
            if api_key.is_none() {
                if let Some(root) = root_key {
                    *api_key = Some(root.clone());
                } else {
                    anyhow::bail!(
                        "AzureOpenAI config requires an `api_key`, and none was provided or available in the root"
                    );
                }
            }
        }
        LLMConfiguration::Anthropic { api_key, .. } => {
            if api_key.is_none() {
                if let Some(root) = root_key {
                    *api_key = Some(root.clone());
                } else {
                    anyhow::bail!(
                        "Anthropic config requires an `api_key`, and none was provided or available in the root"
                    );
                }
            }
        }
        LLMConfiguration::OpenRouter { api_key, .. } => {
            if api_key.is_none() {
                if let Some(root) = root_key {
                    *api_key = Some(root.clone());
                } else {
                    anyhow::bail!(
                        "OpenRouter config requires an `api_key`, and none was provided or available in the root"
                    );
                }
            }
            // Nothing to do for OpenRouter
        }
        LLMConfiguration::Ollama { .. } | LLMConfiguration::FastEmbed { .. } => {
            // Nothing to do for Ollama / FastEmbed
        }
    }
    Ok(())
}
