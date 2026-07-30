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

fn unwrap_val<'a, 'b>(val: &'a zbus::zvariant::Value<'b>) -> &'a zbus::zvariant::Value<'b> {
    let mut curr = val;
    while let zbus::zvariant::Value::Value(inner) = curr {
        curr = inner;
    }
    curr
}

pub async fn fetch_dbus_menu(
    conn: &zbus::Connection,
    bus_name: &str,
    menu_path: &str,
) -> Vec<DBusMenuItem> {
    let interfaces = ["com.canonical.dbusmenu", "org.ayatana.dbusmenu"];

    for iface in interfaces {
        if let Ok(msg) = conn
            .call_method(
                Some(bus_name),
                menu_path,
                Some(iface),
                "GetLayout",
                &(0i32, 10i32, Vec::<String>::new()),
            )
            .await
        {
            let body = msg.body();
            type MenuLayout<'a> = (
                i32,
                HashMap<String, zbus::zvariant::Value<'a>>,
                Vec<zbus::zvariant::Value<'a>>,
            );
            if let Ok((_rev, (_root_id, _props, children))) =
                body.deserialize::<(u32, MenuLayout<'_>)>()
            {
                let mut items = Vec::new();
                for child in children {
                    if let Some(item) = parse_single_item(&child) {
                        items.push(item);
                    }
                }
                if !items.is_empty() {
                    return items;
                }
            } else if let Ok((_rev, root_val)) = body.deserialize::<(u32, zbus::zvariant::Value)>()
            {
                let items = parse_menu_node(&root_val);
                if !items.is_empty() {
                    return items;
                }
            }
        }
    }

    vec![]
}

pub async fn trigger_dbus_menu_item(
    conn: &zbus::Connection,
    bus_name: &str,
    menu_path: &str,
    item_id: i32,
) -> anyhow::Result<()> {
    let interfaces = ["com.canonical.dbusmenu", "org.ayatana.dbusmenu"];
    for iface in interfaces {
        let res = conn
            .call_method(
                Some(bus_name),
                menu_path,
                Some(iface),
                "Event",
                &(item_id, "clicked", zbus::zvariant::Value::U32(0), 0u32),
            )
            .await;
        if res.is_ok() {
            return Ok(());
        }
    }
    Ok(())
}

fn parse_menu_node(val: &zbus::zvariant::Value) -> Vec<DBusMenuItem> {
    let mut items = Vec::new();
    let unwrapped = unwrap_val(val);
    let root_struct = match unwrapped {
        zbus::zvariant::Value::Structure(s) => s,
        _ => {
            return items;
        }
    };

    let fields = root_struct.fields();
    if fields.len() < 3 {
        return items;
    }

    // Children array is field index 2
    let children_val = unwrap_val(&fields[2]);
    let children_arr = match children_val {
        zbus::zvariant::Value::Array(a) => a,
        _ => {
            return items;
        }
    };

    for child in children_arr.iter() {
        if let Some(item) = parse_single_item(child) {
            items.push(item);
        }
    }

    items
}

fn parse_single_item(val: &zbus::zvariant::Value) -> Option<DBusMenuItem> {
    let unwrapped = unwrap_val(val);
    let s = match unwrapped {
        zbus::zvariant::Value::Structure(s) => s,
        _ => {
            return None;
        }
    };

    let fields = s.fields();
    if fields.len() < 2 {
        return None;
    }

    let id = match unwrap_val(&fields[0]) {
        zbus::zvariant::Value::I32(i) => *i,
        zbus::zvariant::Value::U32(u) => *u as i32,
        _ => {
            return None;
        }
    };

    let mut label = String::new();
    let mut enabled = true;
    let mut visible = true;
    let mut is_separator = false;

    let dict_val = unwrap_val(&fields[1]);
    if let zbus::zvariant::Value::Dict(d) = dict_val {
        let mut map: HashMap<String, &zbus::zvariant::Value> = HashMap::new();
        for (k, v) in d.iter() {
            let k_unwrapped = unwrap_val(k);
            if let zbus::zvariant::Value::Str(ks) = k_unwrapped {
                map.insert(ks.to_string(), unwrap_val(v));
            }
        }

        if let Some(val) = map.get("label") {
            if let zbus::zvariant::Value::Str(l) = unwrap_val(val) {
                label = l.replace('_', "");
            }
        }
        if let Some(val) = map.get("enabled") {
            if let zbus::zvariant::Value::Bool(b) = unwrap_val(val) {
                enabled = *b;
            }
        }
        if let Some(val) = map.get("visible") {
            if let zbus::zvariant::Value::Bool(b) = unwrap_val(val) {
                visible = *b;
            }
        }
        if let Some(val) = map.get("type") {
            if let zbus::zvariant::Value::Str(t) = unwrap_val(val) {
                if t == "separator" {
                    is_separator = true;
                }
            }
        }
    }

    if !visible {
        return None;
    }

    let mut children = Vec::new();
    if fields.len() >= 3 {
        let children_val = unwrap_val(&fields[2]);
        if let zbus::zvariant::Value::Array(arr) = children_val {
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
