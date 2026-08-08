//! HTML templates for the installer wizard

use crate::InstallConfig;

const STYLE: &str = r#"
<style>
* { box-sizing: border-box; margin: 0; padding: 0; }
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%); min-height: 100vh; color: #fff; }
.container { max-width: 600px; margin: 0 auto; padding: 40px 20px; }
.card { background: rgba(255,255,255,0.1); border-radius: 16px; padding: 40px; backdrop-filter: blur(10px); }
.logo { text-align: center; margin-bottom: 30px; }
.logo h1 { font-size: 2.5em; color: #00d4ff; }
h2 { margin-bottom: 20px; }
.form-group { margin-bottom: 20px; }
label { display: block; margin-bottom: 8px; color: #ccc; }
input { width: 100%; padding: 12px; border: 1px solid rgba(255,255,255,0.2); border-radius: 8px; background: rgba(0,0,0,0.3); color: #fff; }
.btn { padding: 14px 28px; background: #00d4ff; color: #000; border: none; border-radius: 8px; cursor: pointer; text-decoration: none; display: inline-block; }
.buttons { display: flex; justify-content: space-between; margin-top: 30px; }
.check { padding: 12px; background: rgba(0,0,0,0.2); border-radius: 8px; margin-bottom: 10px; }
.check.pass { border-left: 3px solid #4ade80; }
.check.fail { border-left: 3px solid #f87171; }
.summary { background: rgba(0,0,0,0.2); border-radius: 8px; padding: 20px; margin-bottom: 20px; }
.summary-row { display: flex; justify-content: space-between; padding: 8px 0; border-bottom: 1px solid rgba(255,255,255,0.1); }
</style>
"#;

pub struct RequirementCheck {
    pub name: String,
    pub passed: bool,
    pub message: String,
}

fn header() -> String {
    format!(r#"<!DOCTYPE html><html><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Raksha Installer</title>{STYLE}</head><body><div class="container"><div class="logo"><h1>🛡️ Raksha</h1><p>Security Platform</p></div><div class="card">"#)
}

fn footer() -> &'static str {
    "</div></div></body></html>"
}

pub fn welcome() -> String {
    format!(r#"{}
    <h2>Welcome to Raksha Installation</h2>
    <p style="color:#aaa;margin-bottom:20px">This wizard will set up your security platform.</p>
    <div class="buttons"><span></span><a href="/install/requirements" class="btn">Get Started →</a></div>
    {}"#, header(), footer())
}

pub fn requirements(checks: &[RequirementCheck]) -> String {
    let checks_html: String = checks.iter().map(|c| {
        format!(r#"<div class="check {}">{} <strong>{}</strong> - {}</div>"#,
            if c.passed { "pass" } else { "fail" },
            if c.passed { "✓" } else { "✗" },
            c.name, c.message)
    }).collect();
    let all_ok = checks.iter().all(|c| c.passed);
    format!(r#"{}<h2>System Requirements</h2>{}<div class="buttons">
        <a href="/install" class="btn">← Back</a>
        {}</div>{}"#, header(), checks_html,
        if all_ok { r#"<a href="/install/database" class="btn">Continue →</a>"# } else { "<span></span>" },
        footer())
}

pub fn database(config: &InstallConfig) -> String {
    format!(r#"{}<h2>Database Configuration</h2>
    <form method="POST" action="/install/database">
    <div class="form-group"><label>Host</label><input name="db_host" value="{}" required></div>
    <div class="form-group"><label>Port</label><input name="db_port" type="number" value="{}" required></div>
    <div class="form-group"><label>Database</label><input name="db_name" value="{}" required></div>
    <div class="form-group"><label>User</label><input name="db_user" value="{}" required></div>
    <div class="form-group"><label>Password</label><input name="db_password" type="password" value="{}" required></div>
    <div class="form-group"><label>Redis URL</label><input name="redis_url" value="{}"></div>
    <div class="buttons"><a href="/install/requirements" class="btn">← Back</a><button class="btn">Continue →</button></div>
    </form>{}"#, header(),
        if config.db_host.is_empty() { "localhost" } else { &config.db_host },
        if config.db_port == 0 { 5432 } else { config.db_port },
        if config.db_name.is_empty() { "raksha" } else { &config.db_name },
        if config.db_user.is_empty() { "raksha" } else { &config.db_user },
        config.db_password,
        if config.redis_url.is_empty() { "redis://localhost:6379" } else { &config.redis_url },
        footer())
}

pub fn admin(config: &InstallConfig) -> String {
    format!(r#"{}<h2>SuperAdmin Account</h2>
    <form method="POST" action="/install/admin">
    <div class="form-group"><label>Site Name</label><input name="site_name" value="{}" required></div>
    <div class="form-group"><label>Admin Name</label><input name="admin_name" value="{}" required></div>
    <div class="form-group"><label>Admin Email</label><input name="admin_email" type="email" value="{}" required></div>
    <div class="form-group"><label>Password</label><input name="admin_password" type="password" minlength="8" required></div>
    <div class="buttons"><a href="/install/database" class="btn">← Back</a><button class="btn">Continue →</button></div>
    </form>{}"#, header(),
        if config.site_name.is_empty() { "Raksha Security" } else { &config.site_name },
        config.admin_name, config.admin_email, footer())
}

pub fn finish(config: &InstallConfig) -> String {
    format!(r#"{}<h2>Ready to Install</h2>
    <div class="summary">
    <div class="summary-row"><span>Database</span><span>{}:{}/{}</span></div>
    <div class="summary-row"><span>Admin</span><span>{}</span></div>
    </div>
    <form method="POST" action="/install/run">
    <div class="buttons"><a href="/install/admin" class="btn">← Back</a><button class="btn">🚀 Install Now</button></div>
    </form>{}"#, header(), config.db_host, config.db_port, config.db_name, config.admin_email, footer())
}

pub fn success() -> String {
    format!(r#"{}<h2 style="text-align:center">🎉 Installation Complete!</h2>
    <p style="text-align:center;color:#aaa;margin:20px 0">You can now login with your SuperAdmin account.</p>
    <div style="text-align:center"><a href="/" class="btn">Go to Login →</a></div>{}"#, header(), footer())
}

pub fn error(msg: &str) -> String {
    format!(r#"{}<h2>❌ Installation Failed</h2><p style="color:#f87171">{}</p>
    <div class="buttons"><a href="/install/finish" class="btn">← Try Again</a></div>{}"#, header(), msg, footer())
}
