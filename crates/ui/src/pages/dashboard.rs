//! Dashboard page

use leptos::*;
use tinystore_shared::api::{ServerStatus, StorageStats};
use crate::api::client;

#[component]
pub fn Dashboard() -> impl IntoView {
    let (server_status, set_server_status) = create_signal(None::<ServerStatus>);
    let (storage_stats, set_storage_stats) = create_signal(None::<StorageStats>);
    let (error, set_error) = create_signal(None::<String>);
    let (loading, set_loading) = create_signal(true);

    // Fetch data on mount
    create_effect(move |_| {
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            match client::fetch_server_status().await {
                Ok(status) => set_server_status.set(Some(status)),
                Err(e) => set_error.set(Some(format!("Failed to fetch server status: {}", e))),
            }

            match client::fetch_storage_stats().await {
                Ok(stats) => set_storage_stats.set(Some(stats)),
                Err(e) => set_error.set(Some(format!("Failed to fetch storage stats: {}", e))),
            }

            set_loading.set(false);
        });
    });

    view! {
        <div class="dashboard">
            <div class="page-header">
                <h1 class="page-title">"Dashboard"</h1>
                <p class="page-subtitle">"Overview of your TinyStore instance"</p>
            </div>

            {move || error.get().map(|err| view! {
                <div class="alert alert-error">
                    <span class="alert-icon">"⚠️"</span>
                    <span>{err}</span>
                </div>
            })}

            {move || if loading.get() {
                view! {
                    <div class="loading">
                        <div class="spinner"></div>
                        <p>"Loading dashboard..."</p>
                    </div>
                }.into_view()
            } else {
                view! {
                    <div class="dashboard-grid">
                        // Server Status Card
                        <div class="card">
                            <div class="card-header">
                                <h2 class="card-title">"Server Status"</h2>
                            </div>
                            <div class="card-body">
                                {move || server_status.get().map(|status| view! {
                                    <div class="stats-list">
                                        <div class="stat-item">
                                            <span class="stat-label">"Version:"</span>
                                            <span class="stat-value">{status.version}</span>
                                        </div>
                                        <div class="stat-item">
                                            <span class="stat-label">"Uptime:"</span>
                                            <span class="stat-value">{format_uptime(status.uptime_seconds)}</span>
                                        </div>
                                        <div class="stat-item">
                                            <span class="stat-label">"Memory Usage:"</span>
                                            <span class="stat-value">{format!("{:.2} MB", status.memory_usage_mb)}</span>
                                        </div>
                                    </div>
                                })}
                            </div>
                        </div>

                        // Storage Statistics Card
                        <div class="card">
                            <div class="card-header">
                                <h2 class="card-title">"Storage Statistics"</h2>
                            </div>
                            <div class="card-body">
                                {move || storage_stats.get().map(|stats| view! {
                                    <div class="stats-list">
                                        <div class="stat-item">
                                            <span class="stat-label">"Total Buckets:"</span>
                                            <span class="stat-value">{stats.total_buckets}</span>
                                        </div>
                                        <div class="stat-item">
                                            <span class="stat-label">"Total Objects:"</span>
                                            <span class="stat-value">{stats.total_objects}</span>
                                        </div>
                                        <div class="stat-item">
                                            <span class="stat-label">"Total Size:"</span>
                                            <span class="stat-value">{format_bytes(stats.total_size_bytes)}</span>
                                        </div>
                                    </div>
                                })}
                            </div>
                        </div>

                        // Quick Actions Card
                        <div class="card">
                            <div class="card-header">
                                <h2 class="card-title">"Quick Actions"</h2>
                            </div>
                            <div class="card-body">
                                <div class="action-buttons">
                                    <a href="/ui/buckets" class="btn btn-primary">
                                        "View Buckets"
                                    </a>
                                    <a href="/ui/settings" class="btn btn-secondary">
                                        "Manage Credentials"
                                    </a>
                                </div>
                            </div>
                        </div>
                    </div>
                }.into_view()
            }}
        </div>
    }
}

/// Format bytes into human-readable format
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

/// Format uptime in seconds to human-readable format
fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, minutes, secs)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}
