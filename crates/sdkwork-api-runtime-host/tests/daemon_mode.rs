use pingora_core::server::configuration::{Opt, ServerConf};

#[test]
fn rejects_daemon_mode_from_yaml() {
    let error = ServerConf::from_yaml(
        r#"
---
version: 1
daemon: true
"#,
    )
    .expect_err("daemon mode should be rejected");

    let message = error.to_string();
    assert!(message.contains("foreground"));
    assert!(message.contains("service manager"));
}

#[test]
fn rejects_daemon_mode_from_cli_override() {
    let opt = Opt {
        daemon: true,
        ..Default::default()
    };

    let conf = ServerConf::new_with_opt_override(&opt);
    assert!(conf.is_none(), "daemon CLI override should be rejected");
}
