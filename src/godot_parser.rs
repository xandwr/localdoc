use quick_xml::de::from_str;
use serde::Deserialize;
use std::fs;
use std::path::Path;

use crate::docpack::{DocEntry, EntryType, Example, Manifest};

#[derive(Debug, Deserialize)]
struct GodotClass {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@inherits", default)]
    inherits: Option<String>,
    brief_description: String,
    description: String,
    #[serde(default)]
    methods: Methods,
    #[serde(default)]
    members: Members,
    #[serde(default)]
    signals: Signals,
    #[serde(default)]
    constants: Constants,
}

#[derive(Debug, Deserialize, Default)]
struct Methods {
    #[serde(rename = "method", default)]
    method: Vec<Method>,
}

#[derive(Debug, Deserialize)]
struct Method {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@qualifiers", default)]
    qualifiers: Option<String>,
    #[serde(rename = "return")]
    return_type: ReturnType,
    description: String,
    #[serde(default)]
    param: Vec<Param>,
}

#[derive(Debug, Deserialize)]
struct ReturnType {
    #[serde(rename = "@type")]
    type_name: String,
}

#[derive(Debug, Deserialize)]
struct Param {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@type")]
    type_name: String,
}

#[derive(Debug, Deserialize, Default)]
struct Members {
    #[serde(rename = "member", default)]
    member: Vec<Member>,
}

#[derive(Debug, Deserialize)]
struct Member {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@type")]
    type_name: String,
    #[serde(default)]
    description: String,
}

#[derive(Debug, Deserialize, Default)]
struct Signals {
    #[serde(rename = "signal", default)]
    signal: Vec<Signal>,
}

#[derive(Debug, Deserialize)]
struct Signal {
    #[serde(rename = "@name")]
    name: String,
    description: String,
}

#[derive(Debug, Deserialize, Default)]
struct Constants {
    #[serde(rename = "constant", default)]
    constant: Vec<Constant>,
}

#[derive(Debug, Deserialize)]
struct Constant {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@value")]
    value: String,
    #[serde(default)]
    description: String,
}

pub fn parse_godot_xml(xml_path: &Path) -> Result<Vec<DocEntry>, Box<dyn std::error::Error>> {
    let xml_content = fs::read_to_string(xml_path)?;
    let godot_class: GodotClass = from_str(&xml_content)?;
    
    let mut entries = Vec::new();
    let class_name = &godot_class.name;
    
    // Add the class itself
    let class_id = format!("godot::{}", class_name);
    let mut class_content = format!("# {}\n\n", class_name);
    
    if let Some(ref inherits) = godot_class.inherits {
        class_content.push_str(&format!("**Inherits:** {}\n\n", inherits));
    }
    
    class_content.push_str(&format!("{}\n\n", godot_class.description));
    
    let class_entry = DocEntry::builder(
        class_id.clone(),
        EntryType::Class,
        class_name.clone(),
    )
    .path("godot".to_string())
    .title(class_name.clone())
    .summary(godot_class.brief_description.clone())
    .content(class_content)
    .tags(vec!["godot".to_string(), "class".to_string()])
    .build();
    
    entries.push(class_entry);
    
    // Add methods
    for method in &godot_class.methods.method {
        let method_id = format!("godot::{}::{}", class_name, method.name);
        let params_str = method.param.iter()
            .map(|p| format!("{}: {}", p.name, p.type_name))
            .collect::<Vec<_>>()
            .join(", ");
        
        let signature = format!("{}({}) -> {}", 
            method.name, 
            params_str,
            method.return_type.type_name
        );
        
        let mut method_content = format!("# {}.{}\n\n", class_name, method.name);
        method_content.push_str(&format!("```gdscript\n{}\n```\n\n", signature));
        method_content.push_str(&method.description);
        
        let method_entry = DocEntry::builder(
            method_id,
            EntryType::Method,
            method.name.clone(),
        )
        .path(format!("godot::{}", class_name))
        .title(format!("{}.{}", class_name, method.name))
        .summary(format!("{} method", method.name))
        .content(method_content)
        .tags(vec!["godot".to_string(), "method".to_string(), class_name.to_lowercase()])
        .build();
        
        entries.push(method_entry);
    }
    
    // Add properties/members
    for member in &godot_class.members.member {
        let member_id = format!("godot::{}::{}", class_name, member.name);
        let mut member_content = format!("# {}.{}\n\n", class_name, member.name);
        member_content.push_str(&format!("**Type:** {}\n\n", member.type_name));
        member_content.push_str(&member.description);
        
        let member_entry = DocEntry::builder(
            member_id,
            EntryType::Other("property".to_string()),
            member.name.clone(),
        )
        .path(format!("godot::{}", class_name))
        .title(format!("{}.{}", class_name, member.name))
        .summary(format!("{} property", member.name))
        .content(member_content)
        .tags(vec!["godot".to_string(), "property".to_string(), class_name.to_lowercase()])
        .build();
        
        entries.push(member_entry);
    }
    
    Ok(entries)
}

pub fn create_godot_manifest(version: &str) -> Manifest {
    Manifest::new("godot", version, "game-engine")
}
