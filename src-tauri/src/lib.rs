mod agents;
mod error;
mod git;
mod persistence;
mod pty;
mod shell;
mod state;
mod tasks;

use state::AppState;
use tracing_subscriber::EnvFilter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("parallel_code=info")),
        )
        .init();

    // Eagerly resolve login shell PATH to avoid delay on first agent spawn
    std::thread::spawn(|| {
        shell::login_path();
    });

    // Hint WebKitGTK toward hardware-accelerated rendering on Linux.
    // These must be set before GTK/WebKit initialisation (i.e. before
    // tauri::Builder runs).
    #[cfg(target_os = "linux")]
    {
        use std::env;

        // Prefer the GL renderer for the GSK scene graph (GTK 4).
        // "ngl" is the modern GL backend; "gl" is the older one.
        // Either is faster than the default "cairo" software path.
        if env::var_os("GSK_RENDERER").is_none() {
            env::set_var("GSK_RENDERER", "ngl");
        }

        // On some NVIDIA + Wayland setups the DMA-BUF renderer causes
        // blank or corrupt WebKit compositing layers.  Disabling it
        // forces a safer EGL/GLX code-path that still uses the GPU.
        if env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
            env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            pty::spawn_agent,
            pty::write_to_agent,
            pty::resize_agent,
            pty::kill_agent,
            pty::count_running_agents,
            pty::kill_all_agents,
            agents::list_agents,
            tasks::create_task,
            tasks::delete_task,
            git::get_changed_files,
            git::get_file_diff,
            git::get_gitignored_dirs,
            git::get_worktree_status,
            git::merge_task,
            git::get_branch_log,
            git::push_task,
            git::check_merge_status,
            git::rebase_task,
            git::get_main_branch,
            git::get_current_branch,
            persistence::save_app_state,
            persistence::load_app_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
