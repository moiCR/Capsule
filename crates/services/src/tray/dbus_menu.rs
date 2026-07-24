use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DBusMenuItem {
    pub id: i32,
    pub label: String,
    pub enabled: bool,
    pub visible: bool,
    pub is_separator: bool,
    pub children: Vec<DBusMenuItem>,
}

pub async fn fetch_dbus_menu(
    conn: &zbus::Connection,
    bus_name: &str,
    menu_path: &str,
) -> Vec<DBusMenuItem> {
    let msg = match conn
        .call_method(
            Some(bus_name),
            menu_path,
            Some("com.canonical.dbusmenu"),
            "GetLayout",
            &(0i32, 2i32, Vec::<String>::new()),
        )
        .await
    {
        Ok(m) => m,
        Err(_) => return vec![],
    };

    let body = msg.body();
    // Layout signature: u(ia{sv}av) -> revision, root_node
    let res: Result<(u32, zbus::zvariant::Value), _> = body.deserialize();
    let (_, root_val) = match res {
        Ok(val) => val,
        Err(_) => return vec![],
    };

    parse_menu_node(&root_val)
}

pub async fn trigger_dbus_menu_item(
    conn: &zbus::Connection,
    bus_name: &str,
    menu_path: &str,
    item_id: i32,
) -> anyhow::Result<()> {
    conn.call_method(
        Some(bus_name),
        menu_path,
        Some("com.canonical.dbusmenu"),
        "Event",
        &(item_id, "clicked", zbus::zvariant::Value::U32(0), 0u32),
    )
    .await?;
    Ok(())
}

fn parse_menu_node(val: &zbus::zvariant::Value) -> Vec<DBusMenuItem> {
    let mut items = Vec::new();
    let root_struct = match val {
        zbus::zvariant::Value::Structure(s) => s,
        _ => return items,
    };

    let fields = root_struct.fields();
    if fields.len() < 3 {
        return items;
    }

    // Children array is field index 2
    let children_arr = match &fields[2] {
        zbus::zvariant::Value::Array(a) => a,
        _ => return items,
    };

    for child in children_arr.iter() {
        if let Some(item) = parse_single_item(child) {
            items.push(item);
        }
    }

    items
}

fn parse_single_item(val: &zbus::zvariant::Value) -> Option<DBusMenuItem> {
    let s = match val {
        zbus::zvariant::Value::Structure(s) => s,
        _ => return None,
    };

    let fields = s.fields();
    if fields.len() < 2 {
        return None;
    }

    let id = match &fields[0] {
        zbus::zvariant::Value::I32(i) => *i,
        _ => return None,
    };

    let mut label = String::new();
    let mut enabled = true;
    let mut visible = true;
    let mut is_separator = false;

    if let zbus::zvariant::Value::Dict(d) = &fields[1] {
        let mut map: HashMap<String, zbus::zvariant::Value> = HashMap::new();
        for (k, v) in d.iter() {
            if let zbus::zvariant::Value::Str(ks) = k {
                map.insert(ks.to_string(), v.clone());
            }
        }

        if let Some(zbus::zvariant::Value::Str(l)) = map.get("label") {
            // Strip underscores used for hotkey mnemonics (e.g. "_Exit" -> "Exit")
            label = l.replace('_', "");
        }
        if let Some(zbus::zvariant::Value::Bool(b)) = map.get("enabled") {
            enabled = *b;
        }
        if let Some(zbus::zvariant::Value::Bool(b)) = map.get("visible") {
            visible = *b;
        }
        if let Some(zbus::zvariant::Value::Str(t)) = map.get("type") {
            if t == "separator" {
                is_separator = true;
            }
        }
    }

    if !visible {
        return None;
    }

    let mut children = Vec::new();
    if fields.len() >= 3 {
        if let zbus::zvariant::Value::Array(arr) = &fields[2] {
            for c in arr.iter() {
                if let Some(child_item) = parse_single_item(c) {
                    children.push(child_item);
                }
            }
        }
    }

    Some(DBusMenuItem {
        id,
        label,
        enabled,
        visible,
        is_separator,
        children,
    })
}
