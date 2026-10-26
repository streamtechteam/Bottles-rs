use bottles_core::config::{RegistryRule, DEFAULT_INHERITED_ENVIRONMENT};
use bottles_core::error::{BottlesError, Result};
use bottles_core::health::HealthChecker;
use bottles_core::journal::{JournalPeriod, JournalSeverity};
use bottles_core::manager::{bottle_prefix_path, NewBottleOptions};
use bottles_core::parity;
use bottles_core::registry_rules::RegistryRuleManager;
use bottles_core::settings::AppSettings;
use bottles_core::umu::{UmuGame, UmuRepository, DEFAULT_PROTON_VALUE};
use bottles_core::wine::{serialize_arguments, split_arguments, WineLauncher, WineTool};
use bottles_core::{AppContext, Bottle, Paths, APP_VERSION};
use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use serde::Serialize;
use serde_json::{json, Value as JsonValue};
use serde_yaml::Value as YamlValue;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

#[derive(Debug, Parser)]
#[command(
    name = "bottles-cli",
    about = "Bottles is a tool to manage your bottles",
    version = APP_VERSION,
    disable_version_flag = true
)]
struct Cli {
    /// Output machine-readable JSON where the Python CLI supports it.
    #[arg(short = 'j', long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Show information about Bottles.
    Info {
        #[arg(value_enum)]
        r#type: InfoType,
    },
    /// List bottles or components.
    List {
        #[arg(value_enum)]
        r#type: ListType,
        #[arg(short = 'f', long)]
        filter: Option<String>,
    },
    /// List programs in a bottle.
    Programs {
        #[arg(short = 'b', long)]
        bottle: String,
    },
    /// Add a program to a bottle.
    Add {
        #[arg(short = 'b', long)]
        bottle: String,
        #[arg(short = 'n', long)]
        name: String,
        #[arg(short = 'p', long)]
        path: String,
        #[arg(short = 'l', long)]
        launch_options: Option<String>,
        #[arg(long)]
        no_d7vk: bool,
        #[arg(long)]
        no_dxvk: bool,
        #[arg(long)]
        no_vkd3d: bool,
        #[arg(long)]
        no_dxvk_nvapi: bool,
    },
    /// Launch a Wine tool.
    Tools {
        #[arg(value_enum)]
        tool: ToolArg,
        #[arg(short = 'b', long)]
        bottle: String,
    },
    /// Manage a bottle registry.
    Reg {
        #[arg(value_enum)]
        action: RegAction,
        #[arg(short = 'b', long)]
        bottle: String,
        #[arg(short = 'k', long)]
        key: String,
        #[arg(short = 'v', long)]
        value: String,
        #[arg(short = 'd', long)]
        data: Option<String>,
        #[arg(short = 't', long = "key-type", value_enum)]
        key_type: Option<RegKeyType>,
    },
    /// Manage reusable registry rules.
    #[command(name = "reg-rules")]
    RegRules {
        #[arg(value_enum)]
        action: RegRulesAction,
        #[arg(short = 'b', long)]
        bottle: Option<String>,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        bundle: Option<PathBuf>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        triggers: Option<String>,
        #[arg(long)]
        run_once: bool,
        #[arg(long)]
        trigger: Option<String>,
        #[arg(long, num_args = 0..)]
        rules: Vec<String>,
    },
    /// Edit a bottle configuration.
    Edit {
        #[arg(short = 'b', long)]
        bottle: String,
        #[arg(long)]
        params: Option<String>,
        #[arg(long = "env-var")]
        env_var: Option<String>,
        #[arg(long)]
        win: Option<String>,
        #[arg(long)]
        runner: Option<String>,
        #[arg(long)]
        d7vk: Option<String>,
        #[arg(long)]
        dxvk: Option<String>,
        #[arg(long)]
        vkd3d: Option<String>,
        #[arg(long)]
        nvapi: Option<String>,
        #[arg(long)]
        latencyflex: Option<String>,
    },
    /// Create a new bottle.
    New {
        #[arg(long = "bottle-name")]
        bottle_name: String,
        #[arg(long)]
        environment: String,
        #[arg(long = "custom-environment")]
        custom_environment: Option<PathBuf>,
        #[arg(long)]
        arch: Option<String>,
        #[arg(long)]
        runner: Option<String>,
        #[arg(long)]
        d7vk: Option<String>,
        #[arg(long)]
        dxvk: Option<String>,
        #[arg(long)]
        vkd3d: Option<String>,
        #[arg(long)]
        nvapi: Option<String>,
        #[arg(long)]
        latencyflex: Option<String>,
    },
    /// Run a program.
    Run {
        #[arg(short = 'b', long)]
        bottle: String,
        #[arg(short = 'e', long)]
        executable: Option<String>,
        #[arg(short = 'p', long)]
        program: Option<String>,
        #[arg(long = "program-id")]
        program_id: Option<String>,
        #[arg(long = "args-replace", default_value_t = true, action = ArgAction::SetFalse)]
        keep_args: bool,
        #[arg(num_args = 0.., trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run programs configured to start at login.
    Autostart,
    /// Stop all processes running in a bottle.
    Stop {
        #[arg(short = 'b', long)]
        bottle: String,
    },
    /// Generate a standalone launcher script.
    Standalone {
        #[arg(short = 'b', long)]
        bottle: String,
    },
    /// Launch commands in a Wine shell.
    Shell {
        #[arg(short = 'b', long)]
        bottle: String,
        #[arg(short = 'i', long)]
        input: String,
    },
    /// Manage UMU games.
    Umu {
        #[arg(value_enum)]
        action: UmuAction,
        #[arg(long)]
        game: Option<String>,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        executable: Option<String>,
        #[arg(long = "game-id", default_value = "umu-default")]
        game_id: String,
        #[arg(long, value_enum, default_value = "none")]
        store: StoreArg,
        #[arg(long)]
        proton: Option<String>,
        #[arg(long)]
        sandbox: bool,
        #[arg(long = "share-network")]
        share_network: bool,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum InfoType {
    #[value(name = "bottles-path")]
    BottlesPath,
    #[value(name = "health-check")]
    HealthCheck,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ListType {
    Bottles,
    Components,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ToolArg {
    Cmd,
    Winecfg,
    Uninstaller,
    Regedit,
    Taskmgr,
    Control,
    Explorer,
}

impl ToolArg {
    fn to_wine_tool(self) -> WineTool {
        WineTool::from_name(match self {
            Self::Cmd => "cmd",
            Self::Winecfg => "winecfg",
            Self::Uninstaller => "uninstaller",
            Self::Regedit => "regedit",
            Self::Taskmgr => "taskmgr",
            Self::Control => "control",
            Self::Explorer => "explorer",
        })
        .expect("tool names are derived from WineTool")
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum RegAction {
    Add,
    Edit,
    #[value(name = "del")]
    Del,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum RegKeyType {
    #[value(name = "REG_DWORD")]
    RegDword,
    #[value(name = "REG_SZ")]
    RegSz,
    #[value(name = "REG_BINARY")]
    RegBinary,
    #[value(name = "REG_MULTI_SZ")]
    RegMultiSz,
}

impl RegKeyType {
    fn as_str(self) -> &'static str {
        match self {
            Self::RegDword => "REG_DWORD",
            Self::RegSz => "REG_SZ",
            Self::RegBinary => "REG_BINARY",
            Self::RegMultiSz => "REG_MULTI_SZ",
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum RegRulesAction {
    List,
    Apply,
    Set,
    Delete,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum UmuAction {
    List,
    Status,
    Add,
    Install,
    Run,
    #[value(name = "set-executable")]
    SetExecutable,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum StoreArg {
    #[value(name = "none")]
    None,
    #[value(name = "amazon")]
    Amazon,
    #[value(name = "battlenet")]
    Battlenet,
    #[value(name = "ea")]
    Ea,
    #[value(name = "egs")]
    Egs,
    #[value(name = "gog")]
    Gog,
    #[value(name = "humble")]
    Humble,
    #[value(name = "itchio")]
    Itchio,
    #[value(name = "steam")]
    Steam,
    #[value(name = "ubisoft")]
    Ubisoft,
    #[value(name = "umu")]
    Umu,
    #[value(name = "zoomplatform")]
    Zoomplatform,
}

impl StoreArg {
    fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Amazon => "amazon",
            Self::Battlenet => "battlenet",
            Self::Ea => "ea",
            Self::Egs => "egs",
            Self::Gog => "gog",
            Self::Humble => "humble",
            Self::Itchio => "itchio",
            Self::Steam => "steam",
            Self::Ubisoft => "ubisoft",
            Self::Umu => "umu",
            Self::Zoomplatform => "zoomplatform",
        }
    }
}

fn main() -> ExitCode {
    env_logger::init();
    let raw: Vec<String> = std::env::args().collect();
    if raw.len() == 2 && (raw[1] == "-v" || raw[1] == "--version") {
        println!("Bottles {APP_VERSION}");
        return ExitCode::SUCCESS;
    }
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<()> {
    let mut context = AppContext::load()?;
    let json = cli.json;
    match cli.command {
        Commands::Info { r#type } => info(&context, r#type, json),
        Commands::List { r#type, filter } => list(&context, r#type, filter.as_deref(), json),
        Commands::Programs { bottle } => programs(&context, &bottle, json),
        Commands::Add {
            bottle,
            name,
            path,
            launch_options,
            no_d7vk,
            no_dxvk,
            no_vkd3d,
            no_dxvk_nvapi,
        } => {
            context.manager.add_program(
                &bottle,
                &name,
                &path,
                launch_options.as_deref(),
                no_d7vk,
                no_dxvk,
                no_vkd3d,
                no_dxvk_nvapi,
            )?;
            println!("'{name}' added to '{bottle}'!");
            Ok(())
        }
        Commands::Tools { tool, bottle } => launch_tool(&context, &bottle, tool.to_wine_tool()),
        Commands::Reg {
            action,
            bottle,
            key,
            value,
            data,
            key_type,
        } => manage_reg(
            &context,
            &bottle,
            action,
            &key,
            &value,
            data.as_deref(),
            key_type,
        ),
        Commands::RegRules {
            action,
            bottle,
            name,
            bundle,
            description,
            triggers,
            run_once,
            trigger,
            rules,
        } => manage_reg_rules(
            &mut context,
            action,
            bottle.as_deref(),
            name.as_deref(),
            bundle.as_deref(),
            description.as_deref(),
            triggers.as_deref(),
            run_once,
            trigger.as_deref(),
            &rules,
            json,
        ),
        Commands::Edit {
            bottle,
            params,
            env_var,
            win,
            runner,
            d7vk,
            dxvk,
            vkd3d,
            nvapi,
            latencyflex,
        } => edit_bottle(
            &context,
            &bottle,
            params.as_deref(),
            env_var.as_deref(),
            win.as_deref(),
            runner.as_deref(),
            d7vk.as_deref(),
            dxvk.as_deref(),
            vkd3d.as_deref(),
            nvapi.as_deref(),
            latencyflex.as_deref(),
        ),
        Commands::New {
            bottle_name,
            environment,
            custom_environment,
            arch,
            runner,
            d7vk,
            dxvk,
            vkd3d,
            nvapi,
            latencyflex,
        } => {
            let options = NewBottleOptions {
                name: bottle_name,
                environment,
                custom_environment,
                arch: arch.unwrap_or_else(|| "win64".to_owned()),
                runner,
                d7vk,
                dxvk,
                vkd3d,
                nvapi,
                latencyflex,
                ..NewBottleOptions::default()
            };
            let bottle = context.manager.create_bottle(options)?;
            if json {
                print_json(&bottle.config)?;
            } else {
                println!(
                    "Bottle '{}' created at {}",
                    bottle.config.name,
                    bottle.prefix.display()
                );
            }
            Ok(())
        }
        Commands::Run {
            bottle,
            executable,
            program,
            program_id,
            keep_args,
            args,
        } => run_program(
            &context,
            &bottle,
            executable.as_deref(),
            program.as_deref(),
            program_id.as_deref(),
            keep_args,
            &args,
        ),
        Commands::Autostart => autostart(&context),
        Commands::Stop { bottle } => stop_bottle(&context, &bottle),
        Commands::Standalone { bottle } => generate_standalone(&context, &bottle),
        Commands::Shell { bottle, input } => run_shell(&context, &bottle, &input),
        Commands::Umu {
            action,
            game,
            name,
            executable,
            game_id,
            store,
            proton,
            sandbox,
            share_network,
        } => manage_umu(
            &mut context,
            action,
            game.as_deref(),
            name.as_deref(),
            executable.as_deref(),
            &game_id,
            store.as_str(),
            proton.as_deref(),
            sandbox,
            share_network,
            json,
        ),
    }
}

fn info(context: &AppContext, kind: InfoType, json: bool) -> Result<()> {
    match kind {
        InfoType::BottlesPath => {
            // The Python CLI intentionally prints a bare path here.
            println!("{}", context.paths.bottles.display());
            Ok(())
        }
        InfoType::HealthCheck => {
            let report = HealthChecker::collect();
            if json {
                print_json(&report.to_json())?;
            } else {
                print!("{}", report.to_yaml()?);
            }
            Ok(())
        }
    }
}

fn list(context: &AppContext, kind: ListType, filter: Option<&str>, json: bool) -> Result<()> {
    match kind {
        ListType::Bottles => {
            let bottles = context.manager.list_bottles()?;
            if let Some(filter) = filter.and_then(|value| value.strip_prefix("environment:")) {
                let names: Vec<&str> = bottles
                    .iter()
                    .filter(|bottle| bottle.config.environment.eq_ignore_ascii_case(filter))
                    .map(|bottle| bottle.name())
                    .collect();
                if json {
                    print_json(&names)?;
                } else {
                    println!("Found {} bottles:", names.len());
                    for name in names {
                        println!("- {name}");
                    }
                }
            } else {
                let values: BTreeMap<String, _> = bottles
                    .into_iter()
                    .map(|bottle| (bottle.config.name.clone(), bottle.config))
                    .collect();
                if json {
                    print_json(&values)?;
                } else {
                    println!("Found {} bottles:", values.len());
                    for name in values.keys() {
                        println!("- {name}");
                    }
                }
            }
            Ok(())
        }
        ListType::Components => {
            let mut components = context.manager.components();
            if let Some(category) = filter.and_then(|value| value.strip_prefix("category:")) {
                if let Some(selected) = components.get(category).cloned() {
                    components = BTreeMap::from([(category.to_owned(), selected)]);
                }
            }
            if json {
                print_json(&components)?;
            } else {
                for (name, values) in components {
                    println!("Found {} {name}", values.len());
                    for value in values {
                        println!("- {value}");
                    }
                }
            }
            Ok(())
        }
    }
}

fn programs(context: &AppContext, bottle_name: &str, json: bool) -> Result<()> {
    let bottle = context.manager.get_bottle(bottle_name)?;
    let programs = bottle.config.programs();
    if json {
        print_json(&programs)?;
    } else {
        println!("Found {} programs:", programs.len());
        for program in programs {
            println!("- {}", program.display_name());
        }
    }
    Ok(())
}

fn launch_tool(context: &AppContext, bottle_name: &str, tool: WineTool) -> Result<()> {
    let bottle = context.manager.get_bottle(bottle_name)?;
    let launcher = launcher(context, &bottle);
    let plan = launcher.tool_plan(tool, &[])?;
    launcher.spawn(&plan)?;
    Ok(())
}

fn manage_reg(
    context: &AppContext,
    bottle_name: &str,
    action: RegAction,
    key: &str,
    value: &str,
    data: Option<&str>,
    key_type: Option<RegKeyType>,
) -> Result<()> {
    let bottle = context.manager.get_bottle(bottle_name)?;
    let launcher = launcher(context, &bottle);
    let key_type = key_type.unwrap_or(RegKeyType::RegSz);
    let arguments = match action {
        RegAction::Add | RegAction::Edit => {
            let data = data.ok_or_else(|| {
                BottlesError::InvalidInput("Missing or invalid data or key type".to_owned())
            })?;
            vec![
                "add".to_owned(),
                key.to_owned(),
                "/v".to_owned(),
                value.to_owned(),
                "/t".to_owned(),
                key_type.as_str().to_owned(),
                "/d".to_owned(),
                data.to_owned(),
                "/f".to_owned(),
            ]
        }
        RegAction::Del => vec![
            "delete".to_owned(),
            key.to_owned(),
            "/v".to_owned(),
            value.to_owned(),
            "/f".to_owned(),
        ],
    };
    let plan = launcher.tool_plan(WineTool::Reg, &arguments)?;
    let output = launcher.execute(&plan)?;
    if !output.status.success() {
        return Err(BottlesError::CommandFailed {
            command: "reg".to_owned(),
            message: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        });
    }
    print!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn manage_reg_rules(
    context: &mut AppContext,
    action: RegRulesAction,
    bottle_name: Option<&str>,
    name: Option<&str>,
    bundle: Option<&Path>,
    description: Option<&str>,
    triggers: Option<&str>,
    run_once: bool,
    trigger: Option<&str>,
    rules: &[String],
    json: bool,
) -> Result<()> {
    let bottle_name = bottle_name.ok_or_else(|| {
        BottlesError::InvalidInput("--bottle is required for this action".to_owned())
    })?;
    let mut bottle = context.manager.get_bottle(bottle_name)?;
    match action {
        RegRulesAction::List => {
            let rules = RegistryRuleManager::list_rules(&bottle.config);
            if json {
                print_json(&rules)?;
            } else {
                for rule in rules {
                    let triggers = if rule.triggers.is_empty() {
                        "-".to_owned()
                    } else {
                        rule.triggers.join(", ")
                    };
                    println!(
                        "{}: {} (triggers: {})",
                        rule.name, rule.description, triggers
                    );
                }
            }
            Ok(())
        }
        RegRulesAction::Set => {
            let name = name.ok_or_else(|| {
                BottlesError::InvalidInput("--name and --bundle are required".to_owned())
            })?;
            let bundle = bundle.ok_or_else(|| {
                BottlesError::InvalidInput("--name and --bundle are required".to_owned())
            })?;
            let keys = fs::read_to_string(bundle).map_err(|source| BottlesError::Io {
                path: bundle.to_path_buf(),
                source,
            })?;
            let triggers = triggers
                .unwrap_or_default()
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
                .collect();
            RegistryRuleManager::upsert_rule(
                &mut bottle.config,
                RegistryRule {
                    name: name.to_owned(),
                    description: description.unwrap_or_default().to_owned(),
                    keys,
                    triggers,
                    run_once,
                },
            );
            context.manager.save_config(&bottle)
        }
        RegRulesAction::Delete => {
            let name =
                name.ok_or_else(|| BottlesError::InvalidInput("--name is required".to_owned()))?;
            if RegistryRuleManager::delete_rule(&mut bottle.config, name) {
                context.manager.save_config(&bottle)?;
            }
            Ok(())
        }
        RegRulesAction::Apply => {
            let applied = RegistryRuleManager::apply_rules(
                &context.paths,
                &bottle.config,
                &bottle.prefix,
                (!rules.is_empty()).then_some(rules),
                trigger,
            )?;
            if json {
                print_json(&applied)?;
            }
            Ok(())
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn edit_bottle(
    context: &AppContext,
    bottle_name: &str,
    params: Option<&str>,
    env_var: Option<&str>,
    windows: Option<&str>,
    runner: Option<&str>,
    d7vk: Option<&str>,
    dxvk: Option<&str>,
    vkd3d: Option<&str>,
    nvapi: Option<&str>,
    latencyflex: Option<&str>,
) -> Result<()> {
    let mut bottle = context.manager.get_bottle(bottle_name)?;
    let mut changed = false;
    if let Some(params) = params {
        for pair in params.split(',') {
            let (key, value) = pair.split_once(':').ok_or_else(|| {
                BottlesError::InvalidInput(format!("Invalid parameter assignment: {pair}"))
            })?;
            if key == "d7vk" {
                return Err(BottlesError::InvalidInput(
                    "Use --d7vk to change D7VK".to_owned(),
                ));
            }
            if key == "renderer" {
                set_renderer(context, &bottle, value)?;
            }
            if key == "sync" {
                restart_wineserver(context, &bottle)?;
            }
            bottle.config.set_parameter_from_text(key, value)?;
            changed = true;
        }
    }
    if let Some(env_var) = env_var {
        let (key, value) = env_var
            .split_once('=')
            .ok_or_else(|| BottlesError::InvalidInput("--env-var requires KEY=VALUE".to_owned()))?;
        bottle.config.set_environment_variable(key, value)?;
        changed = true;
    }
    if let Some(windows) = windows {
        set_windows_version(context, &mut bottle, windows)?;
        changed = true;
    }
    if let Some(runner) = runner {
        runner_update(context, &mut bottle, runner)?;
        changed = true;
    }
    if let Some(version) = d7vk {
        set_d7vk(context, &mut bottle, version)?;
        changed = true;
    }
    if let Some(version) = dxvk {
        set_component_version(context, &mut bottle, "dxvk", version)?;
        changed = true;
    }
    if let Some(version) = vkd3d {
        set_component_version(context, &mut bottle, "vkd3d", version)?;
        changed = true;
    }
    if let Some(version) = nvapi {
        set_component_version(context, &mut bottle, "nvapi", version)?;
        changed = true;
    }
    if let Some(version) = latencyflex {
        set_component_version(context, &mut bottle, "latencyflex", version)?;
        changed = true;
    }
    if changed {
        context.manager.save_config(&bottle)?;
        let _ = RegistryRuleManager::apply_rules(
            &context.paths,
            &bottle.config,
            &bottle.prefix,
            None,
            Some("components"),
        );
    }
    Ok(())
}

fn run_program(
    context: &AppContext,
    bottle_name: &str,
    executable: Option<&str>,
    program: Option<&str>,
    program_id: Option<&str>,
    keep_args: bool,
    args: &[String],
) -> Result<()> {
    let bottle_name = trim_matching_quotes(bottle_name);
    let bottle = context.manager.get_bottle(bottle_name)?;
    let launcher = launcher(context, &bottle);
    let serialized_args = serialize_arguments(args);
    let extra_args = split_arguments(&serialized_args);
    if program.is_some() || program_id.is_some() {
        if executable.is_some() {
            return Err(BottlesError::InvalidInput(
                "Cannot specify --executable with --program or --program-id".to_owned(),
            ));
        }
        if program.is_some() && program_id.is_some() {
            return Err(BottlesError::InvalidInput(
                "Cannot specify both --program and --program-id".to_owned(),
            ));
        }
        let identifier = program.or(program_id).unwrap_or_default();
        let found = bottle
            .config
            .external_programs
            .values()
            .find(|candidate| {
                program_id
                    .map(|id| candidate.id.as_deref() == Some(id))
                    .unwrap_or_else(|| candidate.name.as_deref() == Some(identifier))
            })
            .cloned()
            .ok_or_else(|| BottlesError::ProgramNotFound(identifier.to_owned()))?;
        let mut program = found;
        if !keep_args {
            program.arguments = None;
        }
        let plan = launcher.program_plan(&program, &extra_args)?;
        launcher.spawn(&plan)?;
        return Ok(());
    }
    if let Some(executable) = executable {
        let executable = trim_matching_quotes(executable.trim_start_matches("file://"));
        let plan = launcher.plan(executable, &extra_args, None)?;
        launcher.spawn(&plan)?;
        return Ok(());
    }
    Err(BottlesError::InvalidInput(
        "No program or executable specified, you must use either --program or --executable"
            .to_owned(),
    ))
}

fn autostart(context: &AppContext) -> Result<()> {
    let current = std::env::current_exe().map_err(|source| BottlesError::Io {
        path: PathBuf::from("current_exe"),
        source,
    })?;
    for (bottle, program) in context.manager.autostart_programs()? {
        let Some(id) = program.id else { continue };
        let _ = Command::new(&current)
            .args(["run", "-b", &bottle.config.name, "--program-id", &id])
            .spawn();
    }
    Ok(())
}

fn stop_bottle(context: &AppContext, bottle_name: &str) -> Result<()> {
    let bottle = context
        .manager
        .get_bottle(trim_matching_quotes(bottle_name))?;
    let launcher = launcher(context, &bottle);
    let kill = launcher.tool_plan(WineTool::Wineserver, &["-k".to_owned()])?;
    launcher.execute(&kill)?;
    let wait = launcher.tool_plan(WineTool::Wineserver, &["-w".to_owned()])?;
    launcher.execute(&wait)?;
    println!("Stopped all processes in bottle {bottle_name}");
    Ok(())
}

fn generate_standalone(context: &AppContext, bottle_name: &str) -> Result<()> {
    let bottle = context.manager.get_bottle(bottle_name)?;
    let launcher = launcher(context, &bottle);
    let script = launcher.standalone_script()?;
    let path = bottle.prefix.join("standalone");
    if path.is_file() {
        fs::remove_file(&path).map_err(|source| BottlesError::Io {
            path: path.clone(),
            source,
        })?;
    }
    fs::write(&path, script).map_err(|source| BottlesError::Io {
        path: path.clone(),
        source,
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).map_err(|source| {
            BottlesError::Io {
                path: path.clone(),
                source,
            }
        })?;
    }
    println!("Standalone generated in {}", bottle.prefix.display());
    println!("Re-generate after every bottle change.");
    Ok(())
}

fn run_shell(context: &AppContext, bottle_name: &str, input: &str) -> Result<()> {
    let bottle = context.manager.get_bottle(bottle_name)?;
    let launcher = launcher(context, &bottle);
    let mut args = vec!["/c".to_owned()];
    args.extend(split_arguments(input));
    let plan = launcher.tool_plan(WineTool::Cmd, &args)?;
    let output = launcher.execute(&plan)?;
    print!("{}", String::from_utf8_lossy(&output.stdout));
    if !output.status.success() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn manage_umu(
    context: &mut AppContext,
    action: UmuAction,
    game: Option<&str>,
    name: Option<&str>,
    executable: Option<&str>,
    game_id: &str,
    store: &str,
    proton: Option<&str>,
    sandbox: bool,
    share_network: bool,
    json: bool,
) -> Result<()> {
    let repository = context.umu_repository();
    if action_variant_eq(action, UmuAction::List) {
        let games = repository.list_games()?;
        if json {
            print_json(&games)?;
        } else {
            for game in games {
                println!("{}  {:10}  {}", game.id, game.state, game.name);
            }
        }
        return Ok(());
    }

    let mut owned_game: Option<UmuGame> = None;
    if matches!(action, UmuAction::Add | UmuAction::Install) {
        let name = name.ok_or_else(|| {
            BottlesError::InvalidInput("--name and --executable are required".to_owned())
        })?;
        let executable = absolute_existing_path(executable.ok_or_else(|| {
            BottlesError::InvalidInput("--name and --executable are required".to_owned())
        })?)?;
        let settings = AppSettings::load();
        let proton = proton
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| {
                if settings.umu_proton.is_empty() {
                    DEFAULT_PROTON_VALUE.to_owned()
                } else {
                    settings.umu_proton
                }
            });
        let mut new_game = repository.new_game(
            name,
            executable.clone(),
            proton,
            game_id.to_owned(),
            store.to_owned(),
            sandbox,
            share_network,
        );
        new_game.extra.insert(
            "dependency_tool".to_owned(),
            YamlValue::String(settings.umu_dependency_tool),
        );
        new_game.extra.insert(
            "source_mode".to_owned(),
            YamlValue::String(
                if action_variant_eq(action, UmuAction::Install) {
                    "installer"
                } else {
                    "portable"
                }
                .to_owned(),
            ),
        );
        new_game.state = if action_variant_eq(action, UmuAction::Install) {
            "installing"
        } else {
            "ready"
        }
        .to_owned();
        if action_variant_eq(action, UmuAction::Install) {
            new_game.extra.insert(
                "installer".to_owned(),
                YamlValue::String(executable.to_string_lossy().into_owned()),
            );
        }
        repository.save(&new_game)?;
        println!("{}", new_game.id);
        context.library.sync_umu_game(
            &new_game.id.to_string(),
            &new_game.id.to_string(),
            &new_game.name,
            &new_game.store,
        )?;
        if action_variant_eq(action, UmuAction::Add) {
            return Ok(());
        }
        owned_game = Some(new_game);
    }

    let game = match owned_game {
        Some(game) => game,
        None => find_umu_game(&repository, game)?,
    };

    if action_variant_eq(action, UmuAction::Status) {
        let status = json!({
            "id": game.id.to_string(),
            "name": game.name,
            "state": game.state,
            "running": false,
            "prefix": repository.prefix_path(&game)?.display().to_string(),
            "executable": game.executable.display().to_string(),
        });
        if json {
            print_json(&status)?;
        } else {
            for (key, value) in status.as_object().unwrap() {
                println!(
                    "{key}: {}",
                    match value {
                        JsonValue::String(value) => value.clone(),
                        other => other.to_string(),
                    }
                );
            }
        }
        return Ok(());
    }

    if action_variant_eq(action, UmuAction::SetExecutable) {
        let executable =
            absolute_existing_path(executable.ok_or_else(|| {
                BottlesError::InvalidInput("--executable is required".to_owned())
            })?)?;
        let mut game = game;
        game.executable = executable;
        game.state = "ready".to_owned();
        repository.save(&game)?;
        return Ok(());
    }

    if action_variant_eq(action, UmuAction::Run) && game.state != "ready" {
        return Err(BottlesError::InvalidInput(
            "The UMU game is not ready to run".to_owned(),
        ));
    }
    let install_only = action_variant_eq(action, UmuAction::Install);
    let launcher_override = context.settings.umu_launcher_path.clone();
    let mut command = repository.launch_command(
        &game,
        (!launcher_override.is_empty()).then_some(launcher_override.as_str()),
        install_only,
    )?;
    let status = command.status().map_err(|source| BottlesError::Io {
        path: PathBuf::from("umu-run"),
        source,
    })?;
    if install_only {
        let state = if status.success() { "draft" } else { "failed" };
        let updated = repository.set_state(game.id, state)?;
        println!("state: {}", updated.state);
    }
    if !status.success() {
        return Err(BottlesError::CommandFailed {
            command: "umu-run".to_owned(),
            message: format!("exit status {}", status),
        });
    }
    Ok(())
}

fn action_variant_eq(left: UmuAction, right: UmuAction) -> bool {
    std::mem::discriminant(&left) == std::mem::discriminant(&right)
}

fn find_umu_game(repository: &UmuRepository, query: Option<&str>) -> Result<UmuGame> {
    let query = query.ok_or_else(|| {
        BottlesError::InvalidInput("--game is required for this UMU action".to_owned())
    })?;
    let mut matches = repository.find(query)?;
    if matches.len() != 1 {
        return Err(BottlesError::InvalidInput(format!(
            "UMU game not found or ambiguous: {query}"
        )));
    }
    Ok(matches.remove(0))
}

fn absolute_existing_path(value: &str) -> Result<PathBuf> {
    let expanded = if let Some(rest) = value.strip_prefix("~/") {
        Paths::detect()
            .xdg_data_home
            .parent()
            .map(|home| home.join(rest))
            .unwrap_or_else(|| PathBuf::from(value))
    } else {
        PathBuf::from(value)
    };
    let path = if expanded.is_absolute() {
        expanded
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("/"))
            .join(expanded)
    };
    if !path.is_file() {
        return Err(BottlesError::InvalidInput(format!(
            "Executable not found: {}",
            path.display()
        )));
    }
    Ok(path)
}

fn launcher(context: &AppContext, bottle: &Bottle) -> WineLauncher {
    WineLauncher::new(
        context.paths.clone(),
        bottle.config.clone(),
        bottle.prefix.clone(),
    )
}

fn restart_wineserver(context: &AppContext, bottle: &Bottle) -> Result<()> {
    let launcher = launcher(context, bottle);
    let kill = launcher.tool_plan(WineTool::Wineserver, &["-k".to_owned()])?;
    launcher.execute(&kill)?;
    let wait = launcher.tool_plan(WineTool::Wineserver, &["-w".to_owned()])?;
    launcher.execute(&wait)?;
    Ok(())
}

fn set_renderer(context: &AppContext, bottle: &Bottle, value: &str) -> Result<()> {
    if !["gl", "gdi", "vulkan"].contains(&value) {
        return Err(BottlesError::InvalidInput(format!(
            "{value} is not a valid renderer (gl, gdi, vulkan)"
        )));
    }
    let launcher = launcher(context, bottle);
    let args = vec![
        "add".to_owned(),
        r"HKEY_CURRENT_USER\Software\Wine\Direct3D".to_owned(),
        "/v".to_owned(),
        "renderer".to_owned(),
        "/t".to_owned(),
        "REG_SZ".to_owned(),
        "/d".to_owned(),
        value.to_owned(),
        "/f".to_owned(),
    ];
    let plan = launcher.tool_plan(WineTool::Reg, &args)?;
    launcher.execute(&plan)?;
    Ok(())
}

fn set_windows_version(context: &AppContext, bottle: &mut Bottle, version: &str) -> Result<()> {
    let launcher = launcher(context, bottle);
    let plan = launcher.tool_plan(WineTool::WineCfg, &["-v".to_owned(), version.to_owned()])?;
    launcher.execute(&plan)?;
    bottle.config.windows = version.to_owned();
    Ok(())
}

fn runner_update(context: &AppContext, bottle: &mut Bottle, runner: &str) -> Result<()> {
    if !runner.starts_with("sys-") && !context.paths.runners.join(runner).exists() {
        return Err(BottlesError::InvalidInput(format!(
            "Runner {runner} not found"
        )));
    }
    restart_wineserver(context, bottle)?;
    bottle.config.runner = runner.to_owned();
    context.manager.save_config(bottle)?;
    let launcher = launcher(context, bottle);
    let update = launcher.tool_plan(WineTool::Wineboot, &["-u".to_owned()])?;
    launcher.spawn(&update)?;
    let _ = RegistryRuleManager::apply_rules(
        &context.paths,
        &bottle.config,
        &bottle.prefix,
        None,
        Some("runner_change"),
    );
    Ok(())
}

fn set_d7vk(context: &AppContext, bottle: &mut Bottle, version: &str) -> Result<()> {
    if version.eq_ignore_ascii_case("disabled") {
        bottle.config.parameters.d7vk = false;
        bottle.config.d7vk.clear();
    } else {
        if !context.paths.d7vk.join(version).exists() {
            return Err(BottlesError::InvalidInput(format!(
                "D7VK version {version} not available"
            )));
        }
        bottle.config.parameters.d7vk = true;
        bottle.config.d7vk = version.to_owned();
    }
    Ok(())
}

fn set_component_version(
    context: &AppContext,
    bottle: &mut Bottle,
    component: &str,
    version: &str,
) -> Result<()> {
    let directory = match component {
        "dxvk" => &context.paths.dxvk,
        "vkd3d" => &context.paths.vkd3d,
        "nvapi" => &context.paths.nvapi,
        "latencyflex" => &context.paths.latencyflex,
        _ => return Err(BottlesError::InvalidInput("unknown component".to_owned())),
    };
    if !directory.join(version).exists() {
        return Err(BottlesError::InvalidInput(format!(
            "{component} version {version} not available"
        )));
    }
    match component {
        "dxvk" => bottle.config.dxvk = version.to_owned(),
        "vkd3d" => bottle.config.vkd3d = version.to_owned(),
        "nvapi" => bottle.config.nvapi = version.to_owned(),
        "latencyflex" => bottle.config.latency_flex = version.to_owned(),
        _ => unreachable!(),
    }
    Ok(())
}

fn trim_matching_quotes(value: &str) -> &str {
    if (value.starts_with('"') && value.ends_with('"'))
        || (value.starts_with('\'') && value.ends_with('\''))
    {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

fn print_json<T: Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_contract_matches_python_command_surface() {
        let command = Cli::command();
        let subcommands: Vec<String> = command
            .get_subcommands()
            .map(|value| value.get_name().to_owned())
            .collect();
        for expected in bottles_core::CLI_COMMANDS {
            assert!(
                subcommands.contains(&expected.to_string()),
                "missing {expected}"
            );
        }
        assert_eq!(subcommands.len(), bottles_core::CLI_COMMANDS.len());
    }

    #[test]
    fn parity_inventory_is_embedded() {
        assert!(parity::inventory().len() > 350);
    }
}
