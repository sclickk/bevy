use crate::{
	render_resource::{ProcessedShader, ShaderImport, Source, SHADER_IMPORT_PROCESSOR},
	Shader,
};

use bevy_asset::Handle;
use bevy_utils::{tracing::error, HashMap};
use regex::Regex;
use std::{borrow::Cow, collections::HashSet, ops::Deref};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ProcessShaderError {
	#[error("Too many '# endif' lines. Each endif should be preceded by an if statement.")]
	TooManyEndIfs,
	#[error(
		"Not enough '# endif' lines. Each if statement should be followed by an endif statement."
	)]
	NotEnoughEndIfs,
	#[error("This Shader's format does not support processing shader defs.")]
	ShaderFormatDoesNotSupportShaderDefs,
	#[error("This Shader's formatdoes not support imports.")]
	ShaderFormatDoesNotSupportImports,
	#[error("Unresolved import: {0:?}.")]
	UnresolvedImport(ShaderImport),
	#[error("The shader import {0:?} does not match the source file type. Support for this might be added in the future.")]
	MismatchedImportFormat(ShaderImport),
}

pub struct ShaderProcessor {
	ifdef_regex: Regex,
	ifndef_regex: Regex,
	else_regex: Regex,
	endif_regex: Regex,
}

impl Default for ShaderProcessor {
	fn default() -> Self {
		Self {
			ifdef_regex: Regex::new(r"^\s*#\s*ifdef\s*([\w|\d|_]+)").unwrap(),
			ifndef_regex: Regex::new(r"^\s*#\s*ifndef\s*([\w|\d|_]+)").unwrap(),
			else_regex: Regex::new(r"^\s*#\s*else").unwrap(),
			endif_regex: Regex::new(r"^\s*#\s*endif").unwrap(),
		}
	}
}

impl ShaderProcessor {
	pub fn process(
		&self,
		shader: &Shader,
		shader_defs: &[String],
		shaders: &HashMap<Handle<Shader>, Shader>,
		import_handles: &HashMap<ShaderImport, Handle<Shader>>,
	) -> Result<ProcessedShader, ProcessShaderError> {
		let shader_str = match &shader.source {
			Source::Wgsl(source) => source.deref(),
			Source::Glsl(source, _stage) => source.deref(),
			Source::SpirV(source) => {
				if shader_defs.is_empty() {
					return Ok(ProcessedShader::SpirV(source.clone()));
				}
				return Err(ProcessShaderError::ShaderFormatDoesNotSupportShaderDefs);
			},
		};

		let shader_defs_unique = HashSet::<String>::from_iter(shader_defs.iter().cloned());
		let mut scopes = vec![true];
		let mut final_string = String::new();
		for line in shader_str.lines() {
			if let Some(cap) = self.ifdef_regex.captures(line) {
				let def = cap.get(1).unwrap();
				scopes.push(*scopes.last().unwrap() && shader_defs_unique.contains(def.as_str()));
			} else if let Some(cap) = self.ifndef_regex.captures(line) {
				let def = cap.get(1).unwrap();
				scopes.push(*scopes.last().unwrap() && !shader_defs_unique.contains(def.as_str()));
			} else if self.else_regex.is_match(line) {
				let mut is_parent_scope_truthy = true;
				if scopes.len() > 1 {
					is_parent_scope_truthy = scopes[scopes.len() - 2];
				}
				if let Some(last) = scopes.last_mut() {
					*last = is_parent_scope_truthy && !*last;
				}
			} else if self.endif_regex.is_match(line) {
				scopes.pop();
				if scopes.is_empty() {
					return Err(ProcessShaderError::TooManyEndIfs);
				}
			} else if *scopes.last().unwrap() {
				if let Some(cap) = SHADER_IMPORT_PROCESSOR
					.import_asset_path_regex
					.captures(line)
				{
					let import = ShaderImport::AssetPath(cap.get(1).unwrap().as_str().to_string());
					self.apply_import(
						import_handles,
						shaders,
						&import,
						shader,
						shader_defs,
						&mut final_string,
					)?;
				} else if let Some(cap) = SHADER_IMPORT_PROCESSOR
					.import_custom_path_regex
					.captures(line)
				{
					let import = ShaderImport::Custom(cap.get(1).unwrap().as_str().to_string());
					self.apply_import(
						import_handles,
						shaders,
						&import,
						shader,
						shader_defs,
						&mut final_string,
					)?;
				} else if SHADER_IMPORT_PROCESSOR
					.define_import_path_regex
					.is_match(line)
				{
					// ignore import path lines
				} else {
					final_string.push_str(line);
					final_string.push('\n');
				}
			}
		}

		(scopes.len() == 1)
			.then(|| {
				let processed_source = Cow::from(final_string);

				match &shader.source {
					Source::Wgsl(_source) => ProcessedShader::Wgsl(processed_source),
					Source::Glsl(_source, stage) => ProcessedShader::Glsl(processed_source, *stage),
					Source::SpirV(_source) => {
						unreachable!("SpirV has early return");
					},
				}
			})
			.ok_or(ProcessShaderError::NotEnoughEndIfs)
	}

	fn apply_import(
		&self,
		import_handles: &HashMap<ShaderImport, Handle<Shader>>,
		shaders: &HashMap<Handle<Shader>, Shader>,
		import: &ShaderImport,
		shader: &Shader,
		shader_defs: &[String],
		final_string: &mut String,
	) -> Result<(), ProcessShaderError> {
		let imported_shader = import_handles
			.get(import)
			.and_then(|handle| shaders.get(handle))
			.ok_or(ProcessShaderError::UnresolvedImport(import.clone()))?;
		let imported_processed = self.process(imported_shader, shader_defs, shaders, import_handles)?;

		match &shader.source {
			Source::Wgsl(_) => {
				if let ProcessedShader::Wgsl(import_source) = &imported_processed {
					final_string.push_str(import_source);
				} else {
					return Err(ProcessShaderError::MismatchedImportFormat(import.clone()));
				}
			},
			Source::Glsl(_, _) => {
				if let ProcessedShader::Glsl(import_source, _) = &imported_processed {
					final_string.push_str(import_source);
				} else {
					return Err(ProcessShaderError::MismatchedImportFormat(import.clone()));
				}
			},
			Source::SpirV(_) => {
				return Err(ProcessShaderError::ShaderFormatDoesNotSupportImports);
			},
		}

		Ok(())
	}
}
