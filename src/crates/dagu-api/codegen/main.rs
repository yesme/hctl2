//! Generate the Dagu v1 OpenAPI client from the locked `api/v1/api.yaml` specification.

#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::path::PathBuf;

fn scrub_value(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            map.remove("not");
            map.remove("pattern");
            if let Some(serde_json::Value::String(fmt)) = map.get("format")
                && matches!(
                    fmt.as_str(),
                    "date-time" | "date" | "uuid" | "uri" | "email" | "hostname" | "ipv4" | "ipv6"
                )
            {
                map.remove("format");
            }
            for child in map.values_mut() {
                scrub_value(child);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                scrub_value(item);
            }
        }
        _ => {}
    }
}

fn is_vec_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty
        && let Some(seg) = type_path.path.segments.last()
    {
        return seg.ident == "Vec";
    }
    false
}

fn has_serde_default(attrs: &[syn::Attribute]) -> bool {
    for attr in attrs {
        if attr.path().is_ident("serde")
            && let syn::Meta::List(list) = &attr.meta
        {
            let tokens = list.tokens.to_string();
            for part in tokens.split(',') {
                if part.trim() == "default" {
                    return true;
                }
            }
        }
    }
    false
}

fn struct_derives_deserialize(attrs: &[syn::Attribute]) -> bool {
    for attr in attrs {
        if attr.path().is_ident("derive")
            && let syn::Meta::List(list) = &attr.meta
            && list.tokens.to_string().contains("Deserialize")
        {
            return true;
        }
    }
    false
}

fn patch_nullable_go_slices(file: &mut syn::File) {
    let helper: syn::Item = syn::parse_str(
        r#"
        fn deserialize_null_default<'de, D, T>(deserializer: D) -> ::std::result::Result<T, D::Error>
        where
            T: ::std::default::Default + ::serde::Deserialize<'de>,
            D: ::serde::Deserializer<'de>,
        {
            use ::serde::Deserialize as _;
            Ok(::std::option::Option::<T>::deserialize(deserializer)?.unwrap_or_default())
        }
        "#,
    )
    .expect("parse deserialize_null_default helper");

    let attr_with_default: syn::Attribute = {
        let item: syn::ItemStruct = syn::parse_str(
            "#[serde(default, deserialize_with = \"deserialize_null_default\")] struct S;",
        )
        .expect("parse serde attr with default");
        item.attrs.into_iter().next().expect("attr")
    };
    let attr_without_default: syn::Attribute = {
        let item: syn::ItemStruct =
            syn::parse_str("#[serde(deserialize_with = \"deserialize_null_default\")] struct S;")
                .expect("parse serde attr without default");
        item.attrs.into_iter().next().expect("attr")
    };

    for item in &mut file.items {
        if let syn::Item::Mod(module) = item
            && module.ident == "types"
            && let Some((_, items)) = &mut module.content
        {
            items.insert(0, helper.clone());
            for sub in items.iter_mut() {
                if let syn::Item::Struct(st) = sub
                    && struct_derives_deserialize(&st.attrs)
                    && let syn::Fields::Named(named) = &mut st.fields
                {
                    for field in &mut named.named {
                        if is_vec_type(&field.ty) {
                            if has_serde_default(&field.attrs) {
                                field.attrs.push(attr_without_default.clone());
                            } else {
                                field.attrs.push(attr_with_default.clone());
                            }
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    let mut args = env::args().skip(1);
    let input = args
        .next()
        .map_or_else(|| PathBuf::from("api/v1/api.yaml"), PathBuf::from);
    let output = args
        .next()
        .map_or_else(|| PathBuf::from("src/generated.rs"), PathBuf::from);

    let yaml_str = fs::read_to_string(&input).expect("read api/v1/api.yaml");
    let mut raw: serde_json::Value =
        serde_yaml::from_str::<serde_json::Value>(&yaml_str).expect("parse OpenAPI YAML");
    scrub_value(&mut raw);
    let mut spec: openapiv3::OpenAPI =
        serde_json::from_value::<openapiv3::OpenAPI>(raw).expect("parse OpenAPI 3.0 document");

    let whitelist = [
        "/health",
        "/auth/setup",
        "/auth/login",
        "/api-keys",
        "/dags",
        "/dags/validate",
        "/dags/{fileName}",
        "/dags/{fileName}/spec",
        "/dags/{fileName}/start",
        "/dags/{fileName}/enqueue",
        "/dags/{fileName}/rename",
        "/dag-runs",
        "/dag-runs/enqueue",
        "/dag-runs/{name}/{dagRunId}",
        "/dag-runs/{name}/{dagRunId}/stop",
        "/dag-runs/{name}/{dagRunId}/retry",
        "/dag-runs/{name}/{dagRunId}/reschedule",
        "/dag-runs/{name}/{dagRunId}/human-tasks/{stepId}/complete",
        "/dag-runs/{name}/{dagRunId}/human-tasks/resume",
        "/dag-runs/{name}/{dagRunId}/steps/{stepName}/status",
        "/dag-runs/{name}/{dagRunId}/steps/{stepName}/approve",
        "/dag-runs/{name}/{dagRunId}/steps/{stepName}/reject",
    ];
    spec.paths
        .paths
        .retain(|key, _| whitelist.contains(&key.as_str()));
    for item in spec.paths.paths.values_mut() {
        if let openapiv3::ReferenceOr::Item(item) = item {
            for op in [
                &mut item.get,
                &mut item.post,
                &mut item.put,
                &mut item.delete,
                &mut item.patch,
            ]
            .into_iter()
            .flatten()
            {
                if !op.responses.responses.is_empty() {
                    op.responses.default = None;
                }
            }
        }
    }

    if let Some(components) = &mut spec.components {
        for schema_name in ["DAGRunSummary", "DAGRunDetails"] {
            if let Some(openapiv3::ReferenceOr::Item(schema)) =
                components.schemas.get_mut(schema_name)
                && let openapiv3::SchemaKind::Type(openapiv3::Type::Object(obj)) =
                    &mut schema.schema_kind
            {
                for field in ["labels", "tags"] {
                    if let Some(openapiv3::ReferenceOr::Item(prop)) = obj.properties.get_mut(field)
                    {
                        prop.schema_data.nullable = true;
                    }
                }
            }
        }
    }

    let mut generator = progenitor::Generator::default();
    let tokens = generator.generate_tokens(&spec).expect("generate_tokens");
    let mut file = syn::parse2::<syn::File>(tokens).expect("parse generated tokens");
    patch_nullable_go_slices(&mut file);
    let formatted = prettyplease::unparse(&file);
    if let Some(parent) = output.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).expect("create output parent directory");
    }
    fs::write(&output, formatted).expect("write generated source");
}
