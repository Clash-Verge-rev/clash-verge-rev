use crate::{
    config::*,
    core::*,
    feat,
    utils::{dirs, help},
};
use crate::{log_err, ret_err, wrap_err};
use anyhow::{Context, Result};
use network_interface::NetworkInterface;
use serde_yaml::Mapping;
use std::collections::HashMap;
use sysproxy::{Autoproxy, Sysproxy};
type CmdResult<T = ()> = Result<T, String>;
use reqwest_dav::list_cmd::ListFile;
use tauri::Manager;

#[tauri::command]
pub fn copy_clash_env() -> CmdResult {
    feat::copy_clash_env();
    Ok(())
}

#[tauri::command]
pub fn get_profiles() -> CmdResult<IProfiles> {
    let _ = tray::Tray::global().update_menu();
    Ok(Config::profiles().data().clone())
}

#[tauri::command]
pub async fn enhance_profiles() -> CmdResult {
    wrap_err!(CoreManager::global().update_config().await)?;
    log_err!(tray::Tray::global().update_tooltip());
    handle::Handle::refresh_clash();
    Ok(())
}

#[tauri::command]
pub async fn import_profile(url: String, option: Option<PrfOption>) -> CmdResult {
    let item = wrap_err!(PrfItem::from_url(&url, None, None, option).await)?;
    wrap_err!(Config::profiles().data().append_item(item))
}

#[tauri::command]
pub async fn reorder_profile(active_id: String, over_id: String) -> CmdResult {
    wrap_err!(Config::profiles().data().reorder(active_id, over_id))
}

#[tauri::command]
pub async fn create_profile(item: PrfItem, file_data: Option<String>) -> CmdResult {
    let item = wrap_err!(PrfItem::from(item, file_data).await)?;
    wrap_err!(Config::profiles().data().append_item(item))
}

#[tauri::command]
pub async fn update_profile(index: String, option: Option<PrfOption>) -> CmdResult {
    wrap_err!(feat::update_profile(index, option).await)
}

#[tauri::command]
pub async fn delete_profile(index: String) -> CmdResult {
    let should_update = wrap_err!({ Config::profiles().data().delete_item(index) })?;
    if should_update {
        wrap_err!(CoreManager::global().update_config().await)?;
        handle::Handle::refresh_clash();
    }
    Ok(())
}

/// 修改profiles的
#[tauri::command]
pub async fn patch_profiles_config(profiles: IProfiles) -> CmdResult {
    // 使用 ConfigGuard 来管理配置状态，声明为可变
    let mut config_guard = ConfigGuard::new(Config::profiles().latest().clone());
    
    // 1. 应用新配置
    log::info!(target: "app", "Applying new profile configuration");
    wrap_err!({ Config::profiles().draft().patch_config(profiles) })?;
    
    // 2. 尝试更新核心配置
    match CoreManager::global().update_config().await {
        Ok(_) => {
            log::info!(target: "app", "Core configuration updated successfully");
            
            // 3. 更新系统状态
            handle::Handle::refresh_clash();
            if let Err(e) = tray::Tray::global().update_tooltip() {
                log::warn!(target: "app", "Failed to update tray tooltip: {}", e);
            }
            
            // 4. 保存配置
            Config::profiles().apply();
            wrap_err!(Config::profiles().data().save_file())
                .map_err(|e| {
                    log::error!(target: "app", "Failed to save profile configuration: {}", e);
                    format!("保存配置文件失败: {}", e)
                })?;
            
            // 配置更新成功，丢弃回滚状态
            config_guard.discard();
            Ok(())
        }
        Err(err) => {
            log::error!(target: "app", "Failed to update core configuration: {}", err);
            
            // 5. 回滚配置
            if let Err(e) = config_guard.rollback().await {
                log::error!(target: "app", "Failed to rollback configuration: {}", e);
                return Err(format!("配置更新失败，且无法回滚到之前的配置: {}", err));
            }
            
            Err(format!("配置更新失败: {}", err))
        }
    }
}

/// 根据profile name修改profiles
#[tauri::command]
pub async fn patch_profiles_config_by_profile_index(profile_index: String) -> CmdResult {
    let profiles = IProfiles{current: Some(profile_index), items: None};
    patch_profiles_config(profiles).await
}

/// 修改某个profile item的
#[tauri::command]
pub fn patch_profile(index: String, profile: PrfItem) -> CmdResult {
    wrap_err!(Config::profiles().data().patch_item(index, profile))?;
    wrap_err!(timer::Timer::global().refresh())
}

#[tauri::command]
pub fn view_profile(app_handle: tauri::AppHandle, index: String) -> CmdResult {
    let file = {
        wrap_err!(Config::profiles().latest().get_item(&index))?
            .file
            .clone()
            .ok_or("the file field is null")
    }?;

    let path = wrap_err!(dirs::app_profiles_dir())?.join(file);
    if !path.exists() {
        ret_err!("the file not found");
    }

    wrap_err!(help::open_file(app_handle, path))
}

#[tauri::command]
pub fn read_profile_file(index: String) -> CmdResult<String> {
    let profiles = Config::profiles();
    let profiles = profiles.latest();
    let item = wrap_err!(profiles.get_item(&index))?;
    let data = wrap_err!(item.read_file())?;
    Ok(data)
}

#[tauri::command]
pub fn save_profile_file(index: String, file_data: Option<String>) -> CmdResult {
    if file_data.is_none() {
        return Ok(());
    }

    let profiles = Config::profiles();
    let profiles = profiles.latest();
    let item = wrap_err!(profiles.get_item(&index))?;
    wrap_err!(item.save_file(file_data.unwrap()))
}

#[tauri::command]
pub fn get_clash_info() -> CmdResult<ClashInfo> {
    Ok(Config::clash().latest().get_client_info())
}

#[tauri::command]
pub fn get_runtime_config() -> CmdResult<Option<Mapping>> {
    Ok(Config::runtime().latest().config.clone())
}

#[tauri::command]
pub fn get_runtime_yaml() -> CmdResult<String> {
    let runtime = Config::runtime();
    let runtime = runtime.latest();
    let config = runtime.config.as_ref();
    wrap_err!(config
        .ok_or(anyhow::anyhow!("failed to parse config to yaml file"))
        .and_then(
            |config| serde_yaml::to_string(config).context("failed to convert config to yaml")
        ))
}

#[tauri::command]
pub fn get_runtime_exists() -> CmdResult<Vec<String>> {
    Ok(Config::runtime().latest().exists_keys.clone())
}

#[tauri::command]
pub fn get_runtime_logs() -> CmdResult<HashMap<String, Vec<(String, String)>>> {
    Ok(Config::runtime().latest().chain_logs.clone())
}

#[tauri::command]
pub async fn patch_clash_config(payload: Mapping) -> CmdResult {
    wrap_err!(feat::patch_clash(payload).await)
}

#[tauri::command]
pub async fn patch_clash_mode(payload: String) -> CmdResult {
    Ok(feat::change_clash_mode(payload))
}


#[tauri::command]
pub fn get_verge_config() -> CmdResult<IVergeResponse> {
    let verge = Config::verge();
    let verge_data = verge.data().clone();
    Ok(IVergeResponse::from(verge_data))
}

#[tauri::command]
pub async fn patch_verge_config(payload: IVerge) -> CmdResult {
    wrap_err!(feat::patch_verge(payload).await)
}

#[tauri::command]
pub async fn change_clash_core(clash_core: Option<String>) -> CmdResult {
    wrap_err!(CoreManager::global().change_core(clash_core).await)
}

/// restart the sidecar
#[tauri::command]
pub async fn restart_core() -> CmdResult {
    wrap_err!(CoreManager::global().restart_core().await)
}

/// get the system proxy
#[tauri::command]
pub fn get_sys_proxy() -> CmdResult<Mapping> {
    let current = wrap_err!(Sysproxy::get_system_proxy())?;
    let mut map = Mapping::new();
    map.insert("enable".into(), current.enable.into());
    map.insert(
        "server".into(),
        format!("{}:{}", current.host, current.port).into(),
    );
    map.insert("bypass".into(), current.bypass.into());

    Ok(map)
}

/// get the system proxy
#[tauri::command]
pub fn get_auto_proxy() -> CmdResult<Mapping> {
    let current = wrap_err!(Autoproxy::get_auto_proxy())?;

    let mut map = Mapping::new();
    map.insert("enable".into(), current.enable.into());
    map.insert("url".into(), current.url.into());

    Ok(map)
}

#[tauri::command]
pub fn open_app_dir() -> CmdResult<()> {
    let app_dir = wrap_err!(dirs::app_home_dir())?;
    wrap_err!(open::that(app_dir))
}

#[tauri::command]
pub fn open_core_dir() -> CmdResult<()> {
    let core_dir = wrap_err!(tauri::utils::platform::current_exe())?;
    let core_dir = core_dir.parent().ok_or("failed to get core dir")?;
    wrap_err!(open::that(core_dir))
}

#[tauri::command]
pub fn open_logs_dir() -> CmdResult<()> {
    let log_dir = wrap_err!(dirs::app_logs_dir())?;
    wrap_err!(open::that(log_dir))
}

#[tauri::command]
pub fn open_web_url(url: String) -> CmdResult<()> {
    wrap_err!(open::that(url))
}

#[cfg(windows)]
pub mod uwp {
    use super::*;
    use crate::core::win_uwp;

    #[tauri::command]
    pub async fn invoke_uwp_tool() -> CmdResult {
        wrap_err!(win_uwp::invoke_uwptools().await)
    }
}

#[tauri::command]
pub async fn clash_api_get_proxy_delay(
    name: String,
    url: Option<String>,
    timeout: i32,
) -> CmdResult<clash_api::DelayRes> {
    match clash_api::get_proxy_delay(name, url, timeout).await {
        Ok(res) => Ok(res),
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
pub fn get_portable_flag() -> CmdResult<bool> {
    Ok(*dirs::PORTABLE_FLAG.get().unwrap_or(&false))
}

#[tauri::command]
pub async fn test_delay(url: String) -> CmdResult<u32> {
    Ok(feat::test_delay(url).await.unwrap_or(10000u32))
}

#[tauri::command]
pub fn get_app_dir() -> CmdResult<String> {
    let app_home_dir = wrap_err!(dirs::app_home_dir())?
        .to_string_lossy()
        .to_string();
    Ok(app_home_dir)
}

#[tauri::command]
pub async fn download_icon_cache(url: String, name: String) -> CmdResult<String> {
    let icon_cache_dir = wrap_err!(dirs::app_home_dir())?.join("icons").join("cache");
    let icon_path = icon_cache_dir.join(name);
    if !icon_cache_dir.exists() {
        let _ = std::fs::create_dir_all(&icon_cache_dir);
    }
    if !icon_path.exists() {
        let response = wrap_err!(reqwest::get(url).await)?;

        let mut file = wrap_err!(std::fs::File::create(&icon_path))?;

        let content = wrap_err!(response.bytes().await)?;
        wrap_err!(std::io::copy(&mut content.as_ref(), &mut file))?;
    }
    Ok(icon_path.to_string_lossy().to_string())
}
#[tauri::command]
pub fn copy_icon_file(path: String, name: String) -> CmdResult<String> {
    let file_path = std::path::Path::new(&path);
    let icon_dir = wrap_err!(dirs::app_home_dir())?.join("icons");
    if !icon_dir.exists() {
        let _ = std::fs::create_dir_all(&icon_dir);
    }
    let ext = match file_path.extension() {
        Some(e) => e.to_string_lossy().to_string(),
        None => "ico".to_string(),
    };

    let png_dest_path = icon_dir.join(format!("{name}.png"));
    let ico_dest_path = icon_dir.join(format!("{name}.ico"));
    let dest_path = icon_dir.join(format!("{name}.{ext}"));
    if file_path.exists() {
        std::fs::remove_file(png_dest_path).unwrap_or_default();
        std::fs::remove_file(ico_dest_path).unwrap_or_default();
        match std::fs::copy(file_path, &dest_path) {
            Ok(_) => Ok(dest_path.to_string_lossy().to_string()),
            Err(err) => Err(err.to_string()),
        }
    } else {
        Err("file not found".to_string())
    }
}

#[tauri::command]
pub fn get_network_interfaces() -> Vec<String> {
    use sysinfo::Networks;
    let mut result = Vec::new();
    let networks = Networks::new_with_refreshed_list();
    for (interface_name, _) in &networks {
        result.push(interface_name.clone());
    }
    result
}

#[tauri::command]
pub fn get_network_interfaces_info() -> CmdResult<Vec<NetworkInterface>> {
    use network_interface::NetworkInterface;
    use network_interface::NetworkInterfaceConfig;

    let names = get_network_interfaces();
    let interfaces = wrap_err!(NetworkInterface::show())?;

    let mut result = Vec::new();

    for interface in interfaces {
        if names.contains(&interface.name) {
            result.push(interface);
        }
    }

    Ok(result)
}

#[tauri::command]
pub fn open_devtools(app_handle: tauri::AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        if !window.is_devtools_open() {
            window.open_devtools();
        } else {
            window.close_devtools();
        }
    }
}

#[tauri::command]
pub async fn exit_app() -> CmdResult<()> {
    log::info!(target: "app", "Exiting application");
    
    // 执行清理工作
    if let Err(e) = CoreManager::global().cleanup_on_exit().await {
        log::error!(target: "app", "Error during exit cleanup: {}", e);
    }

    // 获取app handle
    let app_handle = handle::Handle::global().app_handle().unwrap();
    
    // 关闭主窗口
    if let Some(window) = app_handle.get_webview_window("main") {
        window.close().unwrap_or_else(|e| {
            log::error!(target: "app", "Error closing main window: {}", e);
        });
    }

    // 退出程序
    app_handle.exit(0);
    Ok(())
}

#[tauri::command]
pub async fn save_webdav_config(url: String, username: String, password: String) -> CmdResult<()> {
    let patch = IVerge {
        webdav_url: Some(url),
        webdav_username: Some(username),
        webdav_password: Some(password),
        ..IVerge::default()
    };
    Config::verge().draft().patch_config(patch.clone());
    Config::verge().apply();
    Config::verge()
        .data()
        .save_file()
        .map_err(|err| err.to_string())?;
    backup::WebDavClient::global().reset();
    Ok(())
}

#[tauri::command]
pub async fn create_webdav_backup() -> CmdResult<()> {
    wrap_err!(feat::create_backup_and_upload_webdav().await)
}

#[tauri::command]
pub async fn list_webdav_backup() -> CmdResult<Vec<ListFile>> {
    wrap_err!(feat::list_wevdav_backup().await)
}

#[tauri::command]
pub async fn delete_webdav_backup(filename: String) -> CmdResult<()> {
    wrap_err!(feat::delete_webdav_backup(filename).await)
}

#[tauri::command]
pub async fn restore_webdav_backup(filename: String) -> CmdResult<()> {
    wrap_err!(feat::restore_webdav_backup(filename).await)
}

#[tauri::command]
pub async fn restart_app() -> CmdResult<()> {
    feat::restart_app();
    Ok(())
}

#[cfg(not(windows))]
pub mod uwp {
    use super::*;

    #[tauri::command]
    pub async fn invoke_uwp_tool() -> CmdResult {
        Ok(())
    }
}

// 使用 RAII 模式管理配置状态
struct ConfigGuard {
    original_config: IProfiles,
    should_rollback: std::sync::atomic::AtomicBool,
    rollback_error: std::sync::Mutex<Option<String>>,
}

impl ConfigGuard {
    fn new(config: IProfiles) -> Self {
        Self {
            original_config: config,
            should_rollback: std::sync::atomic::AtomicBool::new(true),
            rollback_error: std::sync::Mutex::new(None),
        }
    }
    
    fn discard(&self) {
        self.should_rollback.store(false, std::sync::atomic::Ordering::SeqCst);
    }
    
    async fn rollback(&self) -> Result<(), String> {
        if !self.should_rollback.load(std::sync::atomic::Ordering::SeqCst) {
            return Ok(());
        }
        
        // 回滚到原始配置
        Config::profiles().discard();
        
        // 尝试应用原始配置
        if let Err(e) = Config::profiles().draft().patch_config(self.original_config.clone()) {
            let error = format!("回滚配置失败: {}", e);
            *self.rollback_error.lock().unwrap() = Some(error.clone());
            return Err(error);
        }
        
        Config::profiles().apply();
        
        // 重新应用原始配置
        match CoreManager::global().update_config().await {
            Ok(_) => {
                log::info!(target: "app", "Successfully rolled back configuration");
                Ok(())
            }
            Err(e) => {
                let error = format!("回滚配置失败: {}", e);
                *self.rollback_error.lock().unwrap() = Some(error.clone());
                Err(error)
            }
        }
    }
}

impl Drop for ConfigGuard {
    fn drop(&mut self) {
        if self.should_rollback.load(std::sync::atomic::Ordering::SeqCst) {
            // 获取可能的回滚错误
            let error = self.rollback_error.lock().unwrap().clone();
            match error {
                Some(err) => {
                    log::error!(
                        target: "app",
                        "ConfigGuard dropped while rollback was needed, previous rollback error: {}",
                        err
                    );
                }
                None => {
                    log::warn!(
                        target: "app",
                        "ConfigGuard dropped while rollback was needed, no previous rollback attempt"
                    );
                }
            }
        }
    }
}
