use base64::{engine::general_purpose::STANDARD as base64, Engine as _};
use std::{
    fmt::Display,
    io::{self, Cursor},
};

use soroban_env_host::xdr::{
    self, Limited, Limits, ReadXdr, ScEnvMetaEntry, ScMetaEntry, ScMetaV0, ScSpecEntry,
    ScSpecFunctionV0, ScSpecUdtEnumV0, ScSpecUdtErrorEnumV0, ScSpecUdtStructV0, ScSpecUdtUnionV0,
    StringM, WriteXdr,
};

pub struct Spec {
    pub env_meta_base64: Option<String>,
    pub env_meta: Vec<ScEnvMetaEntry>,
    pub meta_base64: Option<String>,
    pub meta: Vec<ScMetaEntry>,
    pub spec_base64: Option<String>,
    pub spec: Vec<ScSpecEntry>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("reading file {filepath}: {error}")]
    CannotReadContractFile {
        filepath: std::path::PathBuf,
        error: io::Error,
    },
    #[error("cannot parse wasm file {file}: {error}")]
    CannotParseWasm {
        file: std::path::PathBuf,
        error: wasmparser::BinaryReaderError,
    },
    #[error("xdr processing error: {0}")]
    Xdr(#[from] xdr::Error),

    #[error(transparent)]
    Parser(#[from] wasmparser::BinaryReaderError),
}

impl Spec {
    pub fn new(bytes: &[u8]) -> Result<Self, Error> {
        let mut env_meta: Option<&[u8]> = None;
        let mut meta: Option<&[u8]> = None;
        let mut spec: Option<&[u8]> = None;
        for payload in wasmparser::Parser::new(0).parse_all(bytes) {
            let payload = payload?;
            if let wasmparser::Payload::CustomSection(section) = payload {
                let out = match section.name() {
                    "contractenvmetav0" => &mut env_meta,
                    "contractmetav0" => &mut meta,
                    "contractspecv0" => &mut spec,
                    _ => continue,
                };
                *out = Some(section.data());
            };
        }

        let mut env_meta_base64 = None;
        let env_meta = if let Some(env_meta) = env_meta {
            env_meta_base64 = Some(base64.encode(env_meta));
            let cursor = Cursor::new(env_meta);
            let mut read = Limited::new(cursor, Limits::none());
            ScEnvMetaEntry::read_xdr_iter(&mut read).collect::<Result<Vec<_>, xdr::Error>>()?
        } else {
            vec![]
        };

        let mut meta_base64 = None;
        let meta = if let Some(meta) = meta {
            meta_base64 = Some(base64.encode(meta));
            let cursor = Cursor::new(meta);
            let mut depth_limit_read = Limited::new(cursor, Limits::none());
            ScMetaEntry::read_xdr_iter(&mut depth_limit_read)
                .collect::<Result<Vec<_>, xdr::Error>>()?
        } else {
            vec![]
        };

        let mut spec_base64 = None;
        let spec = if let Some(spec) = spec {
            spec_base64 = Some(base64.encode(spec));
            let cursor = Cursor::new(spec);
            let mut read = Limited::new(cursor, Limits::none());
            ScSpecEntry::read_xdr_iter(&mut read).collect::<Result<Vec<_>, xdr::Error>>()?
        } else {
            vec![]
        };

        Ok(Spec {
            env_meta_base64,
            env_meta,
            meta_base64,
            meta,
            spec_base64,
            spec,
        })
    }

    pub fn spec_as_json_array(&self) -> Result<String, Error> {
        let spec = self
            .spec
            .iter()
            .map(|e| Ok(format!("\"{}\"", e.to_xdr_base64(Limits::none())?)))
            .collect::<Result<Vec<_>, Error>>()?
            .join(",\n");
        Ok(format!("[{spec}]"))
    }
}

impl Display for Spec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(env_meta) = &self.env_meta_base64 {
            writeln!(f, "Env Meta: {env_meta}")?;
            for env_meta_entry in &self.env_meta {
                match env_meta_entry {
                    ScEnvMetaEntry::ScEnvMetaKindInterfaceVersion(v) => {
                        writeln!(f, " • Interface Version: {v}")?;
                    }
                }
            }
            writeln!(f)?;
        } else {
            writeln!(f, "Env Meta: None\n")?;
        }

        if let Some(_meta) = &self.meta_base64 {
            writeln!(f, "Contract Meta:")?;
            for meta_entry in &self.meta {
                match meta_entry {
                    ScMetaEntry::ScMetaV0(ScMetaV0 { key, val }) => {
                        writeln!(f, " • {key}: {val}")?;
                    }
                }
            }
            writeln!(f)?;
        } else {
            writeln!(f, "Contract Meta: None\n")?;
        }

        if let Some(_spec_base64) = &self.spec_base64 {
            writeln!(f, "Contract Spec:")?;
            for spec_entry in &self.spec {
                match spec_entry {
                    ScSpecEntry::FunctionV0(func) => write_func(f, func)?,
                    ScSpecEntry::UdtUnionV0(udt) => write_union(f, udt)?,
                    ScSpecEntry::UdtStructV0(udt) => write_struct(f, udt)?,
                    ScSpecEntry::UdtEnumV0(udt) => write_enum(f, udt)?,
                    ScSpecEntry::UdtErrorEnumV0(udt) => write_error(f, udt)?,
                }
            }
        } else {
            writeln!(f, "Contract Spec: None")?;
        }
        Ok(())
    }
}

fn write_func(f: &mut std::fmt::Formatter<'_>, func: &ScSpecFunctionV0) -> std::fmt::Result {
    writeln!(f, " • Function: {}", func.name.to_utf8_string_lossy())?;
    if func.doc.len() > 0 {
        writeln!(
            f,
            "     Docs: {}",
            &indent(&func.doc.to_utf8_string_lossy(), 11).trim()
        )?;
    }
    writeln!(
        f,
        "     Inputs: {}",
        indent(&format!("{:#?}", func.inputs), 5).trim()
    )?;
    writeln!(
        f,
        "     Output: {}",
        indent(&format!("{:#?}", func.outputs), 5).trim()
    )?;
    writeln!(f)?;
    Ok(())
}

fn write_union(f: &mut std::fmt::Formatter<'_>, udt: &ScSpecUdtUnionV0) -> std::fmt::Result {
    writeln!(f, " • Union: {}", format_name(&udt.lib, &udt.name))?;
    if udt.doc.len() > 0 {
        writeln!(
            f,
            "     Docs: {}",
            indent(&udt.doc.to_utf8_string_lossy(), 10).trim()
        )?;
    }
    writeln!(f, "     Cases:")?;
    for case in udt.cases.iter() {
        writeln!(f, "      • {}", indent(&format!("{case:#?}"), 8).trim())?;
    }
    writeln!(f)?;
    Ok(())
}

fn write_struct(f: &mut std::fmt::Formatter<'_>, udt: &ScSpecUdtStructV0) -> std::fmt::Result {
    writeln!(f, " • Struct: {}", format_name(&udt.lib, &udt.name))?;
    if udt.doc.len() > 0 {
        writeln!(
            f,
            "     Docs: {}",
            indent(&udt.doc.to_utf8_string_lossy(), 10).trim()
        )?;
    }
    writeln!(f, "     Fields:")?;
    for field in udt.fields.iter() {
        writeln!(
            f,
            "      • {}: {}",
            field.name.to_utf8_string_lossy(),
            indent(&format!("{:#?}", field.type_), 8).trim()
        )?;
        if field.doc.len() > 0 {
            writeln!(f, "{}", indent(&format!("{:#?}", field.doc), 8))?;
        }
    }
    writeln!(f)?;
    Ok(())
}

fn write_enum(f: &mut std::fmt::Formatter<'_>, udt: &ScSpecUdtEnumV0) -> std::fmt::Result {
    writeln!(f, " • Enum: {}", format_name(&udt.lib, &udt.name))?;
    if udt.doc.len() > 0 {
        writeln!(
            f,
            "     Docs: {}",
            indent(&udt.doc.to_utf8_string_lossy(), 10).trim()
        )?;
    }
    writeln!(f, "     Cases:")?;
    for case in udt.cases.iter() {
        writeln!(f, "      • {}", indent(&format!("{case:#?}"), 8).trim())?;
    }
    writeln!(f)?;
    Ok(())
}

fn write_error(f: &mut std::fmt::Formatter<'_>, udt: &ScSpecUdtErrorEnumV0) -> std::fmt::Result {
    writeln!(f, " • Error: {}", format_name(&udt.lib, &udt.name))?;
    if udt.doc.len() > 0 {
        writeln!(
            f,
            "     Docs: {}",
            indent(&udt.doc.to_utf8_string_lossy(), 10).trim()
        )?;
    }
    writeln!(f, "     Cases:")?;
    for case in udt.cases.iter() {
        writeln!(f, "      • {}", indent(&format!("{case:#?}"), 8).trim())?;
    }
    writeln!(f)?;
    Ok(())
}

fn indent(s: &str, n: usize) -> String {
    let pad = " ".repeat(n);
    s.lines()
        .map(|line| format!("{pad}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_name(lib: &StringM<80>, name: &StringM<60>) -> String {
    if lib.len() > 0 {
        format!(
            "{}::{}",
            lib.to_utf8_string_lossy(),
            name.to_utf8_string_lossy()
        )
    } else {
        name.to_utf8_string_lossy()
    }
}
