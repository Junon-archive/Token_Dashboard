use token_dashboard::dashboard::{DashboardRuntime, FrontendSnapshot};
use tokio::sync::Mutex;

type AppDashboardRuntime = DashboardRuntime<
    token_dashboard::http::ReqwestUsageHttpClient,
    token_dashboard::refresh::ReqwestRefreshHttpClient,
    token_dashboard::dashboard::DefaultCredentialSource,
>;

#[tauri::command]
async fn usage_snapshots(
    runtime: tauri::State<'_, Mutex<AppDashboardRuntime>>,
) -> Result<Vec<FrontendSnapshot>, String> {
    Ok(runtime.lock().await.frontend_snapshots().await)
}

fn main() {
    #[cfg(target_os = "linux")]
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");

    tauri::Builder::default()
        .manage(Mutex::new(DashboardRuntime::default()))
        .invoke_handler(tauri::generate_handler![usage_snapshots])
        .run(tauri::generate_context!())
        .expect("failed to run Token Dashboard");
}
