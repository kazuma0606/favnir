use std::fmt;
use std::io::{self, Read, Write};

use super::codegen::Constant;

const MAGIC: &[u8; 4] = b"FVC\x01";
const VERSION: u8 = 0x06;

#[derive(Debug, Clone, PartialEq)]
pub struct FvcGlobal {
    pub name_idx: u32,
    pub kind: u8,
    pub fn_idx: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FvcFunction {
    pub name_idx: u32,
    pub param_count: u32,
    pub local_count: u32,
    pub source_line: u32,
    pub return_ty_str_idx: u32,
    pub effect_str_idx: u32,
    pub constants: Vec<Constant>,
    pub code: Vec<u8>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct FvcWriter {
    pub str_table: Vec<String>,
    pub globals: Vec<FvcGlobal>,
    pub functions: Vec<FvcFunction>,
}

impl FvcWriter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn intern(&mut self, value: &str) -> u32 {
        if let Some(idx) = self.str_table.iter().position(|s| s == value) {
            return idx as u32;
        }
        let idx = self.str_table.len() as u32;
        self.str_table.push(value.to_string());
        idx
    }

    pub fn add_global(&mut self, global: FvcGlobal) {
        self.globals.push(global);
    }

    pub fn add_function(&mut self, function: FvcFunction) {
        self.functions.push(function);
    }

    pub fn write_to(&self, w: &mut impl Write) -> io::Result<()> {
        w.write_all(MAGIC)?;
        w.write_all(&[VERSION, 0, 0, 0])?;
        write_u32(w, self.str_table.len() as u32)?;
        write_u32(w, self.functions.len() as u32)?;
        write_u32(w, self.globals.len() as u32)?;

        for value in &self.str_table {
            let bytes = value.as_bytes();
            write_u32(w, bytes.len() as u32)?;
            w.write_all(bytes)?;
        }

        for function in &self.functions {
            write_u32(w, function.return_ty_str_idx)?;
            write_u32(w, function.effect_str_idx)?;
        }

        for global in &self.globals {
            write_u32(w, global.name_idx)?;
            w.write_all(&[global.kind])?;
            write_u32(w, global.fn_idx)?;
        }

        for function in &self.functions {
            write_u32(w, function.name_idx)?;
            write_u32(w, function.param_count)?;
            write_u32(w, function.local_count)?;
            write_u32(w, function.source_line)?;
            write_u32(w, function.return_ty_str_idx)?;
            write_u32(w, function.effect_str_idx)?;
            write_u32(w, function.constants.len() as u32)?;
            for constant in &function.constants {
                write_constant(w, constant)?;
            }
            write_u32(w, function.code.len() as u32)?;
            w.write_all(&function.code)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FvcArtifact {
    pub str_table: Vec<String>,
    pub globals: Vec<FvcGlobal>,
    pub functions: Vec<FvcFunction>,
}

impl FvcArtifact {
    pub fn read_from(r: &mut impl Read) -> Result<Self, ArtifactError> {
        let mut magic = [0u8; 4];
        r.read_exact(&mut magic)?;
        if &magic != MAGIC {
            return Err(ArtifactError::BadMagic(magic));
        }

        let mut rest = [0u8; 4];
        r.read_exact(&mut rest)?;
        if rest[0] != VERSION {
            return Err(ArtifactError::BadVersion(rest[0]));
        }

        let str_count = read_u32(r)? as usize;
        let fn_count = read_u32(r)? as usize;
        let global_count = read_u32(r)? as usize;

        let mut str_table = Vec::with_capacity(str_count);
        for _ in 0..str_count {
            let len = read_u32(r)? as usize;
            let mut buf = vec![0u8; len];
            r.read_exact(&mut buf)?;
            let value = String::from_utf8(buf).map_err(ArtifactError::Utf8Error)?;
            str_table.push(value);
        }

        let mut type_section = Vec::with_capacity(fn_count);
        for _ in 0..fn_count {
            let return_ty_str_idx = read_u32(r)?;
            let effect_str_idx = read_u32(r)?;
            type_section.push((return_ty_str_idx, effect_str_idx));
        }

        let mut globals = Vec::with_capacity(global_count);
        for _ in 0..global_count {
            let name_idx = read_u32(r)?;
            let mut kind = [0u8; 1];
            r.read_exact(&mut kind)?;
            let fn_idx = read_u32(r)?;
            globals.push(FvcGlobal { name_idx, kind: kind[0], fn_idx });
        }

        let mut functions = Vec::with_capacity(fn_count);
        for idx in 0..fn_count {
            let name_idx = read_u32(r)?;
            let param_count = read_u32(r)?;
            let local_count = read_u32(r)?;
            let source_line = read_u32(r)?;
            let return_ty_str_idx = read_u32(r)?;
            let effect_str_idx = read_u32(r)?;
            let (type_ret, type_eff) = type_section[idx];
            if return_ty_str_idx != type_ret || effect_str_idx != type_eff {
                return Err(ArtifactError::BadSectionLayout);
            }
            let const_count = read_u32(r)? as usize;
            let mut constants = Vec::with_capacity(const_count);
            for _ in 0..const_count {
                constants.push(read_constant(r)?);
            }
            let code_len = read_u32(r)? as usize;
            let mut code = vec![0u8; code_len];
            r.read_exact(&mut code)?;
            functions.push(FvcFunction {
                name_idx,
                param_count,
                local_count,
                source_line,
                return_ty_str_idx,
                effect_str_idx,
                constants,
                code,
            });
        }

        Ok(Self { str_table, globals, functions })
    }

    pub fn fn_idx_by_name(&self, name: &str) -> Option<usize> {
        self.functions.iter().position(|f| {
            self.str_table
                .get(f.name_idx as usize)
                .map(|s| s == name)
                .unwrap_or(false)
        })
    }
}

#[derive(Debug)]
pub enum ArtifactError {
    BadMagic([u8; 4]),
    BadVersion(u8),
    BadSectionLayout,
    IoError(io::Error),
    Utf8Error(std::string::FromUtf8Error),
}

impl fmt::Display for ArtifactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArtifactError::BadMagic(magic) => write!(f, "bad artifact magic: {magic:?}"),
            ArtifactError::BadVersion(version) => write!(f, "unsupported artifact version: {version}"),
            ArtifactError::BadSectionLayout => write!(f, "artifact section layout is inconsistent"),
            ArtifactError::IoError(err) => write!(f, "{err}"),
            ArtifactError::Utf8Error(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for ArtifactError {}

impl From<io::Error> for ArtifactError {
    fn from(value: io::Error) -> Self {
        ArtifactError::IoError(value)
    }
}

fn write_u32(w: &mut impl Write, value: u32) -> io::Result<()> {
    w.write_all(&value.to_le_bytes())
}

fn read_u32(r: &mut impl Read) -> Result<u32, ArtifactError> {
    let mut bytes = [0u8; 4];
    r.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

fn write_constant(w: &mut impl Write, constant: &Constant) -> io::Result<()> {
    match constant {
        Constant::Int(v) => {
            w.write_all(&[0x01])?;
            w.write_all(&v.to_le_bytes())
        }
        Constant::Float(v) => {
            w.write_all(&[0x02])?;
            w.write_all(&v.to_le_bytes())
        }
        Constant::Str(v) => {
            w.write_all(&[0x03])?;
            write_u32(w, v.len() as u32)?;
            w.write_all(v.as_bytes())
        }
        Constant::Name(v) => {
            w.write_all(&[0x04])?;
            let len = u16::try_from(v.len()).expect("name constant too long");
            w.write_all(&len.to_le_bytes())?;
            w.write_all(v.as_bytes())
        }
    }
}

fn read_constant(r: &mut impl Read) -> Result<Constant, ArtifactError> {
    let mut tag = [0u8; 1];
    r.read_exact(&mut tag)?;
    match tag[0] {
        0x01 => {
            let mut bytes = [0u8; 8];
            r.read_exact(&mut bytes)?;
            Ok(Constant::Int(i64::from_le_bytes(bytes)))
        }
        0x02 => {
            let mut bytes = [0u8; 8];
            r.read_exact(&mut bytes)?;
            Ok(Constant::Float(f64::from_le_bytes(bytes)))
        }
        0x03 => {
            let len = read_u32(r)? as usize;
            let mut bytes = vec![0u8; len];
            r.read_exact(&mut bytes)?;
            let value = String::from_utf8(bytes).map_err(ArtifactError::Utf8Error)?;
            Ok(Constant::Str(value))
        }
        0x04 => {
            let mut len_bytes = [0u8; 2];
            r.read_exact(&mut len_bytes)?;
            let len = u16::from_le_bytes(len_bytes) as usize;
            let mut bytes = vec![0u8; len];
            r.read_exact(&mut bytes)?;
            let value = String::from_utf8(bytes).map_err(ArtifactError::Utf8Error)?;
            Ok(Constant::Name(value))
        }
        other => Err(ArtifactError::IoError(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unknown constant tag: {other}"),
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::{ArtifactError, FvcArtifact, FvcFunction, FvcGlobal, FvcWriter};
    use crate::backend::codegen::Constant;

    #[test]
    fn writer_round_trips_artifact() {
        let mut writer = FvcWriter::new();
        let main_idx = writer.intern("main");
        let unit_idx = writer.intern("Unit");
        let io_idx = writer.intern("Io");

        writer.add_global(FvcGlobal { name_idx: main_idx, kind: 0, fn_idx: 0 });
        writer.add_function(FvcFunction {
            name_idx: main_idx,
            param_count: 0,
            local_count: 1,
            source_line: 1,
            return_ty_str_idx: unit_idx,
            effect_str_idx: io_idx,
            constants: vec![Constant::Int(1), Constant::Str("fav".into())],
            code: vec![0x01, 0x00, 0x00, 0x16],
        });

        let mut bytes = Vec::new();
        writer.write_to(&mut bytes).expect("write");
        let artifact = FvcArtifact::read_from(&mut bytes.as_slice()).expect("read");

        assert_eq!(artifact.str_table, writer.str_table);
        assert_eq!(artifact.globals, writer.globals);
        assert_eq!(artifact.functions, writer.functions);
        assert_eq!(artifact.fn_idx_by_name("main"), Some(0));
    }

    #[test]
    fn bad_magic_is_rejected() {
        let err = FvcArtifact::read_from(&mut b"NOPE".as_slice()).expect_err("bad magic");
        assert!(matches!(err, ArtifactError::BadMagic(_)));
    }
}
