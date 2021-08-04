use std::{env, fs, path::PathBuf, time::Duration};

use anyhow::Result;
use dbus::blocking::{stdintf::org_freedesktop_dbus::Introspectable, Connection};
use dbus_codegen::GenOpts;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "node")]
struct Node {
    #[serde(rename = "interface", default)]
    interfaces: Vec<Interface>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Interface {
    name: String,

    #[serde(rename = "method", default)]
    methods: Vec<MethodOrSignal>,

    #[serde(rename = "signal", default)]
    signals: Vec<MethodOrSignal>,
}

#[derive(Debug, Deserialize, Serialize)]
struct MethodOrSignal {
    name: String,

    #[serde(rename = "arg", default)]
    args: Vec<Arg>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Arg {
    #[serde(rename = "type")]
    type_: String,

    name: String,
    direction: Option<String>,
}

fn main() -> Result<()> {
    // Basically, we're going to generate code to talk to the swaync DBus
    // service. Ideally, we'll do so by interrogating a running service, but if
    // there isn't one, we can use a canned file included alongside this build
    // script.

    let conn = Connection::new_session()?;
    let proxy = conn.with_proxy(
        "org.erikreider.swaync.cc",
        "/org/erikreider/swaync/cc",
        Duration::from_millis(1000),
    );

    let dest = PathBuf::from(env::var("OUT_DIR")?).join("swaync.rs");
    match proxy.introspect() {
        Ok(data) => {
            // We have the freshest XML! Let's use it to generate swaync.rs.

            // FIXME: we have to exclude AddNotification right now because it
            // has 18 (!) arguments and dbus-codegen breaks after 15, per
            // https://github.com/diwic/dbus-rs/issues/310. We'll just munge the
            // XML for now.
            let mut doc: Node = quick_xml::de::from_str(&data)?;
            doc.interfaces
                .iter_mut()
                .filter(|iface| iface.name == "org.erikreider.swaync.cc")
                .for_each(|iface| {
                    iface
                        .methods
                        .retain(|method| method.name != "AddNotification");
                });

            let xml = quick_xml::se::to_string(&doc)?;

            // Now we can actually generate a Rust client.
            let opts = GenOpts {
                methodtype: None,
                crhandler: None,
                ..GenOpts::default()
            };
            let code = dbus_codegen::generate(&xml, &opts)
                .map_err(|e| anyhow::anyhow!("Error generating code from XML: {}", e))?;

            fs::write(dest, code)?;
        }
        Err(_) => {
            // DBus doesn't have the right service available to introspect. No
            // matter; we can use one we prepared earlier.
            fs::copy(
                PathBuf::from(env::var("CARGO_MANIFEST_DIR")?).join("swaync.rs"),
                dest,
            )?;
        }
    }

    Ok(())
}
